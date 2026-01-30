use serde::{Deserialize, Serialize};

use crate::providers::ExposesUsage;

use super::request::Content;

/// GenerateContent response payload from Gemini.
///
/// Returns candidate responses plus optional prompt feedback, usage, and model status.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates: Option<Vec<Candidate>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_feedback: Option<PromptFeedback>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_metadata: Option<UsageMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_status: Option<ModelStatus>,
}

/// A candidate response from the model.
///
/// Includes generated content, finish reason, safety ratings, and attribution metadata.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_ratings: Option<Vec<SafetyRating>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_metadata: Option<CitationMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_attributions: Option<Vec<GroundingAttribution>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_metadata: Option<GroundingMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_logprobs: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs_result: Option<LogprobsResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_context_metadata: Option<UrlContextMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_message: Option<String>,
}

/// Feedback about the prompt's safety evaluation.
///
/// If blocked, no candidates are returned.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptFeedback {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_reason: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_ratings: Option<Vec<SafetyRating>>,
}

/// Safety rating for a piece of content.
///
/// Contains a harm category, probability, and whether it was blocked.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyRating {
    pub category: String,
    pub probability: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked: Option<bool>,
}

/// Token usage metadata for a response.
///
/// Reports prompt, cached, and candidate token counts plus modality breakdowns.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_token_count: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_content_token_count: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates_token_count: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_prompt_token_count: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thoughts_token_count: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_token_count: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<Vec<ModalityTokenCount>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_tokens_details: Option<Vec<ModalityTokenCount>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates_tokens_details: Option<Vec<ModalityTokenCount>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_prompt_tokens_details: Option<Vec<ModalityTokenCount>>,
}

/// Token counts for a specific modality.
///
/// Used for prompt, cache, candidate, or tool-use token breakdowns.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalityTokenCount {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<u64>,
}

/// Model status information returned by Gemini.
///
/// Indicates model stage and retirement time when applicable.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_stage: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub retirement_time: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Citation metadata for a response.
///
/// Collection of source attributions for model output.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_sources: Option<Vec<CitationSource>>,
}

/// Citation source reference.
///
/// Byte offsets reference the cited segment within the response.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

/// Attribution entry for grounded content.
///
/// Links a grounded source id to the content it supports.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingAttribution {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<AttributionSourceId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,
}

/// Source identifier for grounding attribution.
///
/// Union of inline grounding passage or semantic retriever chunk.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributionSourceId {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_passage: Option<GroundingPassageId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_retriever_chunk: Option<SemanticRetrieverChunk>,
}

/// Identifier for a grounding passage.
///
/// References a passage id and part index from the grounding context.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingPassageId {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passage_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_index: Option<u64>,
}

/// Identifier for a semantic retriever chunk.
///
/// References a retriever source and chunk identifier.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticRetrieverChunk {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk: Option<String>,
}

/// Grounding metadata for a candidate.
///
/// Contains grounding chunks, supports, search entry points, and retrieval metadata.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_chunks: Option<Vec<GroundingChunk>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_supports: Option<Vec<GroundingSupport>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_queries: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_entry_point: Option<SearchEntryPoint>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_metadata: Option<RetrievalMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_maps_widget_context_token: Option<String>,
}

/// Search entry point metadata for web grounding.
///
/// Provides rendered HTML and a base64-encoded SDK blob.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchEntryPoint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendered_content: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdk_blob: Option<String>,
}

/// Grounding chunk with a concrete source.
///
/// Union of web, retrieved-context, or maps grounding.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingChunk {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web: Option<Web>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieved_context: Option<RetrievedContext>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub maps: Option<Maps>,
}

/// Web grounding chunk.
///
/// URI and title for web-sourced grounding.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Web {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// File retrieval grounding chunk.
///
/// File search tool retrieval with document metadata.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrievedContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_search_store: Option<String>,
}

/// Google Maps grounding chunk.
///
/// Place details plus answer sources for Maps grounding.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Maps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_answer_sources: Option<PlaceAnswerSources>,
}

/// Sources used for place answers in Maps grounding.
///
/// Currently backed by review snippets.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceAnswerSources {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_snippets: Option<Vec<ReviewSnippet>>,
}

/// Review snippet used by Maps grounding.
///
/// Provides review id, link, and title.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSnippet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_maps_uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Grounding support references for a segment.
///
/// Associates response segments with supporting grounding chunks.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingSupport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_chunk_indices: Option<Vec<u64>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_scores: Option<Vec<f64>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment: Option<Segment>,
}

/// Segment of content referenced by grounding support.
///
/// Byte offsets reference the content segment within the candidate response.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_index: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Retrieval metadata for grounding.
///
/// Includes dynamic retrieval scoring for Google search grounding.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search_dynamic_retrieval_score: Option<f64>,
}

/// Log probability results for generated tokens.
///
/// Provides top candidates per step and the chosen token sequence.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogprobsResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_candidates: Option<Vec<TopCandidates>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub chosen_candidates: Option<Vec<LogprobsCandidate>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_probability_sum: Option<f64>,
}

/// Top logprob candidates per decoding step.
///
/// Sorted by log probability descending.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopCandidates {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates: Option<Vec<LogprobsCandidate>>,
}

/// Logprob data for a single token.
///
/// Contains token text, token id, and log probability.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogprobsCandidate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_id: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_probability: Option<f64>,
}

/// URL context metadata from retrieval tooling.
///
/// Lists URLs retrieved by the URL context tool.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlContextMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_metadata: Option<Vec<UrlMetadata>>,
}

/// Metadata for a single retrieved URL.
///
/// Includes retrieval status for the URL.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieved_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_retrieval_status: Option<String>,
}

impl ExposesUsage for Response {
    fn get_input_tokens(&self) -> u64 {
        self.usage_metadata
            .as_ref()
            .and_then(|usage| usage.prompt_token_count)
            .unwrap_or(0)
    }

    fn get_output_tokens(&self) -> u64 {
        self.usage_metadata
            .as_ref()
            .and_then(|usage| usage.candidates_token_count)
            .unwrap_or(0)
    }
}

// region: --- Transform methods
pub mod to_openai_transform {
    use super::*;
    use crate::providers::openai::chat_completions::response::{
        CompletionTokensDetails as OpenAiCompletionTokensDetails,
        PromptTokensDetails as OpenAiPromptTokensDetails, Response as OpenAiResponse,
        ResponseChoice as OpenAiResponseChoice,
        ResponseChoiceFunctionToolCall as OpenAiResponseChoiceFunctionToolCall,
        ResponseChoiceMessage as OpenAiResponseChoiceMessage,
        ResponseChoiceToolCall as OpenAiResponseChoiceToolCall,
        ResponseUsage as OpenAiResponseUsage,
    };
    use crate::providers::{
        Transformation, TransformationContext, TransformationLoss, Transformer,
    };

    #[derive(Debug)]
    pub struct Loss {}

    #[derive(Debug)]
    pub struct Context {
        pub model: Option<String>,
    }

    impl TransformationContext<Response, OpenAiResponse> for Context {}
    impl TransformationLoss<Response, OpenAiResponse> for Loss {}

    impl Transformer<OpenAiResponse, Context, Loss> for Response {
        fn transform(self, context: Context) -> Transformation<OpenAiResponse, Loss> {
            let candidates = self.candidates.unwrap_or_default();
            let model = context
                .model
                .or(self.model_version)
                .unwrap_or_else(|| "gemini".to_string());

            Transformation {
                result: OpenAiResponse {
                    id: self.response_id.unwrap_or_else(|| "gemini".to_string()),
                    choices: candidates
                        .into_iter()
                        .enumerate()
                        .map(|(idx, candidate)| transform_response_choice(candidate, idx as u64))
                        .collect(),
                    created: 0,
                    model,
                    system_fingerprint: None,
                    object: "chat.completion".to_string(),
                    usage: transform_response_usage(self.usage_metadata),
                    service_tier: None,
                },
                loss: Loss {},
            }
        }
    }

    fn transform_response_usage(usage: Option<UsageMetadata>) -> OpenAiResponseUsage {
        if let Some(usage) = usage {
            let prompt_tokens = usage.prompt_token_count.unwrap_or(0);
            let completion_tokens = usage.candidates_token_count.unwrap_or(0);
            let total_tokens = usage
                .total_token_count
                .unwrap_or(prompt_tokens + completion_tokens);
            let completion_tokens_details =
                usage
                    .thoughts_token_count
                    .map(|value| OpenAiCompletionTokensDetails {
                        accepted_prediction_tokens: None,
                        audio_tokens: None,
                        reasoning_tokens: Some(value),
                        rejected_prediction_tokens: None,
                    });
            let prompt_tokens_details =
                usage
                    .cached_content_token_count
                    .map(|value| OpenAiPromptTokensDetails {
                        audio_tokens: None,
                        cached_tokens: Some(value),
                    });

            OpenAiResponseUsage {
                completion_tokens,
                prompt_tokens,
                total_tokens,
                completion_tokens_details,
                prompt_tokens_details,
            }
        } else {
            OpenAiResponseUsage {
                completion_tokens: 0,
                prompt_tokens: 0,
                total_tokens: 0,
                completion_tokens_details: None,
                prompt_tokens_details: None,
            }
        }
    }

    fn transform_response_choice(
        candidate: Candidate,
        candidate_index: u64,
    ) -> OpenAiResponseChoice {
        let (content, tool_calls) = transform_content(candidate.content, candidate_index);
        OpenAiResponseChoice {
            finish_reason: transform_finish_reason(candidate.finish_reason),
            index: candidate.index.unwrap_or(candidate_index),
            message: OpenAiResponseChoiceMessage {
                content,
                role: "assistant".to_string(),
                tool_calls,
                refusal: None,
                annotations: None,
                audio: None,
                function_call: None,
            },
            logprobs: None,
        }
    }

    fn transform_content(
        content: Option<Content>,
        candidate_index: u64,
    ) -> (Option<String>, Option<Vec<OpenAiResponseChoiceToolCall>>) {
        let content = match content {
            Some(content) => content,
            None => return (None, None),
        };

        let mut text_parts: Vec<String> = Vec::new();
        let mut tool_calls: Vec<OpenAiResponseChoiceToolCall> = Vec::new();

        for (idx, part) in content.parts.iter().enumerate() {
            if let Some(text) = &part.text {
                text_parts.push(text.clone());
            }
            if let Some(function_call) = &part.function_call {
                let arguments =
                    serde_json::to_string(&function_call.args).unwrap_or_else(|_| "{}".to_string());
                tool_calls.push(OpenAiResponseChoiceToolCall::Function {
                    id: format!("gemini-call-{}-{}", candidate_index, idx),
                    function: OpenAiResponseChoiceFunctionToolCall {
                        name: function_call.name.clone(),
                        arguments,
                    },
                });
            }
        }

        let content = if text_parts.is_empty() {
            None
        } else {
            Some(text_parts.join(""))
        };
        let tool_calls = if tool_calls.is_empty() {
            None
        } else {
            Some(tool_calls)
        };

        (content, tool_calls)
    }

    fn transform_finish_reason(reason: Option<String>) -> String {
        match reason.as_deref().map(|value| value.to_ascii_uppercase()) {
            Some(value) if value == "STOP" => "stop".to_string(),
            Some(value) if value == "MAX_TOKENS" => "length".to_string(),
            Some(value) if value == "SAFETY" => "content_filter".to_string(),
            Some(value) if value == "RECITATION" => "content_filter".to_string(),
            Some(value) => value.to_ascii_lowercase(),
            None => "stop".to_string(),
        }
    }
}

pub mod to_openai_responses_transform {
    use super::*;
    use crate::providers::openai::responses::response::{
        Response as OpenAiResponse, ResponseError, ResponseErrorCode, ResponseIncompleteDetails,
        ResponseIncompleteReason, ResponseInputTokensDetails, ResponseObject, ResponseOutputTokensDetails,
        ResponseStatus, ResponseUsage,
    };
    use crate::providers::openai::responses::types::{
        FunctionToolCall, FunctionToolCallType, ItemStatus, OutputContent, OutputItem, OutputMessage,
        OutputMessageRole, Tool, ToolChoice, ToolChoiceMode, Truncation, Reasoning, TextConfig,
        ServiceTier,
    };
    use crate::providers::{Transformation, TransformationContext, TransformationLoss, Transformer};
    use std::collections::HashMap;

    #[derive(Debug)]
    pub struct Loss {}

    #[derive(Debug)]
    pub struct Context {
        pub model: Option<String>,
        pub parallel_tool_calls: Option<bool>,
        pub previous_response_id: Option<String>,
        pub reasoning: Option<Reasoning>,
        pub max_output_tokens: Option<u64>,
        pub instructions: Option<String>,
        pub text: Option<TextConfig>,
        pub tools: Option<Vec<Tool>>,
        pub tool_choice: Option<ToolChoice>,
        pub truncation: Option<Truncation>,
        pub metadata: Option<HashMap<String, String>>,
        pub temperature: Option<f64>,
        pub top_p: Option<f64>,
        pub user: Option<String>,
        pub service_tier: Option<ServiceTier>,
    }

    impl TransformationContext<Response, OpenAiResponse> for Context {}
    impl TransformationLoss<Response, OpenAiResponse> for Loss {}

    impl Transformer<OpenAiResponse, Context, Loss> for Response {
        fn transform(self, context: Context) -> Transformation<OpenAiResponse, Loss> {
            let model = context
                .model
                .or(self.model_version)
                .unwrap_or_else(|| "gemini".to_string());
            let response_id = self.response_id.unwrap_or_else(|| "gemini".to_string());

            let candidates = self.candidates.unwrap_or_default();
            let (output, output_text, incomplete_details) = transform_candidates(candidates);
            let error = transform_prompt_feedback_error(&self.prompt_feedback);

            let status = if error.is_some() {
                ResponseStatus::Failed
            } else if incomplete_details.is_some() {
                ResponseStatus::Incomplete
            } else {
                ResponseStatus::Completed
            };

            let tools = context.tools.unwrap_or_default();
            let tool_choice = context
                .tool_choice
                .unwrap_or_else(|| default_tool_choice(&tools));

            Transformation {
                result: OpenAiResponse {
                    id: response_id,
                    object: ResponseObject::Response,
                    created_at: 0,
                    status,
                    error,
                    incomplete_details,
                    output,
                    output_text,
                    usage: self.usage_metadata.map(transform_usage),
                    parallel_tool_calls: context.parallel_tool_calls.unwrap_or(false),
                    previous_response_id: context.previous_response_id,
                    model,
                    reasoning: context.reasoning,
                    max_output_tokens: context.max_output_tokens,
                    instructions: context.instructions,
                    text: context.text,
                    tools,
                    tool_choice,
                    truncation: context.truncation,
                    metadata: context.metadata,
                    temperature: context.temperature,
                    top_p: context.top_p,
                    user: context.user,
                    service_tier: context.service_tier,
                },
                loss: Loss {},
            }
        }
    }

    fn default_tool_choice(tools: &[Tool]) -> ToolChoice {
        if tools.is_empty() {
            ToolChoice::Mode(ToolChoiceMode::None)
        } else {
            ToolChoice::Mode(ToolChoiceMode::Auto)
        }
    }

    fn transform_usage(usage: UsageMetadata) -> ResponseUsage {
        let input_tokens = usage.prompt_token_count.unwrap_or(0);
        let output_tokens = usage.candidates_token_count.unwrap_or(0);
        let total_tokens = usage
            .total_token_count
            .unwrap_or(input_tokens + output_tokens);
        let cached_tokens = usage.cached_content_token_count.unwrap_or(0);
        let reasoning_tokens = usage.thoughts_token_count.unwrap_or(0);

        ResponseUsage {
            input_tokens,
            input_tokens_details: ResponseInputTokensDetails { cached_tokens },
            output_tokens,
            output_tokens_details: ResponseOutputTokensDetails { reasoning_tokens },
            total_tokens,
        }
    }

    fn transform_prompt_feedback_error(
        feedback: &Option<PromptFeedback>,
    ) -> Option<ResponseError> {
        let reason = feedback.as_ref().and_then(|value| value.block_reason.as_ref())?;
        Some(ResponseError {
            code: ResponseErrorCode::InvalidPrompt,
            message: format!("Prompt blocked: {}", reason),
        })
    }

    fn transform_candidates(
        candidates: Vec<Candidate>,
    ) -> (Vec<OutputItem>, Option<String>, Option<ResponseIncompleteDetails>) {
        let mut output_items = Vec::new();
        let mut output_text = None;
        let mut incomplete_reason = None;

        for (idx, candidate) in candidates.into_iter().enumerate() {
            if let Some(reason) = candidate.finish_reason.as_ref() {
                match reason.to_ascii_uppercase().as_str() {
                    "MAX_TOKENS" => {
                        incomplete_reason = Some(ResponseIncompleteReason::MaxOutputTokens);
                    }
                    "SAFETY" | "RECITATION" => {
                        incomplete_reason = Some(ResponseIncompleteReason::ContentFilter);
                    }
                    _ => {}
                }
            }

            if let Some(content) = candidate.content {
                let (text, tool_calls) = transform_content(content, idx as u64);
                if let Some(text) = text {
                    if output_text.is_none() {
                        output_text = Some(text.clone());
                    }
                    output_items.push(OutputItem::OutputMessage(OutputMessage {
                        id: format!("gemini-msg-{}", idx),
                        message_type: None,
                        role: OutputMessageRole::Assistant,
                        content: vec![OutputContent::OutputText {
                            text,
                            annotations: Vec::new(),
                        }],
                        status: ItemStatus::Completed,
                    }));
                }
                for tool_call in tool_calls {
                    output_items.push(OutputItem::FunctionToolCall(tool_call));
                }
            }
        }

        let incomplete_details = incomplete_reason.map(|reason| ResponseIncompleteDetails { reason });

        (output_items, output_text, incomplete_details)
    }

    fn transform_content(
        content: Content,
        candidate_index: u64,
    ) -> (Option<String>, Vec<FunctionToolCall>) {
        let mut text_parts = Vec::new();
        let mut tool_calls = Vec::new();

        for (idx, part) in content.parts.iter().enumerate() {
            if let Some(text) = &part.text {
                text_parts.push(text.clone());
            }
            if let Some(function_call) = &part.function_call {
                let call_id = format!("gemini-call-{}-{}", candidate_index, idx);
                let arguments =
                    serde_json::to_string(&function_call.args).unwrap_or_else(|_| "{}".to_string());
                tool_calls.push(FunctionToolCall {
                    id: Some(call_id.clone()),
                    tool_type: FunctionToolCallType::FunctionCall,
                    call_id,
                    name: function_call.name.clone(),
                    arguments,
                    status: Some(ItemStatus::Completed),
                });
            }
        }

        let content = if text_parts.is_empty() {
            None
        } else {
            Some(text_parts.join(""))
        };

        (content, tool_calls)
    }
}
// endregion: --- Transform methods

#[cfg(test)]
mod tests {
    use super::{Response, to_openai_responses_transform, to_openai_transform};
    use crate::providers::Transformer;
    use crate::providers::openai::chat_completions::response::Response as OpenAiResponse;
    use crate::providers::openai::responses::response::Response as OpenAiResponsesResponse;

    #[test]
    fn response_schema_example_roundtrip() {
        let json = r#"{
            "candidates": [{
                "content": {"role": "model", "parts": [{"text": "Hello"}]},
                "finishReason": "STOP",
                "safetyRatings": [{
                    "category": "HARM_CATEGORY_HARASSMENT",
                    "probability": "LOW",
                    "blocked": false
                }],
                "citationMetadata": {
                    "citationSources": [{
                        "startIndex": 0,
                        "endIndex": 5,
                        "uri": "https://example.com",
                        "license": "MIT"
                    }]
                },
                "tokenCount": 5,
                "groundingAttributions": [{
                    "sourceId": {"groundingPassage": {"passageId": "p1", "partIndex": 0}},
                    "content": {"parts": [{"text": "source"}]}
                }],
                "groundingMetadata": {
                    "groundingChunks": [{"web": {"uri": "https://example.com", "title": "Example"}}],
                    "groundingSupports": [{
                        "groundingChunkIndices": [0],
                        "confidenceScores": [0.9],
                        "segment": {"partIndex": 0, "startIndex": 0, "endIndex": 5, "text": "Hello"}
                    }],
                    "webSearchQueries": ["query"],
                    "searchEntryPoint": {"renderedContent": "<b>snippet</b>", "sdkBlob": "c2VhcmNo"},
                    "retrievalMetadata": {"googleSearchDynamicRetrievalScore": 0.1},
                    "googleMapsWidgetContextToken": "maps-token"
                },
                "avgLogprobs": -0.1,
                "logprobsResult": {
                    "topCandidates": [{
                        "candidates": [{"token": "Hello", "tokenId": 123, "logProbability": -0.1}]
                    }],
                    "chosenCandidates": [{"token": "Hello", "tokenId": 123, "logProbability": -0.1}],
                    "logProbabilitySum": -0.1
                },
                "urlContextMetadata": {
                    "urlMetadata": [{
                        "retrievedUrl": "https://example.com",
                        "urlRetrievalStatus": "URL_RETRIEVAL_STATUS_SUCCESS"
                    }]
                },
                "index": 0,
                "finishMessage": "stop"
            }],
            "promptFeedback": {
                "blockReason": "BLOCK_REASON_UNSPECIFIED",
                "safetyRatings": [{
                    "category": "HARM_CATEGORY_HARASSMENT",
                    "probability": "LOW",
                    "blocked": false
                }]
            },
            "usageMetadata": {
                "promptTokenCount": 3,
                "candidatesTokenCount": 5,
                "totalTokenCount": 8
            },
            "modelVersion": "gemini-2.0-flash",
            "responseId": "resp-1",
            "modelStatus": {
                "modelStage": "STABLE",
                "retirementTime": "2014-10-02T15:01:23Z",
                "message": "ok"
            }
        }"#;

        let response: Response = serde_json::from_str(json).expect("parse response");
        let candidates = response.candidates.as_ref().expect("candidates");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].finish_reason.as_deref(), Some("STOP"));
        assert_eq!(
            response
                .usage_metadata
                .as_ref()
                .and_then(|usage| usage.total_token_count),
            Some(8)
        );

        let value = serde_json::to_value(&response).expect("serialize response");
        assert_eq!(
            value["candidates"][0]["content"]["parts"][0]["text"],
            "Hello"
        );
        assert_eq!(value["usageMetadata"]["totalTokenCount"], 8);
    }

    #[test]
    fn transform_gemini_response_to_openai() {
        let json = r#"{
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [
                        { "text": "Hi" },
                        { "functionCall": { "name": "getWeather", "args": { "city": "Paris" } } }
                    ]
                },
                "finishReason": "STOP",
                "index": 0
            }],
            "usageMetadata": {
                "promptTokenCount": 2,
                "candidatesTokenCount": 3,
                "totalTokenCount": 5,
                "cachedContentTokenCount": 1,
                "thoughtsTokenCount": 1
            },
            "modelVersion": "gemini-2.0",
            "responseId": "resp-1"
        }"#;

        let response: Response = serde_json::from_str(json).expect("parse response");
        let transformed = response.transform(to_openai_transform::Context { model: None });
        let openai_response: OpenAiResponse = transformed.result;

        assert_eq!(openai_response.id, "resp-1");
        assert_eq!(openai_response.model, "gemini-2.0");
        assert_eq!(openai_response.choices.len(), 1);
        assert_eq!(openai_response.choices[0].finish_reason, "stop");
        assert_eq!(
            openai_response.choices[0].message.content.as_deref(),
            Some("Hi")
        );
        assert_eq!(
            openai_response.choices[0]
                .message
                .tool_calls
                .as_ref()
                .map(|calls| calls.len()),
            Some(1)
        );
        assert_eq!(openai_response.usage.prompt_tokens, 2);
        assert_eq!(openai_response.usage.completion_tokens, 3);
        assert_eq!(openai_response.usage.total_tokens, 5);
        assert_eq!(
            openai_response
                .usage
                .prompt_tokens_details
                .as_ref()
                .and_then(|details| details.cached_tokens),
            Some(1)
        );
        assert_eq!(
            openai_response
                .usage
                .completion_tokens_details
                .as_ref()
                .and_then(|details| details.reasoning_tokens),
            Some(1)
        );
    }

    #[test]
    fn transform_gemini_response_to_openai_responses() {
        let json = r#"{
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [
                        { "text": "Hello" },
                        { "functionCall": { "name": "getWeather", "args": { "city": "Paris" } } }
                    ]
                },
                "finishReason": "STOP",
                "index": 0
            }],
            "usageMetadata": {
                "promptTokenCount": 2,
                "candidatesTokenCount": 3,
                "totalTokenCount": 5,
                "cachedContentTokenCount": 1,
                "thoughtsTokenCount": 1
            },
            "modelVersion": "gemini-2.0",
            "responseId": "resp-1"
        }"#;

        let response: Response = serde_json::from_str(json).expect("parse response");
        let transformed = response.transform(to_openai_responses_transform::Context {
            model: Some("gemini-2.0".to_string()),
            parallel_tool_calls: Some(false),
            previous_response_id: None,
            reasoning: None,
            max_output_tokens: None,
            instructions: None,
            text: None,
            tools: None,
            tool_choice: None,
            truncation: None,
            metadata: None,
            temperature: None,
            top_p: None,
            user: None,
            service_tier: None,
        });
        let openai_response: OpenAiResponsesResponse = transformed.result;

        assert_eq!(openai_response.id, "resp-1");
        assert_eq!(openai_response.model, "gemini-2.0");
        assert_eq!(openai_response.output.len(), 2);
        assert_eq!(openai_response.output_text.as_deref(), Some("Hello"));
        assert!(openai_response.usage.is_some());
        let usage = openai_response.usage.expect("usage");
        assert_eq!(usage.input_tokens, 2);
        assert_eq!(usage.output_tokens, 3);
        assert_eq!(usage.total_tokens, 5);
        assert_eq!(usage.input_tokens_details.cached_tokens, 1);
        assert_eq!(usage.output_tokens_details.reasoning_tokens, 1);
    }
}
