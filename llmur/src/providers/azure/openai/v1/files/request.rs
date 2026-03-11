pub use crate::providers::openai::files::request::{
    CreateFileRequest, ListFilesOrder, ListFilesQuery,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::azure::openai::v1::files::types::FilePurpose;

    #[test]
    fn create_file_request_serializes_expected_shape() {
        let request = CreateFileRequest {
            file: "file-contents".to_string(),
            purpose: FilePurpose::Batch,
        };

        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(
            value.get("file").and_then(|v| v.as_str()),
            Some("file-contents")
        );
        assert_eq!(value.get("purpose").and_then(|v| v.as_str()), Some("batch"));
    }

    #[test]
    fn list_files_query_roundtrips() {
        let query = ListFilesQuery {
            purpose: Some(FilePurpose::BatchOutput),
            limit: Some(25),
            order: Some(ListFilesOrder::Desc),
            after: Some("file_123".to_string()),
        };

        let value = serde_json::to_value(&query).unwrap();
        let decoded: ListFilesQuery = serde_json::from_value(value).unwrap();
        assert_eq!(decoded.purpose, Some(FilePurpose::BatchOutput));
        assert_eq!(decoded.limit, Some(25));
        assert_eq!(decoded.order, Some(ListFilesOrder::Desc));
        assert_eq!(decoded.after.as_deref(), Some("file_123"));
    }
}
