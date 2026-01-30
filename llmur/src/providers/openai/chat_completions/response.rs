use crate::providers::ExposesUsage;
use crate::providers::openai::chat_completions::request::ServiceTier;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub choices: Vec<ResponseChoice>,
    pub created: u64,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    pub object: String,
    pub usage: ResponseUsage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoice {
    pub finish_reason: String,
    pub index: u64,
    pub message: ResponseChoiceMessage,
    pub logprobs: Option<ResponseChoiceLogprob>, // nullable but required
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoiceMessage {
    pub content: Option<String>, // nullable but required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ResponseChoiceToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<ResponseMessageAnnotation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<ResponseMessageAudio>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<ResponseChoiceFunctionToolCall>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseMessageAnnotation {
    #[serde(rename = "url_citation", alias = "url_citation")]
    UrlCitation { url_citation: ResponseUrlCitation },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseUrlCitation {
    pub start_index: u64,
    pub end_index: u64,
    pub url: String,
    pub title: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseMessageAudio {
    pub id: String,
    pub expires_at: u64,
    pub data: String,
    pub transcript: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseChoiceToolCall {
    #[serde(rename = "function", alias = "function")]
    Function {
        id: String,
        function: ResponseChoiceFunctionToolCall,
    },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseUsage {
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    pub total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CompletionTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_prediction_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_prediction_tokens: Option<u64>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u64>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoiceLogprob {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ResponseChoiceLogprobContent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<Vec<ResponseChoiceLogprobContent>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoiceLogprobContent {
    pub token: String,
    pub logprob: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
    pub top_logprobs: Vec<ResponseChoiceTopLogprobContent>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoiceTopLogprobContent {
    pub token: String,
    pub logprob: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoiceFunctionToolCall {
    pub name: String,
    pub arguments: String,
}

impl ExposesUsage for Response {
    fn get_input_tokens(&self) -> u64 {
        self.usage.prompt_tokens
    }
    fn get_output_tokens(&self) -> u64 {
        self.usage.completion_tokens
    }
}

pub mod to_self {
    use crate::providers::openai::chat_completions::response::Response;
    use crate::providers::{
        Transformation, TransformationContext, TransformationLoss, Transformer,
    };

    #[derive(Debug)]
    pub struct Loss {}

    #[derive(Debug)]
    pub struct Context {
        pub model: Option<String>,
    }

    impl TransformationContext<Response, Response> for Context {}
    impl TransformationLoss<Response, Response> for Loss {}

    impl Transformer<Response, Context, Loss> for Response {
        fn transform(self, context: Context) -> Transformation<Response, Loss> {
            Transformation {
                result: Response {
                    id: self.id,
                    choices: self.choices,
                    created: self.created,
                    model: context.model.unwrap_or(self.model),
                    system_fingerprint: self.system_fingerprint,
                    object: self.object,
                    usage: self.usage,
                    service_tier: self.service_tier,
                },
                loss: Loss {},
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Response;

    #[test]
    fn response_optional_usage_details_and_refusal_logprobs() {
        let json = r#"{
            "id": "chatcmpl-test",
            "object": "chat.completion",
            "created": 1,
            "model": "gpt-4o",
            "choices": [{
                "finish_reason": "stop",
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "hi",
                    "refusal": null
                },
                "logprobs": {
                    "content": null,
                    "refusal": null
                }
            }],
            "usage": {
                "prompt_tokens": 1,
                "completion_tokens": 1,
                "total_tokens": 2
            }
        }"#;

        let response: Response = serde_json::from_str(json).expect("parse response");
        assert_eq!(response.choices.len(), 1);
        assert!(response.usage.completion_tokens_details.is_none());
        assert!(response.usage.prompt_tokens_details.is_none());
    }
}
