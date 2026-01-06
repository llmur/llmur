use serde::{Deserialize, Serialize};

// region: --- Response structs
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
    pub completion_tokens_details: CompletionTokensDetails
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CompletionTokensDetails {
    pub reasoning_tokens: u64,
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoiceFunctionToolCall {
    pub name: String,
    pub arguments: String,
}
// endregion: --- Response structs

// region: --- Transform methods
pub mod to_openai_transform {
    use super::*;
    use crate::providers::openai::chat_completions::response::{
        Response as OpenAiResponse,
        ResponseUsage as OpenAiResponseUsage,
        ResponseChoice as OpenAiResponseChoice,
        ResponseChoiceMessage as OpenAiResponseChoiceMessage,
        ResponseChoiceToolCall as OpenAiResponseChoiceToolCall,
        ResponseChoiceFunctionToolCall as OpenAiResponseChoiceFunctionToolCall,
        CompletionTokensDetails as OpenAiCompletionTokensDetails,
        PromptTokensDetails as OpenAiPromptTokensDetails,
    };
    use crate::providers::{Transformation, TransformationContext, TransformationLoss, Transformer};

    #[derive(Debug)]
    pub struct Loss {}

    #[derive(Debug)]
    pub struct Context {
        pub model: Option<String>
    }

    impl TransformationContext<Response, OpenAiResponse> for Context {}
    impl TransformationLoss<Response, OpenAiResponse> for Loss {}

    impl Transformer<OpenAiResponse, Context, Loss> for Response {
        fn transform(self, context: Context) -> Transformation<OpenAiResponse, Loss> {
            Transformation {
                result: OpenAiResponse {
                    id: self.id,
                    choices: self.choices.into_iter().map(|choice| transform_response_choice(choice)).collect(),
                    created: self.created,
                    model: context.model.unwrap_or(self.model),
                    system_fingerprint: self.system_fingerprint,
                    object: self.object,
                    usage: transform_response_usage(self.usage),
                    service_tier: None,
                },
                loss: Loss {},
            }
        }
    }

    fn transform_response_usage(usage: ResponseUsage) -> OpenAiResponseUsage {
        OpenAiResponseUsage {
            completion_tokens: usage.completion_tokens,
            prompt_tokens: usage.prompt_tokens,
            total_tokens: usage.total_tokens,
            completion_tokens_details: OpenAiCompletionTokensDetails {
                accepted_prediction_tokens: 0,
                audio_tokens: 0,
                reasoning_tokens: usage.completion_tokens_details.reasoning_tokens,
                rejected_prediction_tokens: 0,
            },
            prompt_tokens_details: OpenAiPromptTokensDetails { 
                audio_tokens: 0, 
                cached_tokens: 0 
            },
        }
    }

    fn transform_response_choice(choice: ResponseChoice) -> OpenAiResponseChoice {
        OpenAiResponseChoice {
            finish_reason: choice.finish_reason,
            index: choice.index,
            message: transform_response_choice_message(choice.message),
            logprobs: None,
        }
    }

    fn transform_response_choice_message(message: ResponseChoiceMessage) -> OpenAiResponseChoiceMessage {
        OpenAiResponseChoiceMessage {
            content: message.content,
            role: message.role,
            tool_calls: message.tool_calls.map(|tool_calls| {
                tool_calls.into_iter().map(|tc| transform_response_choice_tool_call(tc)).collect()
            }),
        }
    }

    fn transform_response_choice_tool_call(tool_call: ResponseChoiceToolCall) -> OpenAiResponseChoiceToolCall {
        match tool_call {
            ResponseChoiceToolCall::Function { id, function } => OpenAiResponseChoiceToolCall::Function {
                id,
                function: OpenAiResponseChoiceFunctionToolCall {
                    name: function.name,
                    arguments: function.arguments,
                },
            },
        }
    }
}
// endregion: --- Transform methods
