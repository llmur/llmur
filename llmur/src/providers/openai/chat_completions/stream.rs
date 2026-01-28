use serde::Serialize;

// region: --- Stream response structs
/// OpenAI chat completion stream chunk.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Response {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ResponseChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ResponseUsage>,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct ResponseChoice {
    pub index: u64,
    pub delta: ResponseChoiceDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<serde_json::Value>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Default)]
pub struct ResponseChoiceDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ResponseChoiceToolCall>>,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct ResponseChoiceToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ResponseChoiceFunctionToolCall,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct ResponseChoiceFunctionToolCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct ResponseUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}
// endregion: --- Stream response structs
