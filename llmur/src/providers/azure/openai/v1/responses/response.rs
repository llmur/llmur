use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::providers::azure::openai::v1::responses::types::{
    ConversationReference, InputItem, OutputItem, Prompt, PromptCacheRetention, Reasoning,
    TextConfig, Tool, ToolChoice, Truncation,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub object: ResponseObject,
    pub created_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ResponseStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<u64>,
    pub error: Option<ResponseError>,
    pub incomplete_details: Option<ResponseIncompleteDetails>,
    pub output: Vec<OutputItem>,
    pub instructions: Option<ResponseInstructions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ResponseUsage>,
    pub parallel_tool_calls: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<Prompt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<Truncation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<PromptCacheRetention>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<ConversationReference>,
    pub content_filters: Vec<AzureContentFilterForResponsesAPI>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseObject {
    Response,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Completed,
    Failed,
    InProgress,
    Cancelled,
    Queued,
    Incomplete,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseInstructions {
    Text(String),
    Items(Vec<InputItem>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseIncompleteDetails {
    pub reason: ResponseIncompleteReason,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ResponseIncompleteReason {
    #[serde(rename = "max_output_tokens", alias = "max_tokens")]
    MaxOutputTokens,
    #[serde(rename = "content_filter")]
    ContentFilter,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseUsage {
    pub input_tokens: u64,
    pub input_tokens_details: ResponseUsageInputTokensDetails,
    pub output_tokens: u64,
    pub output_tokens_details: ResponseUsageOutputTokensDetails,
    pub total_tokens: u64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseUsageInputTokensDetails {
    pub cached_tokens: u64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseUsageOutputTokensDetails {
    pub reasoning_tokens: u64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: ResponseErrorCode,
    pub message: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseErrorCode {
    ServerError,
    RateLimitExceeded,
    InvalidPrompt,
    VectorStoreTimeout,
    InvalidImage,
    InvalidImageFormat,
    InvalidBase64Image,
    InvalidImageUrl,
    ImageTooLarge,
    ImageTooSmall,
    ImageParseError,
    ImageContentPolicyViolation,
    InvalidImageMode,
    ImageFileTooLarge,
    UnsupportedImageMediaType,
    EmptyImageFile,
    FailedToDownloadImage,
    ImageFileNotFound,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterForResponsesAPI {
    pub blocked: bool,
    pub source_type: String,
    pub content_filter_results: AzureContentFilterResultsForResponsesAPI,
    pub content_filter_offsets: AzureContentFilterResultOffsets,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterResultOffsets {
    pub start_offset: i32,
    pub end_offset: i32,
    pub check_offset: i32,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterResultsForResponsesAPI {
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
    pub jailbreak: AzureContentFilterDetectionResult,
    pub task_adherence: AzureContentFilterDetectionResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protected_material_text: Option<AzureContentFilterDetectionResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protected_material_code: Option<AzureContentFilterProtectedMaterialCodeResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ungrounded_material: Option<AzureContentFilterCompletionTextSpanDetectionResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personally_identifiable_information:
        Option<AzureContentFilterPersonallyIdentifiableInformationResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indirect_attack: Option<AzureContentFilterDetectionResult>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterSeverityResult {
    pub filtered: bool,
    pub severity: AzureContentFilterSeverity,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AzureContentFilterSeverity {
    Safe,
    Low,
    Medium,
    High,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterDetectionResult {
    pub filtered: bool,
    pub detected: bool,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterBlocklistResult {
    pub filtered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<AzureContentFilterBlocklistIdResult>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterBlocklistIdResult {
    pub id: String,
    pub filtered: bool,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterCustomTopicResult {
    pub filtered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<AzureContentFilterCustomTopicIdResult>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterCustomTopicIdResult {
    pub id: String,
    pub detected: bool,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterCompletionTextSpanDetectionResult {
    pub filtered: bool,
    pub detected: bool,
    pub details: Vec<AzureContentFilterCompletionTextSpan>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterCompletionTextSpan {
    pub completion_start_offset: i32,
    pub completion_end_offset: i32,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterProtectedMaterialCodeResult {
    pub filtered: bool,
    pub detected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<AzureContentFilterCodeCitation>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterCodeCitation {
    #[serde(rename = "URL", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterPersonallyIdentifiableInformationResult {
    pub filtered: bool,
    pub detected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacted_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_categories: Option<Vec<AzurePiiSubCategoryResult>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzurePiiSubCategoryResult {
    pub sub_category: String,
    pub filtered: bool,
    pub detected: bool,
    pub redacted: bool,
}

#[cfg(test)]
mod tests {
    use super::Response;

    #[test]
    fn response_deserializes_with_required_fields() {
        let json = r#"{
            "id": "resp_123",
            "object": "response",
            "created_at": 1741487325,
            "status": "completed",
            "error": null,
            "incomplete_details": null,
            "output": [],
            "instructions": null,
            "parallel_tool_calls": true,
            "content_filters": []
        }"#;

        let response: Response = serde_json::from_str(json).expect("parse response");
        assert_eq!(response.id, "resp_123");
        assert_eq!(response.content_filters.len(), 0);
    }
}
