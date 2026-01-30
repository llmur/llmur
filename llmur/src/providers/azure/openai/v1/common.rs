use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum PromptCacheRetention {
    #[serde(rename = "in-memory")]
    InMemory,
    #[serde(rename = "24h")]
    H24,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verbosity {
    Low,
    Medium,
    High,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    None,
    Minimal,
    Low,
    Medium,
    High,
    XHigh,
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AzureContentFilterError {
    pub code: i32,
    pub message: String,
}
