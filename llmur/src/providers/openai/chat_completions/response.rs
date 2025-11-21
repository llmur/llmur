use serde::{Deserialize, Serialize};
use crate::providers::ExposesUsage;

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
    pub service_tier: Option<String>,
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
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ResponseChoiceToolCall>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseChoiceToolCall {
    #[serde(rename = "function", alias = "function")]
    Function { id: String, function: ResponseChoiceFunctionToolCall },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseUsage {
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoiceLogprob {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ResponseChoiceLogprobContent>>,
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
    use crate::providers::{Transformation, TransformationContext, TransformationLoss, Transformer};
    pub struct Loss {}
    pub struct Context { pub model: Option<String> }

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