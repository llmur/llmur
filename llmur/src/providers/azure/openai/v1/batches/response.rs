pub use crate::providers::openai::batches::response::{ListBatchesResponse, ListObject};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_batches_response_roundtrips() {
        let response = ListBatchesResponse {
            object: ListObject::List,
            data: Vec::new(),
            first_id: None,
            last_id: None,
            has_more: false,
        };

        let value = serde_json::to_value(&response).unwrap();
        let decoded: ListBatchesResponse = serde_json::from_value(value).unwrap();
        assert!(!decoded.has_more);
        assert!(decoded.data.is_empty());
    }
}
