use serde_json::Value as JsonValue;

pub use crate::providers::openai::batches::types::{
    BatchEndpoint, BatchError, BatchErrors, BatchErrorsObject, BatchObject, BatchObjectType,
    BatchRequestCounts, BatchStatus, CompletionWindow,
};

pub fn normalize_batch_response(
    mut value: JsonValue,
    endpoint: &BatchEndpoint,
    completion_window: &CompletionWindow,
) -> Result<BatchObject, serde_json::Error> {
    let endpoint_str = batch_endpoint_as_str(endpoint);
    let completion_window_str = completion_window_as_str(completion_window);

    if let Some(obj) = value.as_object_mut() {
        let endpoint_value = obj
            .get("endpoint")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        if endpoint_value.is_empty() {
            obj.insert(
                "endpoint".to_string(),
                JsonValue::String(endpoint_str.to_string()),
            );
        }

        let completion_value = obj
            .get("completion_window")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        if completion_value.is_empty() {
            obj.insert(
                "completion_window".to_string(),
                JsonValue::String(completion_window_str.to_string()),
            );
        }

        for key in ["output_file_id", "error_file_id"] {
            if let Some(value) = obj.get(key).and_then(|value| value.as_str()) {
                if value.is_empty() {
                    obj.insert(key.to_string(), JsonValue::Null);
                }
            }
        }
    }

    serde_json::from_value::<BatchObject>(value)
}

fn batch_endpoint_as_str(endpoint: &BatchEndpoint) -> &'static str {
    match endpoint {
        BatchEndpoint::Responses => "/v1/responses",
        BatchEndpoint::ChatCompletions => "/v1/chat/completions",
        BatchEndpoint::Embeddings => "/v1/embeddings",
        BatchEndpoint::Completions => "/v1/completions",
    }
}

fn completion_window_as_str(window: &CompletionWindow) -> &'static str {
    match window {
        CompletionWindow::Hours24 => "24h",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_value(raw: &'static str) -> JsonValue {
        serde_json::from_str(raw).expect("fixture JSON should be valid")
    }

    #[test]
    fn normalize_batch_response_fills_empty_fields() {
        let input = serde_json::json!({
            "id": "batch_123",
            "object": "batch",
            "endpoint": "",
            "input_file_id": "file_123",
            "completion_window": "",
            "status": "in_progress",
            "output_file_id": "",
            "error_file_id": "",
            "created_at": 1734000000
        });

        let normalized = normalize_batch_response(
            input,
            &BatchEndpoint::ChatCompletions,
            &CompletionWindow::Hours24,
        )
        .unwrap();

        assert_eq!(normalized.endpoint, BatchEndpoint::ChatCompletions);
        assert_eq!(normalized.completion_window, CompletionWindow::Hours24);
        assert_eq!(normalized.output_file_id, None);
        assert_eq!(normalized.error_file_id, None);
    }

    #[test]
    fn normalize_batch_response_preserves_present_values() {
        let input = serde_json::json!({
            "id": "batch_123",
            "object": "batch",
            "endpoint": "/v1/embeddings",
            "input_file_id": "file_123",
            "completion_window": "24h",
            "status": "completed",
            "output_file_id": "file_out",
            "error_file_id": null,
            "created_at": 1734000000
        });

        let normalized = normalize_batch_response(
            input,
            &BatchEndpoint::ChatCompletions,
            &CompletionWindow::Hours24,
        )
        .unwrap();

        assert_eq!(normalized.endpoint, BatchEndpoint::Embeddings);
        assert_eq!(normalized.output_file_id.as_deref(), Some("file_out"));
    }

    #[test]
    fn normalize_batch_response_from_fixture_fills_missing_and_ignores_extras() {
        let input = fixture_value(include_str!("fixtures/azure_batch_partial_counts.json"));

        let normalized = normalize_batch_response(
            input,
            &BatchEndpoint::ChatCompletions,
            &CompletionWindow::Hours24,
        )
        .unwrap();

        assert_eq!(normalized.id, "batch_fixture_az_1");
        assert_eq!(normalized.endpoint, BatchEndpoint::ChatCompletions);
        assert_eq!(normalized.completion_window, CompletionWindow::Hours24);
        assert_eq!(normalized.output_file_id, None);
        assert_eq!(normalized.error_file_id, None);
        assert_eq!(normalized.request_counts.as_ref().map(|v| v.total), Some(3));
    }

    #[test]
    fn normalize_batch_response_from_fixture_preserves_existing_values() {
        let input = fixture_value(include_str!("fixtures/azure_batch_preserve_values.json"));

        let normalized = normalize_batch_response(
            input,
            &BatchEndpoint::ChatCompletions,
            &CompletionWindow::Hours24,
        )
        .unwrap();

        assert_eq!(normalized.id, "batch_fixture_az_2");
        assert_eq!(normalized.endpoint, BatchEndpoint::Embeddings);
        assert_eq!(normalized.completion_window, CompletionWindow::Hours24);
        assert_eq!(
            normalized.output_file_id.as_deref(),
            Some("file_fixture_az_output_2")
        );
    }
}
