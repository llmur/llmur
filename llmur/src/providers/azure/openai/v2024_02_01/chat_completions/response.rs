use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Response {
    pub choices: Vec<ResponseChoice>,
    pub created: u64,
    pub id: String,
    pub model: String,
    pub object: String,
    pub system_fingerprint: Option<String>,
    pub usage: ResponseUsage,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseUsage {
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoice {
    pub finish_reason: String,
    pub index: u64,
    pub message: ResponseChoiceMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_filter_results: Option<ResponseContentFilterResults>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseContentFilterResults {
    // TODO
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoiceMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ResponseChoiceToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<AzureMessageContext>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureMessageContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<AzureContextCitation>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContextCitation {
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseChoiceToolCall {
    #[serde(rename = "function", alias = "function")]
    Function { id: String, function: ResponseChoiceFunctionToolCall },
}

#[derive(Debug, PartialEq, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ResponseChoiceFunctionToolCall {
    pub name: String,
    pub arguments: String,
}

pub mod to_openai_transform {
    use crate::providers::azure::openai::v2024_02_01::chat_completions::response::{
        Response as AzureResponse,
        ResponseUsage as AzureResponseUsage,
        ResponseChoice as AzureResponseChoice,
        ResponseChoiceMessage as AzureResponseChoiceMessage,
        ResponseChoiceToolCall as AzureResponseChoiceToolCall,
        ResponseChoiceFunctionToolCall as AzureResponseChoiceFunctionToolCall,
    };
    use crate::providers::openai::chat_completions::response::{
        Response as OpenAiResponse,
        ResponseUsage as OpenAiResponseUsage,
        ResponseChoice as OpenAiResponseChoice,
        ResponseChoiceMessage as OpenAiResponseChoiceMessage,
        ResponseChoiceToolCall as OpenAiResponseChoiceToolCall,
        ResponseChoiceFunctionToolCall as OpenAiResponseChoiceFunctionToolCall,
    };
    use crate::providers::{Transformation, TransformationContext, TransformationLoss, Transformer};

    pub struct Loss {}

    pub struct Context {
        pub model: Option<String>
    }

    impl TransformationContext<AzureResponse, OpenAiResponse> for Context {}
    impl TransformationLoss<AzureResponse, OpenAiResponse> for Loss {}

    impl Transformer<OpenAiResponse, Context, Loss> for AzureResponse {
        fn transform(self, context: Context) -> Transformation<OpenAiResponse, Loss> {
            Transformation {
                result: OpenAiResponse {
                    id: self.id,
                    choices: self.choices.into_iter().map(Into::into).collect(),
                    created: self.created,
                    model: context.model.unwrap_or(self.model),
                    system_fingerprint: self.system_fingerprint,
                    object: self.object,
                    usage: self.usage.into(),
                    service_tier: None,
                },
                loss: Loss {},
            }
        }
    }

    impl From<AzureResponseUsage> for OpenAiResponseUsage {
        fn from(value: AzureResponseUsage) -> Self {
            OpenAiResponseUsage {
                completion_tokens: value.completion_tokens,
                prompt_tokens: value.prompt_tokens,
                total_tokens: value.total_tokens,
            }
        }
    }

    impl From<AzureResponseChoice> for OpenAiResponseChoice {
        fn from(value: AzureResponseChoice) -> Self {
            OpenAiResponseChoice {
                finish_reason: value.finish_reason,
                index: value.index,
                message: value.message.into(),
                logprobs: None,
            }
        }
    }
    impl From<AzureResponseChoiceMessage> for OpenAiResponseChoiceMessage {
        fn from(value: AzureResponseChoiceMessage) -> Self {
            OpenAiResponseChoiceMessage {
                content: value.content,
                role: value.role,
                tool_calls: value.tool_calls.map(|t| t.into_iter().map(Into::into).collect()),
            }
        }
    }
    impl From<AzureResponseChoiceToolCall> for OpenAiResponseChoiceToolCall {
        fn from(value: AzureResponseChoiceToolCall) -> Self {
            match value {
                AzureResponseChoiceToolCall::Function { id, function } =>
                    OpenAiResponseChoiceToolCall::Function {
                        id,
                        function: function.into()
                    }
            }
        }
    }
    impl From<AzureResponseChoiceFunctionToolCall> for OpenAiResponseChoiceFunctionToolCall {
        fn from(value: AzureResponseChoiceFunctionToolCall) -> Self {
            OpenAiResponseChoiceFunctionToolCall {
                name: value.name,
                arguments: value.arguments
            }
        }
    }
}