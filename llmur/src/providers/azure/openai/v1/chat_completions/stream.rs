use serde::{Deserialize, Serialize};

use crate::providers::azure::openai::v1::chat_completions::response::{
    AzureContentFilterResultForChoice, AzureContentFilterResultForPrompt, ResponseChoiceLogprob,
    ResponseUsage,
};

// region: --- Stream response structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_filter_results: Option<Vec<AzureContentFilterResultForPrompt>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_filter_results: Option<AzureContentFilterResultForChoice>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoice {
    pub index: u64,
    pub delta: ResponseChoiceDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<ResponseChoiceLogprob>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub struct ResponseChoiceDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ResponseChoiceToolCallChunk>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<ResponseChoiceDeltaFunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoiceDeltaFunctionCall {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoiceToolCallChunk {
    pub index: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub tool_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<ResponseChoiceToolCallChunkFunction>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoiceToolCallChunkFunction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}
// endregion: --- Stream response structs

#[cfg(test)]
mod tests {
    use super::Response;

    #[test]
    fn stream_chunk_with_reasoning_content_parses() {
        let json = r#"{
            "id": "chatcmpl-1",
            "object": "chat.completion.chunk",
            "created": 10,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "delta": {
                    "role": "assistant",
                    "content": "hi",
                    "reasoning_content": "trace",
                    "tool_calls": [{
                        "index": 0,
                        "id": "call_1",
                        "type": "function",
                        "function": { "name": "f", "arguments": "{}" }
                    }]
                },
                "finish_reason": null,
                "logprobs": null
            }],
            "usage": null
        }"#;

        let response: Response = serde_json::from_str(json).expect("parse stream chunk");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].delta.content.as_deref(), Some("hi"));
    }
}
