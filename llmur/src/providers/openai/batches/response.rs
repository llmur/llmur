use crate::providers::ExposesUsage;
use serde::{Deserialize, Serialize};

use crate::providers::openai::batches::types::BatchObject;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ListBatchesResponse {
    pub object: ListObject,
    pub data: Vec<BatchObject>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,

    pub has_more: bool,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListObject {
    List,
}

impl ExposesUsage for ListBatchesResponse {
    fn get_input_tokens(&self) -> u64 {
        0
    }

    fn get_output_tokens(&self) -> u64 {
        0
    }
}
