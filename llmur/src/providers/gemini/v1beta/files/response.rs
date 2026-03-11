use serde::{Deserialize, Serialize};

use crate::providers::ExposesUsage;
use crate::providers::gemini::v1beta::files::types::FileResource;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadResponse {
    pub file: FileResource,
}

impl ExposesUsage for FileUploadResponse {
    fn get_input_tokens(&self) -> u64 {
        0
    }

    fn get_output_tokens(&self) -> u64 {
        0
    }
}

impl ExposesUsage for FileResource {
    fn get_input_tokens(&self) -> u64 {
        0
    }

    fn get_output_tokens(&self) -> u64 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_upload_response_roundtrips_and_has_zero_usage() {
        let response = FileUploadResponse {
            file: FileResource {
                name: "files/abc".to_string(),
                display_name: Some("batch.jsonl".to_string()),
                size_bytes: Some("12".to_string()),
                create_time: Some("2025-01-01T00:00:00Z".to_string()),
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let decoded: FileUploadResponse = serde_json::from_value(value).unwrap();
        assert_eq!(decoded.file.name, "files/abc");
        assert_eq!(decoded.get_input_tokens(), 0);
        assert_eq!(decoded.get_output_tokens(), 0);
    }
}
