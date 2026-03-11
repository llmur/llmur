use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::providers::openai::batches::types::{BatchEndpoint, CompletionWindow};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CreateBatchRequest {
    pub input_file_id: String,
    pub endpoint: BatchEndpoint,
    pub completion_window: CompletionWindow,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ListBatchesQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}
