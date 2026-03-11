pub use crate::providers::openai::batches::request::{CreateBatchRequest, ListBatchesQuery};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::azure::openai::v1::batches::types::{BatchEndpoint, CompletionWindow};

    #[test]
    fn create_batch_request_roundtrips() {
        let request = CreateBatchRequest {
            input_file_id: "file_123".to_string(),
            endpoint: BatchEndpoint::ChatCompletions,
            completion_window: CompletionWindow::Hours24,
            metadata: None,
        };

        let value = serde_json::to_value(&request).unwrap();
        let decoded: CreateBatchRequest = serde_json::from_value(value).unwrap();
        assert_eq!(decoded.input_file_id, "file_123");
        assert_eq!(decoded.endpoint, BatchEndpoint::ChatCompletions);
        assert_eq!(decoded.completion_window, CompletionWindow::Hours24);
    }

    #[test]
    fn list_batches_query_roundtrips() {
        let query = ListBatchesQuery {
            after: Some("batch_001".to_string()),
            limit: Some(20),
        };

        let value = serde_json::to_value(&query).unwrap();
        let decoded: ListBatchesQuery = serde_json::from_value(value).unwrap();
        assert_eq!(decoded.after.as_deref(), Some("batch_001"));
        assert_eq!(decoded.limit, Some(20));
    }
}
