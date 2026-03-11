pub use crate::providers::openai::files::response::{
    DeleteFileResponse, ListFilesResponse, ListObject,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::azure::openai::v1::files::types::FileObjectType;

    #[test]
    fn delete_file_response_roundtrips() {
        let response = DeleteFileResponse {
            id: "file_123".to_string(),
            object: FileObjectType::File,
            deleted: true,
        };

        let value = serde_json::to_value(&response).unwrap();
        let decoded: DeleteFileResponse = serde_json::from_value(value).unwrap();
        assert_eq!(decoded.id, "file_123");
        assert!(decoded.deleted);
    }
}
