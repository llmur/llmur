use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum Message {
    #[serde(rename = "system", alias = "system")]
    SystemMessage {
        content: SystemMessageContent,
        name: Option<String>,
    },

    #[serde(rename = "user", alias = "user")]
    UserMessage {
        content: UserMessageContent,
        name: Option<String>,
    },

    #[serde(rename = "assistant", alias = "assistant")]
    AssistantMessage {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<AssistantMessageContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        refusal: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<AssistantToolCall>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },

    #[serde(rename = "tool", alias = "tool")]
    ToolMessage {
        content: ToolMessageContent,
        tool_call_id: String,
    },

    #[serde(rename = "function", alias = "function")]
    FunctionMessage { content: String, name: String },
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
#[serde(untagged)]
pub enum SystemMessageContent {
    Text(String),
    Array(Vec<SystemMessageContentPart>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SystemMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AssistantMessageContent {
    Text(String),
    Array(Vec<AssistantMessageContentPart>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AssistantMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
    #[serde(rename = "refusal", alias = "refusal")]
    Refusal { text: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolMessageContent {
    Text(String),
    Array(Vec<ToolMessageContentPart>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AssistantToolCall {
    Function {
        id: String,
        function: AssistantToolCallFunction,
    },
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
    use crate::providers::azure::openai::v2024_10_21::chat_completions::message::{
        AssistantToolCall as AzureAssistantToolCall,
        AssistantToolCallFunction as AzureAssistantToolCallFunction,
        AssistantToolCallType as AzureAssistantToolCallType,
        Message as AzureMessage,
        SystemMessageContent as AzureSystemMessageContent,
        SystemMessageContentPart as AzureSystemMessageContentPart,
        UserMessageContent as AzureUserMessageContent,
        UserMessageContentPart as AzureUserMessageContentPart,
        AssistantMessageContent as AzureAssistantMessageContent,
        AssistantMessageContentPart as AzureAssistantMessageContentPart,
        ToolMessageContent as AzureToolMessageContent,
        ToolMessageContentPart as AzureToolMessageContentPart,
    };
    use crate::providers::openai::chat_completions::message::{
        AssistantToolCall as OpenAiAssistantToolCall,
        AssistantToolCallFunction as OpenAiAssistantToolCallFunction,
        AssistantToolCallType as OpenAiAssistantToolCallType,
        Message as OpenAiMessage,
        SystemMessageContent as OpenAiSystemMessageContent,
        SystemMessageContentPart as OpenAiSystemMessageContentPart,
        UserMessageContent as OpenAiUserMessageContent,
        UserMessageContentPart as OpenAiUserMessageContentPart,
        AssistantMessageContent as OpenAiAssistantMessageContent,
        AssistantMessageContentPart as OpenAiAssistantMessageContentPart,
        ToolMessageContent as OpenAiToolMessageContent,
        ToolMessageContentPart as OpenAiToolMessageContentPart,
        DeveloperMessageContent as OpenAiDeveloperMessageContent,
        DeveloperMessageContentPart as OpenAiDeveloperMessageContentPart,
        };

    impl From<OpenAiMessage> for AzureMessage {
        fn from(value: OpenAiMessage) -> Self {
            match value {
                OpenAiMessage::SystemMessage { content, name } => AzureMessage::SystemMessage {
                    content: content.into(),
                    name,
                },
                OpenAiMessage::UserMessage { name, content } => AzureMessage::UserMessage {
                    content: content.into(),
                    name,
                },
                OpenAiMessage::AssistantMessage {
                    content,
                    name,
                    refusal,
                    tool_calls,
                    ..
                } => AzureMessage::AssistantMessage {
                    content: content.map(Into::into),
                    refusal,
                    tool_calls: tool_calls
                        .map(|v|
                            v.into_iter()
                            .filter_map(|part| part.try_into().ok())
                            .collect()
                        ),
                    name,
                },
                OpenAiMessage::ToolMessage {
                    content,
                    tool_call_id,
                } => AzureMessage::ToolMessage {
                    content: content.into(),
                    tool_call_id,
                },
                OpenAiMessage::DeveloperMessage { content, name } => {
                    AzureMessage::SystemMessage {
                        // Convert developer message into System Message
                        content: content.into(),
                        name,
                    }
                }
            }
        }
    }

    impl From<OpenAiUserMessageContent> for AzureUserMessageContent {
        fn from(value: OpenAiUserMessageContent) -> Self {
            match value {
                OpenAiUserMessageContent::Text(s) => AzureUserMessageContent::Text(s),
                OpenAiUserMessageContent::Array(vec) => AzureUserMessageContent::Array(
                    vec.into_iter()
                        .filter_map(|part| part.try_into().ok())
                        .collect(),
                ),
            }
        }
    }
    impl TryFrom<OpenAiUserMessageContentPart> for AzureUserMessageContentPart {
        type Error = ();

        fn try_from(value: OpenAiUserMessageContentPart) -> Result<Self, Self::Error> {
            match value {
                OpenAiUserMessageContentPart::Text { text } => {
                    Ok(AzureUserMessageContentPart::Text { text })
                }
                OpenAiUserMessageContentPart::ImageUrl { image_url } => {
                    Ok(AzureUserMessageContentPart::ImageUrl {
                        image_url: image_url.url,
                    })
                }
                _ => Err(()),
            }
        }
    }

    impl TryFrom<OpenAiAssistantToolCall> for AzureAssistantToolCall {
        type Error = ();

        fn try_from(value: OpenAiAssistantToolCall) -> Result<Self, Self::Error> {
            match value {
                OpenAiAssistantToolCall::Function { id, function } => {
                    Ok(AzureAssistantToolCall::Function {
                        id,
                        function: function.into(),
                    })
                }
                OpenAiAssistantToolCall::Custom { .. } => Err(()),
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
            AzureAssistantToolCallFunction {
                name: value.name,
                arguments: value.arguments,
            }
        }
    }

    impl From<OpenAiToolMessageContent> for AzureToolMessageContent {
        fn from(value: OpenAiToolMessageContent) -> Self {
            match value {
                OpenAiToolMessageContent::Text(s) => {
                    AzureToolMessageContent::Text(s)
                }
                OpenAiToolMessageContent::Array(parts) => {
                    AzureToolMessageContent::Array(parts.into_iter().map(|part| part.into()).collect())
                }
            }
        }
    }
    impl From<OpenAiToolMessageContentPart> for AzureToolMessageContentPart {
        fn from(value: OpenAiToolMessageContentPart) -> Self {
            match value {
                OpenAiToolMessageContentPart::Text { text } =>
                    AzureToolMessageContentPart::Text { text },
            }
        }
    }

    impl From<OpenAiSystemMessageContent> for AzureSystemMessageContent {
        fn from(value: OpenAiSystemMessageContent) -> Self {
            match value {
                OpenAiSystemMessageContent::Text(s) => AzureSystemMessageContent::Text(s),
                OpenAiSystemMessageContent::Array(vec) => {
                    AzureSystemMessageContent::Array(vec.into_iter().map(Into::into).collect())
                }
            }
        }
    }
    impl From<OpenAiSystemMessageContentPart> for AzureSystemMessageContentPart {
        fn from(value: OpenAiSystemMessageContentPart) -> Self {
            match value {
                OpenAiSystemMessageContentPart::Text { text } => {
                    AzureSystemMessageContentPart::Text { text }
                }
            }
        }
    }

    impl From<OpenAiDeveloperMessageContent> for AzureSystemMessageContent {
        fn from(value: OpenAiDeveloperMessageContent) -> Self {
            match value {
                OpenAiDeveloperMessageContent::Text(s) => AzureSystemMessageContent::Text(s),
                OpenAiDeveloperMessageContent::Array(vec) => {
                    AzureSystemMessageContent::Array(vec.into_iter().map(Into::into).collect())
                }
            }
        }
    }
    impl From<OpenAiDeveloperMessageContentPart> for AzureSystemMessageContentPart {
        fn from(value: OpenAiDeveloperMessageContentPart) -> Self {
            match value {
                OpenAiDeveloperMessageContentPart::Text { text } => {
                    AzureSystemMessageContentPart::Text { text }
                }
            }
        }
    }

    impl From<OpenAiAssistantMessageContent> for AzureAssistantMessageContent {
        fn from(value: OpenAiAssistantMessageContent) -> Self {
            match value {
                OpenAiAssistantMessageContent::Text(s) => AzureAssistantMessageContent::Text(s),
                OpenAiAssistantMessageContent::Array(vec) => {
                    AzureAssistantMessageContent::Array(vec.into_iter().map(Into::into).collect())
                }
            }
        }
    }
    impl From<OpenAiAssistantMessageContentPart> for AzureAssistantMessageContentPart {
        fn from(value: OpenAiAssistantMessageContentPart) -> Self {
            match value {
                OpenAiAssistantMessageContentPart::Text { text } => {
                    AzureAssistantMessageContentPart::Text { text }
                }
                OpenAiAssistantMessageContentPart::Refusal { text } => {
                    AzureAssistantMessageContentPart::Refusal { text }
                }
            }
        }
    }


}
