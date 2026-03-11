use crate::providers::ExposesUsage;
use serde::{Deserialize, Serialize};

use crate::providers::openai::files::types::{FileObject, FileObjectType};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DeleteFileResponse {
    pub id: String,
    pub object: FileObjectType,
    pub deleted: bool,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ListFilesResponse {
    pub object: ListObject,
    pub data: Vec<FileObject>,
    pub first_id: String,
    pub last_id: String,
    pub has_more: bool,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListObject {
    List,
}

impl ExposesUsage for DeleteFileResponse {
    fn get_input_tokens(&self) -> u64 {
        0
    }

    fn get_output_tokens(&self) -> u64 {
        0
    }
}

impl ExposesUsage for ListFilesResponse {
    fn get_input_tokens(&self) -> u64 {
        0
    }

    fn get_output_tokens(&self) -> u64 {
        0
    }
}
