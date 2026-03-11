pub use crate::providers::openai::files::types::{
    FileObject, FileObjectType, FilePurpose, FileStatus,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_object_roundtrips() {
        let file = FileObject {
            id: "file_abc".to_string(),
            bytes: 42,
            created_at: 1_734_000_000,
            expires_at: None,
            filename: "input.jsonl".to_string(),
            object: FileObjectType::File,
            purpose: FilePurpose::Batch,
            status: FileStatus::Processed,
            status_details: None,
        };

        let value = serde_json::to_value(&file).unwrap();
        let decoded: FileObject = serde_json::from_value(value).unwrap();
        assert_eq!(decoded.id, "file_abc");
        assert_eq!(decoded.filename, "input.jsonl");
        assert_eq!(decoded.purpose, FilePurpose::Batch);
        assert_eq!(decoded.status, FileStatus::Processed);
    }
}
