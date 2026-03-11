use serde::{Deserialize, Serialize};

use crate::providers::openai::files::types::FilePurpose;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CreateFileRequest {
    pub file: String,
    pub purpose: FilePurpose,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ListFilesQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<FilePurpose>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<ListFilesOrder>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListFilesOrder {
    Asc,
    Desc,
}
