use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum Message {
    #[serde(rename = "system", alias = "system")]
    SystemMessage { content: String },

    #[serde(rename = "user", alias = "user")]
    UserMessage { content: UserMessageContent },

    #[serde(rename = "assistant", alias = "assistant")]
    AssistantMessage {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<AssistantToolCall>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<AssistantMessageContext>,
    },

    #[serde(rename = "tool", alias = "tool")]
    ToolMessage {
        content: String,
        tool_call_id: String,
    },
}

// region:    --- Chat Completion Message Content
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserMessageContent {
    Text(String),
    Array(Vec<UserMessageContentPart>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UserMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
    #[serde(rename = "image_url", alias = "image_url")]
    ImageUrl { image_url: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: AssistantToolCallType,
    pub function: AssistantToolCallFunction,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum AssistantToolCallType {
    #[serde(rename = "function", alias = "function")]
    FunctionType,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantMessageContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<AssistantContextCitation>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantContextCitation {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filepath: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_id: Option<String>,
}
// endregion  --- Chat Completion Message Content

pub mod from_openai_transform {
    use crate::providers::azure::openai::v2024_02_01::chat_completions::message::{
        AssistantToolCall as AzureAssistantToolCall,
        AssistantToolCallFunction as AzureAssistantToolCallFunction,
        AssistantToolCallType as AzureAssistantToolCallType,
        Message as AzureMessage,
        UserMessageContent as AzureUserMessageContent,
        UserMessageContentPart as AzureUserMessageContentPart
    };
    use crate::providers::openai::chat_completions::message::{AssistantToolCall as OpenAiAssistantToolCall, AssistantToolCallFunction as OpenAiAssistantToolCallFunction, AssistantToolCallType as OpenAiAssistantToolCallType, Message as OpenAiMessage, UserMessageContent as OpenAiUserMessageContent, UserMessageContentPart as OpenAiUserMessageContentPart};

    impl From<OpenAiMessage> for AzureMessage {
        fn from(value: OpenAiMessage) -> Self {
            match value {
                OpenAiMessage::SystemMessage { content, .. } => {
                    AzureMessage::SystemMessage { content }
                }
                OpenAiMessage::UserMessage {  content, .. } => {
                    AzureMessage::UserMessage { content: content.into() }
                }
                OpenAiMessage::AssistantMessage { content, tool_calls, .. } => {
                    AzureMessage::AssistantMessage {
                        content,
                        tool_calls: tool_calls.map(|values| values.into_iter().map(Into::into).collect()),
                        context: None,
                    }
                }
                OpenAiMessage::ToolMessage { content, tool_call_id } => {
                    AzureMessage::ToolMessage { content, tool_call_id }
                }
            }
        }
    }

    impl From<OpenAiUserMessageContent> for AzureUserMessageContent {
        fn from(value: OpenAiUserMessageContent) -> Self {
            match value {
                OpenAiUserMessageContent::Text(s) => { AzureUserMessageContent::Text(s) }
                OpenAiUserMessageContent::Array(v) => { AzureUserMessageContent::Array(v.into_iter().map(Into::into).collect()) }
            }
        }
    }

    impl From<OpenAiUserMessageContentPart> for AzureUserMessageContentPart {
        fn from(value: OpenAiUserMessageContentPart) -> Self {
            match value {
                OpenAiUserMessageContentPart::Text { text } => {
                    AzureUserMessageContentPart::Text { text }
                }
                OpenAiUserMessageContentPart::ImageUrl { image_url } => {
                    AzureUserMessageContentPart::ImageUrl { image_url: image_url.url }
                }
            }
        }
    }

    impl From<OpenAiAssistantToolCall> for AzureAssistantToolCall {
        fn from(value: OpenAiAssistantToolCall) -> Self {
            AzureAssistantToolCall {
                id: value.id,
                r#type: value.r#type.into(),
                function: value.function.into(),
            }
        }
    }
    impl From<OpenAiAssistantToolCallType> for AzureAssistantToolCallType {
        fn from(value: OpenAiAssistantToolCallType) -> Self {
            match value {
                OpenAiAssistantToolCallType::FunctionType => {
                    AzureAssistantToolCallType::FunctionType
                }
            }
        }
    }
    impl From<OpenAiAssistantToolCallFunction> for AzureAssistantToolCallFunction {
        fn from(value: OpenAiAssistantToolCallFunction) -> Self {
            AzureAssistantToolCallFunction { name: value.name, arguments: value.arguments }
        }
    }
}