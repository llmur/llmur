use serde::{Deserialize, Serialize};

use crate::providers::azure::openai::v1::common::{
    AzureContentFilterBlocklistResult, AzureContentFilterCompletionTextSpanDetectionResult,
    AzureContentFilterCustomTopicResult, AzureContentFilterDetectionResult,
    AzureContentFilterError, AzureContentFilterPersonallyIdentifiableInformationResult,
    AzureContentFilterProtectedMaterialCodeResult, AzureContentFilterSeverityResult,
};

// region: --- Response structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Response {
    pub choices: Vec<ResponseChoice>,
    pub created: u64,
    pub id: String,
    pub model: String,
    pub object: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    pub usage: ResponseUsage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_filter_results: Option<Vec<AzureContentFilterResultForPrompt>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoice {
    pub finish_reason: String,
    pub index: u64,
    pub message: ResponseChoiceMessage,
    pub logprobs: Option<ResponseChoiceLogprob>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_filter_results: Option<AzureContentFilterResultForChoice>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoiceMessage {
    pub content: Option<String>,
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
    pub function_call: Option<ResponseFunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
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
#[serde(tag = "type")]
pub enum ResponseChoiceToolCall {
    #[serde(rename = "function", alias = "function")]
    Function {
        id: String,
        function: ResponseChoiceFunctionToolCall,
    },
    #[serde(rename = "custom", alias = "custom")]
    Custom {
        id: String,
        custom: ResponseChoiceCustomToolCall,
    },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoiceFunctionToolCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseChoiceCustomToolCall {
    pub name: String,
    pub input: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterResultForPrompt {
    pub prompt_index: i32,
    pub content_filter_results: AzurePromptContentFilterResults,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzurePromptContentFilterResults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sexual: Option<AzureContentFilterSeverityResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hate: Option<AzureContentFilterSeverityResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub violence: Option<AzureContentFilterSeverityResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_harm: Option<AzureContentFilterSeverityResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profanity: Option<AzureContentFilterDetectionResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_blocklists: Option<AzureContentFilterBlocklistResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_topics: Option<AzureContentFilterCustomTopicResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AzureContentFilterError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jailbreak: Option<AzureContentFilterDetectionResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indirect_attack: Option<AzureContentFilterDetectionResult>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterResultForChoice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sexual: Option<AzureContentFilterSeverityResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hate: Option<AzureContentFilterSeverityResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub violence: Option<AzureContentFilterSeverityResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_harm: Option<AzureContentFilterSeverityResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profanity: Option<AzureContentFilterDetectionResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_blocklists: Option<AzureContentFilterBlocklistResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_topics: Option<AzureContentFilterCustomTopicResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AzureContentFilterError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protected_material_text: Option<AzureContentFilterDetectionResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protected_material_code: Option<AzureContentFilterProtectedMaterialCodeResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ungrounded_material: Option<AzureContentFilterCompletionTextSpanDetectionResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personally_identifiable_information:
        Option<AzureContentFilterPersonallyIdentifiableInformationResult>,
}
// endregion: --- Response structs

// region: --- Transform methods
pub mod to_openai_transform {
    use super::*;
    use crate::providers::openai::chat_completions::response as openai;
    use crate::providers::{
        Transformation, TransformationContext, TransformationLoss, Transformer,
    };

    #[derive(Debug)]
    pub struct Loss {}

    #[derive(Debug)]
    pub struct Context {
        pub model: Option<String>,
    }

    impl TransformationContext<Response, openai::Response> for Context {}
    impl TransformationLoss<Response, openai::Response> for Loss {}

    impl Transformer<openai::Response, Context, Loss> for Response {
        fn transform(self, context: Context) -> Transformation<openai::Response, Loss> {
            Transformation {
                result: openai::Response {
                    id: self.id,
                    choices: self
                        .choices
                        .into_iter()
                        .map(transform_response_choice)
                        .collect(),
                    created: self.created,
                    model: context.model.unwrap_or(self.model),
                    system_fingerprint: self.system_fingerprint,
                    object: self.object,
                    usage: transform_usage(self.usage),
                    service_tier: None,
                },
                loss: Loss {},
            }
        }
    }

    fn transform_usage(usage: ResponseUsage) -> openai::ResponseUsage {
        openai::ResponseUsage {
            completion_tokens: usage.completion_tokens,
            prompt_tokens: usage.prompt_tokens,
            total_tokens: usage.total_tokens,
            completion_tokens_details: usage.completion_tokens_details.map(|details| {
                openai::CompletionTokensDetails {
                    accepted_prediction_tokens: details.accepted_prediction_tokens,
                    audio_tokens: details.audio_tokens,
                    reasoning_tokens: details.reasoning_tokens,
                    rejected_prediction_tokens: details.rejected_prediction_tokens,
                }
            }),
            prompt_tokens_details: usage.prompt_tokens_details.map(|details| {
                openai::PromptTokensDetails {
                    audio_tokens: details.audio_tokens,
                    cached_tokens: details.cached_tokens,
                }
            }),
        }
    }

    fn transform_response_choice(choice: ResponseChoice) -> openai::ResponseChoice {
        openai::ResponseChoice {
            finish_reason: choice.finish_reason,
            index: choice.index,
            message: transform_response_choice_message(choice.message),
            logprobs: choice.logprobs.map(transform_logprobs),
        }
    }

    fn transform_response_choice_message(
        message: ResponseChoiceMessage,
    ) -> openai::ResponseChoiceMessage {
        openai::ResponseChoiceMessage {
            content: message.content,
            refusal: message.refusal,
            role: message.role,
            tool_calls: message.tool_calls.map(|tool_calls| {
                tool_calls
                    .into_iter()
                    .filter_map(transform_response_choice_tool_call)
                    .collect()
            }),
            annotations: message
                .annotations
                .map(|annotations| annotations.into_iter().map(transform_annotation).collect()),
            audio: message.audio.map(transform_audio),
            function_call: message.function_call.map(|call| {
                openai::ResponseChoiceFunctionToolCall {
                    name: call.name,
                    arguments: call.arguments,
                }
            }),
        }
    }

    fn transform_logprobs(logprobs: ResponseChoiceLogprob) -> openai::ResponseChoiceLogprob {
        openai::ResponseChoiceLogprob {
            content: logprobs
                .content
                .map(|items| items.into_iter().map(transform_logprob_item).collect()),
            refusal: logprobs
                .refusal
                .map(|items| items.into_iter().map(transform_logprob_item).collect()),
        }
    }

    fn transform_logprob_item(
        item: ResponseChoiceLogprobContent,
    ) -> openai::ResponseChoiceLogprobContent {
        openai::ResponseChoiceLogprobContent {
            token: item.token,
            logprob: item.logprob,
            bytes: item.bytes,
            top_logprobs: item
                .top_logprobs
                .into_iter()
                .map(|top| openai::ResponseChoiceTopLogprobContent {
                    token: top.token,
                    logprob: top.logprob,
                    bytes: top.bytes,
                })
                .collect(),
        }
    }

    fn transform_response_choice_tool_call(
        tool_call: ResponseChoiceToolCall,
    ) -> Option<openai::ResponseChoiceToolCall> {
        match tool_call {
            ResponseChoiceToolCall::Function { id, function } => {
                Some(openai::ResponseChoiceToolCall::Function {
                    id,
                    function: openai::ResponseChoiceFunctionToolCall {
                        name: function.name,
                        arguments: function.arguments,
                    },
                })
            }
            ResponseChoiceToolCall::Custom { .. } => None,
        }
    }

    fn transform_annotation(
        annotation: ResponseMessageAnnotation,
    ) -> openai::ResponseMessageAnnotation {
        match annotation {
            ResponseMessageAnnotation::UrlCitation { url_citation } => {
                openai::ResponseMessageAnnotation::UrlCitation {
                    url_citation: openai::ResponseUrlCitation {
                        start_index: url_citation.start_index,
                        end_index: url_citation.end_index,
                        url: url_citation.url,
                        title: url_citation.title,
                    },
                }
            }
        }
    }

    fn transform_audio(audio: ResponseMessageAudio) -> openai::ResponseMessageAudio {
        openai::ResponseMessageAudio {
            id: audio.id,
            expires_at: audio.expires_at,
            data: audio.data,
            transcript: audio.transcript,
        }
    }
}
// endregion: --- Transform methods

#[cfg(test)]
mod tests {
    use super::Response;

    #[test]
    fn response_with_prompt_filters_and_custom_tool_parses() {
        let json = r#"{
            "id": "chatcmpl-1",
            "object": "chat.completion",
            "created": 42,
            "model": "gpt-4o",
            "choices": [{
                "finish_reason": "stop",
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "hi",
                    "tool_calls": [{
                        "type": "custom",
                        "id": "call_1",
                        "custom": { "name": "validator", "input": "ok" }
                    }],
                    "reasoning_content": "trace"
                },
                "logprobs": null
            }],
            "usage": {
                "prompt_tokens": 1,
                "completion_tokens": 1,
                "total_tokens": 2
            },
            "prompt_filter_results": [{
                "prompt_index": 0,
                "content_filter_results": {
                    "jailbreak": { "filtered": false, "detected": false },
                    "indirect_attack": { "filtered": false, "detected": false }
                }
            }]
        }"#;

        let response: Response = serde_json::from_str(json).expect("parse response");
        assert_eq!(response.choices.len(), 1);
        assert!(response.prompt_filter_results.is_some());
    }
}
