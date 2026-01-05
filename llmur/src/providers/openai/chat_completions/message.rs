use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum Message {
    #[serde(rename = "system", alias = "system")]
    SystemMessage {
        content: SystemMessageContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },

    #[serde(rename = "developer", alias = "developer")]
    DeveloperMessage {
        content: DeveloperMessageContent,
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
        #[serde(skip_serializing_if = "Option::is_none")]
        audio: Option<AssistantMessageAudio>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<AssistantMessageContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        refusal: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<AssistantToolCall>>,
    },

    #[serde(rename = "tool", alias = "tool")]
    ToolMessage {
        content: ToolMessageContent,
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
#[serde(untagged)]
pub enum DeveloperMessageContent {
    Text(String),
    Array(Vec<DeveloperMessageContentPart>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemMessageContent {
    Text(String),
    Array(Vec<SystemMessageContentPart>),
}


#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolMessageContent {
    Text(String),
    Array(Vec<ToolMessageContentPart>),
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
#[serde(tag = "type")]
pub enum SystemMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DeveloperMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantMessageAudio {
    id: String
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UserMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
    #[serde(rename = "image_url", alias = "image_url")]
    ImageUrl { image_url: ImageUrlContentPart },
    #[serde(rename = "input_audio", alias = "input_audio")]
    InputAudio { input_audio: AudioContentPart },
    #[serde(rename = "file", alias = "file")]
    File { file: FileContentPart },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum FileContentPart {
    #[serde(rename = "file_data", alias = "file_data")]
    FileData(String),
    #[serde(rename = "file_id", alias = "file_id")]
    FileId(String),
    #[serde(rename = "file_name", alias = "file_name")]
    FileName(String),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ImageUrlContentPart {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AudioContentPart {
    pub data: String,
    pub format: String,
}


#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AssistantToolCall {
    #[serde(rename = "function", alias = "function")]
    Function {
        id: String,
        function: AssistantToolCallFunction,
    },
    #[serde(rename = "custom", alias = "custom")]
    Custom {
        id: String,
        custom: AssistantToolCallCustom,
    }
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantToolCallCustom {
    pub input: String,
    pub name: String,
}