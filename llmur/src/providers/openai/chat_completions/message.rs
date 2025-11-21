use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum Message {
    #[serde(rename = "system", alias = "system")]
    SystemMessage {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },

    #[serde(rename = "user", alias = "user")]
    UserMessage {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        content: UserMessageContent,
    },

    #[serde(rename = "assistant", alias = "assistant")]
    AssistantMessage {
        //TODO Confirm if content is optional
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<AssistantToolCall>>,
    },

    #[serde(rename = "tool", alias = "tool")]
    ToolMessage {
        content: String,
        tool_call_id: String
    },
}

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
    ImageUrl { image_url: ImageUrlContentPart },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ImageUrlContentPart {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
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
    #[serde(rename = "function")]
    FunctionType,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantToolCallFunction {
    pub name: String,
    pub arguments: String,
}