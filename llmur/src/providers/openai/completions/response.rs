use crate::providers::ExposesUsage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub object: ResponseObjectType,
    pub created: u64,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    pub choices: Vec<ResponseChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ResponseUsage>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoice {
    pub text: String,
    pub index: u64,
    pub logprobs: Option<ResponseChoiceLogprobs>,
    pub finish_reason: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoiceLogprobs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_offset: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_logprobs: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<HashMap<String, f64>>>,
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
#[serde(rename_all = "snake_case")]
pub enum ResponseObjectType {
    TextCompletion,
}

impl ExposesUsage for Response {
    fn get_input_tokens(&self) -> u64 {
        self.usage
            .as_ref()
            .map(|usage| usage.prompt_tokens)
            .unwrap_or(0)
    }

    fn get_output_tokens(&self) -> u64 {
        self.usage
            .as_ref()
            .map(|usage| usage.completion_tokens)
            .unwrap_or(0)
    }
}

pub mod to_self {
    use crate::providers::openai::completions::response::Response;
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
                    model: context.model.unwrap_or(self.model),
                    ..self
                },
                loss: Loss {},
            }
        }
    }
}
