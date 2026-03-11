use serde::{Deserialize, Serialize};

use crate::errors::LLMurError;
use crate::providers::openai::files::types::{FileObject, FileObjectType, FilePurpose, FileStatus};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadFileMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileResource {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,
}

pub fn file_resource_to_openai(
    resource: FileResource,
    default_purpose: FilePurpose,
) -> Result<FileObject, LLMurError> {
    let bytes = resource
        .size_bytes
        .as_deref()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    let created_at = resource
        .create_time
        .as_deref()
        .and_then(parse_rfc3339_timestamp)
        .unwrap_or_else(|| Utc::now().timestamp().max(0) as u64);
    let filename = resource
        .display_name
        .clone()
        .unwrap_or_else(|| resource.name.clone());

    Ok(FileObject {
        id: resource.name,
        bytes,
        created_at,
        expires_at: None,
        filename,
        object: FileObjectType::File,
        purpose: default_purpose,
        status: FileStatus::Processed,
        status_details: None,
    })
}

fn parse_rfc3339_timestamp(value: &str) -> Option<u64> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.timestamp().max(0) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_resource_to_openai_maps_metadata() {
        let resource = FileResource {
            name: "files/xyz".to_string(),
            display_name: Some("input.jsonl".to_string()),
            size_bytes: Some("123".to_string()),
            create_time: Some("2025-01-01T00:00:00Z".to_string()),
        };

        let file = file_resource_to_openai(resource, FilePurpose::BatchOutput).unwrap();
        assert_eq!(file.id, "files/xyz");
        assert_eq!(file.filename, "input.jsonl");
        assert_eq!(file.bytes, 123);
        assert_eq!(file.purpose, FilePurpose::BatchOutput);
        assert_eq!(file.status, FileStatus::Processed);
    }

    #[test]
    fn file_resource_to_openai_uses_defaults_when_optional_values_missing() {
        let resource = FileResource {
            name: "files/no-meta".to_string(),
            display_name: None,
            size_bytes: None,
            create_time: None,
        };

        let file = file_resource_to_openai(resource, FilePurpose::Batch).unwrap();
        assert_eq!(file.filename, "files/no-meta");
        assert_eq!(file.bytes, 0);
        assert!(file.created_at > 0);
    }
}
