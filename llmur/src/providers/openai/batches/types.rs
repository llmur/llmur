use crate::providers::ExposesUsage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BatchObjectType {
    Batch,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum BatchEndpoint {
    #[serde(rename = "/v1/responses")]
    Responses,
    #[serde(rename = "/v1/chat/completions")]
    ChatCompletions,
    #[serde(rename = "/v1/embeddings")]
    Embeddings,
    #[serde(rename = "/v1/completions")]
    Completions,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum CompletionWindow {
    #[serde(rename = "24h")]
    Hours24,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchStatus {
    Validating,
    Failed,
    InProgress,
    Finalizing,
    Completed,
    Expired,
    Cancelling,
    Cancelled,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BatchError {
    pub code: String,
    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BatchErrors {
    pub object: BatchErrorsObject,
    pub data: Vec<BatchError>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BatchErrorsObject {
    List,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BatchRequestCounts {
    pub total: u64,
    pub completed: u64,
    pub failed: u64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BatchObject {
    pub id: String,
    pub object: BatchObjectType,
    pub endpoint: BatchEndpoint,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<BatchErrors>,

    pub input_file_id: String,
    pub completion_window: CompletionWindow,
    pub status: BatchStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_file_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_file_id: Option<String>,

    pub created_at: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_progress_at: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub finalizing_at: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_at: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expired_at: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelling_at: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled_at: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_counts: Option<BatchRequestCounts>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

impl ExposesUsage for BatchObject {
    fn get_input_tokens(&self) -> u64 {
        0
    }

    fn get_output_tokens(&self) -> u64 {
        0
    }
}
