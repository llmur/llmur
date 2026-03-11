use serde::{Deserialize, Serialize};

use crate::providers::gemini::v1beta::files::types::UploadFileMetadata;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResumableUploadStartRequest {
    pub file: UploadFileMetadata,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resumable_upload_start_request_serializes_camel_case_fields() {
        let request = ResumableUploadStartRequest {
            file: UploadFileMetadata {
                display_name: Some("batch-input".to_string()),
                mime_type: Some("application/jsonl".to_string()),
            },
        };

        let value = serde_json::to_value(&request).unwrap();
        let file = value.get("file").unwrap();
        assert_eq!(
            file.get("displayName").and_then(|v| v.as_str()),
            Some("batch-input")
        );
        assert_eq!(
            file.get("mimeType").and_then(|v| v.as_str()),
            Some("application/jsonl")
        );
    }
}
