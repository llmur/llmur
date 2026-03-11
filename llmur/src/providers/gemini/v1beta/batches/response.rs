use serde::{Deserialize, Serialize};

use crate::errors::LLMurError;
use crate::providers::ExposesUsage;
use crate::providers::openai::batches::types::{
    BatchEndpoint, BatchError, BatchErrors, BatchErrorsObject, BatchObject, BatchObjectType,
    BatchRequestCounts, BatchStatus, CompletionWindow,
};
use chrono::Utc;

const GEMINI_MAX_FILE_ID_LEN: usize = 128;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchResponse {
    #[serde(flatten)]
    pub value: serde_json::Value,
}

impl ExposesUsage for BatchResponse {
    fn get_input_tokens(&self) -> u64 {
        0
    }

    fn get_output_tokens(&self) -> u64 {
        0
    }
}

pub fn batch_object_from_value(
    value: serde_json::Value,
    endpoint: &BatchEndpoint,
    completion_window: &CompletionWindow,
    input_file_id: &str,
) -> Result<BatchObject, LLMurError> {
    let id = extract_batch_name(&value)
        .ok_or_else(|| LLMurError::BadRequest("Gemini batch response missing name.".to_string()))?;
    let status = extract_batch_status(&value);
    let created_at = extract_timestamp(&value, "createTime")
        .or_else(|| extract_nested_timestamp(&value, &["metadata", "createTime"]))
        .or_else(|| extract_nested_timestamp(&value, &["response", "createTime"]))
        .unwrap_or_else(|| Utc::now().timestamp() as u64);
    let updated_at = extract_timestamp(&value, "updateTime")
        .or_else(|| extract_nested_timestamp(&value, &["metadata", "updateTime"]))
        .or_else(|| extract_nested_timestamp(&value, &["response", "updateTime"]));

    let mut in_progress_at = None;
    let mut completed_at = None;
    let mut failed_at = None;
    let mut expired_at = None;
    let mut cancelling_at = None;
    let mut cancelled_at = None;
    if let Some(timestamp) = updated_at {
        match status {
            BatchStatus::InProgress => in_progress_at = Some(timestamp),
            BatchStatus::Completed => completed_at = Some(timestamp),
            BatchStatus::Failed => failed_at = Some(timestamp),
            BatchStatus::Expired => expired_at = Some(timestamp),
            BatchStatus::Cancelling => cancelling_at = Some(timestamp),
            BatchStatus::Cancelled => cancelled_at = Some(timestamp),
            BatchStatus::Validating => {}
            BatchStatus::Finalizing => in_progress_at = Some(timestamp),
        }
    }

    let output_file_id = extract_output_file_id(&value, input_file_id);
    let error_file_id = extract_error_file_id(&value);
    let request_counts = extract_request_counts(&value);
    let errors = extract_batch_errors(&value);

    if matches!(status, BatchStatus::Completed) && output_file_id.is_none() {
        tracing::debug!(
            batch_id = %id,
            has_dest = value.get("dest").is_some(),
            has_response = value.get("response").is_some(),
            has_response_responses_file = extract_string_at_path(&value, &["response", "responsesFile"]).is_some()
                || extract_string_at_path(&value, &["response", "responses_file"]).is_some(),
            has_response_dest_file_name = extract_string_at_path(&value, &["response", "dest", "fileName"]).is_some()
                || extract_string_at_path(&value, &["response", "dest", "file_name"]).is_some(),
            has_top_level_responses_file = extract_string_at_path(&value, &["responsesFile"]).is_some()
                || extract_string_at_path(&value, &["responses_file"]).is_some(),
            "Completed Gemini batch response did not expose a usable output file reference."
        );
    }

    Ok(BatchObject {
        id,
        object: BatchObjectType::Batch,
        endpoint: endpoint.clone(),
        errors,
        input_file_id: input_file_id.to_string(),
        completion_window: completion_window.clone(),
        status,
        output_file_id,
        error_file_id,
        created_at,
        in_progress_at,
        expires_at: None,
        finalizing_at: None,
        completed_at,
        failed_at,
        expired_at,
        cancelling_at,
        cancelled_at,
        request_counts,
        metadata: None,
    })
}

fn extract_batch_name(value: &serde_json::Value) -> Option<String> {
    let candidates = [
        extract_string_at_path(value, &["response", "name"]),
        extract_string_at_path(value, &["metadata", "name"]),
        extract_string_at_path(value, &["name"]),
    ];

    for candidate in candidates.iter().flatten() {
        if candidate.starts_with("batches/") {
            return Some((*candidate).to_string());
        }
    }

    candidates
        .iter()
        .flatten()
        .next()
        .map(|value| (*value).to_string())
}

fn extract_batch_status(value: &serde_json::Value) -> BatchStatus {
    if let Some(state) = extract_state(value)
        && let Some(status) = map_gemini_state_to_batch_status(&state)
    {
        return status;
    }

    // Gemini operation-style responses can expose completion via "done" even when
    // a parsable state enum is absent; keep polling resilient in that case.
    if value
        .get("done")
        .and_then(|done| done.as_bool())
        .unwrap_or(false)
    {
        if extract_batch_errors(value).is_some() {
            return BatchStatus::Failed;
        }
        return BatchStatus::Completed;
    }

    BatchStatus::InProgress
}

fn extract_state(value: &serde_json::Value) -> Option<String> {
    let candidates = [
        value.get("state"),
        value.get("metadata").and_then(|meta| meta.get("state")),
        value
            .get("metadata")
            .and_then(|meta| meta.get("batchState")),
        value
            .get("metadata")
            .and_then(|meta| meta.get("batch_state")),
        value
            .get("response")
            .and_then(|response| response.get("state")),
        value
            .get("response")
            .and_then(|response| response.get("metadata"))
            .and_then(|meta| meta.get("state")),
        value
            .get("response")
            .and_then(|response| response.get("metadata"))
            .and_then(|meta| meta.get("batchState")),
        value
            .get("response")
            .and_then(|response| response.get("metadata"))
            .and_then(|meta| meta.get("batch_state")),
    ];
    for candidate in candidates {
        if let Some(state) = extract_state_value(candidate) {
            return Some(state);
        }
    }
    None
}

fn extract_state_value(value: Option<&serde_json::Value>) -> Option<String> {
    let value = value?;
    if let Some(state) = value.as_str() {
        return Some(state.to_string());
    }
    if let Some(state) = value.get("state").and_then(|state| state.as_str()) {
        return Some(state.to_string());
    }
    if let Some(state) = value.get("value").and_then(|state| state.as_str()) {
        return Some(state.to_string());
    }
    value
        .get("name")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}

fn map_gemini_state_to_batch_status(state: &str) -> Option<BatchStatus> {
    let normalized = state.trim().to_ascii_uppercase();
    match normalized.as_str() {
        "JOB_STATE_PENDING" | "PENDING" => Some(BatchStatus::Validating),
        "JOB_STATE_RUNNING" | "RUNNING" => Some(BatchStatus::InProgress),
        "JOB_STATE_SUCCEEDED" | "SUCCEEDED" | "JOB_STATE_COMPLETED" | "COMPLETED" => {
            Some(BatchStatus::Completed)
        }
        "JOB_STATE_FAILED" | "FAILED" => Some(BatchStatus::Failed),
        "JOB_STATE_CANCELLED" | "CANCELLED" => Some(BatchStatus::Cancelled),
        "JOB_STATE_EXPIRED" | "EXPIRED" => Some(BatchStatus::Expired),
        "JOB_STATE_CANCELLING" | "CANCELLING" => Some(BatchStatus::Cancelling),
        "BATCH_STATE_PENDING" => Some(BatchStatus::Validating),
        "BATCH_STATE_RUNNING" => Some(BatchStatus::InProgress),
        "BATCH_STATE_SUCCEEDED" => Some(BatchStatus::Completed),
        "BATCH_STATE_FAILED" => Some(BatchStatus::Failed),
        "BATCH_STATE_CANCELLED" => Some(BatchStatus::Cancelled),
        "BATCH_STATE_EXPIRED" => Some(BatchStatus::Expired),
        "BATCH_STATE_CANCELLING" => Some(BatchStatus::Cancelling),
        "JOB_STATE_UNSPECIFIED" => Some(BatchStatus::InProgress),
        "BATCH_STATE_UNSPECIFIED" => Some(BatchStatus::InProgress),
        _ => None,
    }
}

fn extract_output_file_id(value: &serde_json::Value, input_file_id: &str) -> Option<String> {
    let from_known_paths = extract_first_valid_file_reference(
        value,
        &[
            ("responsesFile", &["responsesFile"]),
            ("responses_file", &["responses_file"]),
            ("outputFile", &["outputFile"]),
            ("output_file", &["output_file"]),
            ("metadata.responsesFile", &["metadata", "responsesFile"]),
            ("metadata.responses_file", &["metadata", "responses_file"]),
            (
                "metadata.output.responsesFile",
                &["metadata", "output", "responsesFile"],
            ),
            (
                "metadata.output.responses_file",
                &["metadata", "output", "responses_file"],
            ),
            ("metadata.outputFile", &["metadata", "outputFile"]),
            ("metadata.output_file", &["metadata", "output_file"]),
            ("dest.fileName", &["dest", "fileName"]),
            ("dest.file_name", &["dest", "file_name"]),
            ("response.responsesFile", &["response", "responsesFile"]),
            ("response.responses_file", &["response", "responses_file"]),
            ("response.outputFile", &["response", "outputFile"]),
            ("response.output_file", &["response", "output_file"]),
            ("response.dest.fileName", &["response", "dest", "fileName"]),
            (
                "response.dest.file_name",
                &["response", "dest", "file_name"],
            ),
            (
                "response.metadata.responsesFile",
                &["response", "metadata", "responsesFile"],
            ),
            (
                "response.metadata.responses_file",
                &["response", "metadata", "responses_file"],
            ),
            (
                "response.metadata.outputFile",
                &["response", "metadata", "outputFile"],
            ),
            (
                "response.metadata.output_file",
                &["response", "metadata", "output_file"],
            ),
            (
                "response.output.fileName",
                &["response", "output", "fileName"],
            ),
            (
                "response.output.file_name",
                &["response", "output", "file_name"],
            ),
            (
                "response.output.responsesFile",
                &["response", "output", "responsesFile"],
            ),
            (
                "response.output.responses_file",
                &["response", "output", "responses_file"],
            ),
        ],
    );
    if from_known_paths.is_some() {
        return from_known_paths;
    }

    extract_output_file_id_fallback(value, input_file_id)
}

fn extract_error_file_id(value: &serde_json::Value) -> Option<String> {
    extract_first_valid_file_reference(
        value,
        &[
            ("errorFile", &["errorFile"]),
            ("error_file", &["error_file"]),
            ("errorsFile", &["errorsFile"]),
            ("errors_file", &["errors_file"]),
            ("metadata.errorFile", &["metadata", "errorFile"]),
            ("metadata.error_file", &["metadata", "error_file"]),
            ("metadata.errorsFile", &["metadata", "errorsFile"]),
            ("metadata.errors_file", &["metadata", "errors_file"]),
            ("errorDest.fileName", &["errorDest", "fileName"]),
            ("errorDest.file_name", &["errorDest", "file_name"]),
            ("error_dest.fileName", &["error_dest", "fileName"]),
            ("error_dest.file_name", &["error_dest", "file_name"]),
            ("response.errorFile", &["response", "errorFile"]),
            ("response.error_file", &["response", "error_file"]),
            ("response.errorsFile", &["response", "errorsFile"]),
            ("response.errors_file", &["response", "errors_file"]),
            (
                "response.errorDest.fileName",
                &["response", "errorDest", "fileName"],
            ),
            (
                "response.errorDest.file_name",
                &["response", "errorDest", "file_name"],
            ),
            (
                "response.error_dest.fileName",
                &["response", "error_dest", "fileName"],
            ),
            (
                "response.error_dest.file_name",
                &["response", "error_dest", "file_name"],
            ),
            (
                "response.metadata.errorFile",
                &["response", "metadata", "errorFile"],
            ),
            (
                "response.metadata.error_file",
                &["response", "metadata", "error_file"],
            ),
            (
                "response.metadata.errorsFile",
                &["response", "metadata", "errorsFile"],
            ),
            (
                "response.metadata.errors_file",
                &["response", "metadata", "errors_file"],
            ),
        ],
    )
}

fn extract_first_valid_file_reference(
    value: &serde_json::Value,
    candidates: &[(&str, &[&str])],
) -> Option<String> {
    for (source_field, path) in candidates {
        let Some(raw_reference) = extract_string_at_path(value, path) else {
            continue;
        };

        match normalize_gemini_file_reference_with_reason(raw_reference) {
            Ok(normalized_reference) => {
                if normalized_reference != raw_reference {
                    tracing::debug!(
                        source_field = *source_field,
                        raw_reference_len = raw_reference.len(),
                        normalized_reference = %normalized_reference,
                        "Normalized Gemini batch file reference."
                    );
                }
                return Some(normalized_reference);
            }
            Err(rejection_reason) => {
                tracing::debug!(
                    source_field = *source_field,
                    raw_reference_len = raw_reference.len(),
                    rejection_reason,
                    "Rejected Gemini batch file reference during extraction."
                );
            }
        }
    }

    None
}

fn extract_output_file_id_fallback(
    value: &serde_json::Value,
    input_file_id: &str,
) -> Option<String> {
    let normalized_input_file_id = normalize_gemini_file_reference(input_file_id);
    let mut refs: Vec<(String, String)> = Vec::new();
    collect_file_references(value, "$".to_string(), &mut refs);

    if refs.is_empty() {
        return None;
    }

    let mut preferred: Vec<String> = Vec::new();
    let mut secondary: Vec<String> = Vec::new();
    let mut tertiary: Vec<String> = Vec::new();

    for (path, reference) in refs {
        if normalized_input_file_id.as_ref() == Some(&reference) {
            continue;
        }

        let normalized_path = path.to_ascii_lowercase();
        if normalized_path.contains("error") {
            continue;
        }
        if normalized_path.contains("input") || normalized_path.contains("src") {
            continue;
        }

        if normalized_path.contains("responses")
            || normalized_path.contains("dest")
            || normalized_path.contains("output")
        {
            preferred.push(reference);
        } else if normalized_path.contains("response") {
            secondary.push(reference);
        } else {
            tertiary.push(reference);
        }
    }

    preferred
        .into_iter()
        .chain(secondary)
        .chain(tertiary)
        .next()
}

fn collect_file_references(
    value: &serde_json::Value,
    path: String,
    refs: &mut Vec<(String, String)>,
) {
    match value {
        serde_json::Value::String(raw_reference) => {
            if let Some(normalized) = normalize_gemini_file_reference(raw_reference) {
                refs.push((path, normalized));
            }
        }
        serde_json::Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_file_references(item, format!("{path}[{index}]"), refs);
            }
        }
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                collect_file_references(nested, format!("{path}.{key}"), refs);
            }
        }
        _ => {}
    }
}

fn extract_string_at_path<'a>(value: &'a serde_json::Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

pub fn normalize_gemini_file_reference(reference: &str) -> Option<String> {
    normalize_gemini_file_reference_with_reason(reference).ok()
}

fn normalize_gemini_file_reference_with_reason(reference: &str) -> Result<String, &'static str> {
    let trimmed = strip_query_and_fragment(reference.trim());
    if trimmed.is_empty() {
        return Err("empty_reference");
    }

    let file_id = if let Some(value) = trimmed.strip_prefix("files/") {
        value
    } else if let Some(index) = trimmed.find("/files/") {
        &trimmed[index + "/files/".len()..]
    } else {
        return Err("missing_files_prefix");
    };
    let file_id = strip_download_suffix(file_id);

    if file_id.is_empty() {
        return Err("missing_file_id");
    }
    if file_id.contains('/') {
        return Err("nested_path_not_supported");
    }
    if file_id.len() > GEMINI_MAX_FILE_ID_LEN {
        return Err("file_id_too_long");
    }
    if !file_id
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || char == '-' || char == '_')
    {
        return Err("invalid_file_id_characters");
    }

    Ok(format!("files/{}", file_id))
}

fn strip_query_and_fragment(reference: &str) -> &str {
    reference.split(['?', '#']).next().unwrap_or(reference)
}

fn strip_download_suffix(file_id: &str) -> &str {
    file_id.strip_suffix(":download").unwrap_or(file_id)
}

fn extract_request_counts(value: &serde_json::Value) -> Option<BatchRequestCounts> {
    let stats = value
        .get("batchStats")
        .or_else(|| value.get("batch_stats"))
        .or_else(|| {
            value
                .get("metadata")
                .and_then(|meta| meta.get("batchStats"))
        })
        .or_else(|| {
            value
                .get("metadata")
                .and_then(|meta| meta.get("batch_stats"))
        })
        .or_else(|| {
            value
                .get("response")
                .and_then(|resp| resp.get("batchStats"))
        })
        .or_else(|| {
            value
                .get("response")
                .and_then(|resp| resp.get("batch_stats"))
        })?;

    let total = extract_u64(stats, &["totalRequestCount", "requestCount", "total"])?;
    let completed = extract_u64(
        stats,
        &[
            "completedRequestCount",
            "succeededRequestCount",
            "successfulRequestCount",
            "successRequestCount",
            "completedCount",
            "succeededCount",
            "successfulCount",
            "processedRequestCount",
            "completed",
            "succeeded",
            "successful",
        ],
    )
    .unwrap_or(0);
    let failed = extract_u64(
        stats,
        &[
            "failedRequestCount",
            "failedCount",
            "errorRequestCount",
            "errorCount",
            "failed",
            "errors",
        ],
    )
    .unwrap_or(0);

    Some(BatchRequestCounts {
        total,
        completed,
        failed,
    })
}

fn extract_batch_errors(value: &serde_json::Value) -> Option<BatchErrors> {
    let error = value.get("error").or_else(|| {
        value
            .get("response")
            .and_then(|response| response.get("error"))
    })?;
    let code = error
        .get("code")
        .and_then(|value| value.as_str())
        .unwrap_or("gemini_error")
        .to_string();
    let message = error
        .get("message")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .unwrap_or_else(|| error.to_string());
    Some(BatchErrors {
        object: BatchErrorsObject::List,
        data: vec![BatchError {
            code,
            message,
            param: None,
            line: None,
        }],
    })
}

fn extract_timestamp(value: &serde_json::Value, key: &str) -> Option<u64> {
    value
        .get(key)
        .and_then(|value| value.as_str())
        .and_then(parse_rfc3339_timestamp)
}

fn extract_nested_timestamp(value: &serde_json::Value, path: &[&str]) -> Option<u64> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str().and_then(parse_rfc3339_timestamp)
}

fn parse_rfc3339_timestamp(value: &str) -> Option<u64> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.timestamp().max(0) as u64)
}

fn extract_u64(value: &serde_json::Value, keys: &[&str]) -> Option<u64> {
    for key in keys {
        if let Some(found) = value.get(*key) {
            if let Some(number) = found.as_u64() {
                return Some(number);
            }
            if let Some(value_str) = found.as_str() {
                if let Ok(number) = value_str.parse::<u64>() {
                    return Some(number);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_value(raw: &'static str) -> serde_json::Value {
        serde_json::from_str(raw).expect("fixture JSON should be valid")
    }

    #[test]
    fn batch_object_from_value_maps_success_status_output_and_counts() {
        let value = serde_json::json!({
            "name": "batches/123",
            "state": "JOB_STATE_SUCCEEDED",
            "createTime": "2025-01-01T00:00:00Z",
            "updateTime": "2025-01-01T00:30:00Z",
            "dest": { "fileName": "files/output-1" },
            "batchStats": {
                "totalRequestCount": "5",
                "completedRequestCount": "4",
                "failedRequestCount": "1"
            }
        });

        let batch = batch_object_from_value(
            value,
            &BatchEndpoint::ChatCompletions,
            &CompletionWindow::Hours24,
            "files/input-1",
        )
        .unwrap();

        assert_eq!(batch.id, "batches/123");
        assert_eq!(batch.endpoint, BatchEndpoint::ChatCompletions);
        assert_eq!(batch.status, BatchStatus::Completed);
        assert_eq!(batch.input_file_id, "files/input-1");
        assert_eq!(batch.output_file_id.as_deref(), Some("files/output-1"));
        assert_eq!(batch.request_counts.as_ref().map(|v| v.total), Some(5));
        assert_eq!(batch.request_counts.as_ref().map(|v| v.completed), Some(4));
        assert_eq!(batch.request_counts.as_ref().map(|v| v.failed), Some(1));
        assert!(batch.created_at > 0);
        assert!(batch.completed_at.is_some());
    }

    #[test]
    fn batch_object_from_value_maps_error_payload() {
        let value = serde_json::json!({
            "name": "batches/999",
            "state": "JOB_STATE_FAILED",
            "error": {
                "code": "INVALID_ARGUMENT",
                "message": "bad input"
            }
        });

        let batch = batch_object_from_value(
            value,
            &BatchEndpoint::Responses,
            &CompletionWindow::Hours24,
            "files/input-2",
        )
        .unwrap();

        assert_eq!(batch.status, BatchStatus::Failed);
        assert_eq!(
            batch.errors.as_ref().map(|e| e.data[0].code.clone()),
            Some("INVALID_ARGUMENT".to_string())
        );
        assert_eq!(
            batch.errors.as_ref().map(|e| e.data[0].message.clone()),
            Some("bad input".to_string())
        );
    }

    #[test]
    fn batch_object_from_value_normalizes_output_and_error_file_references() {
        let value = serde_json::json!({
            "name": "batches/normalize-1",
            "state": "JOB_STATE_SUCCEEDED",
            "response": {
                "responsesFile": "https://generativelanguage.googleapis.com/v1beta/files/output_123?alt=media",
                "errorsFile": "/v1beta/files/error_123#fragment"
            }
        });

        let batch = batch_object_from_value(
            value,
            &BatchEndpoint::Responses,
            &CompletionWindow::Hours24,
            "files/input-3",
        )
        .unwrap();

        assert_eq!(batch.output_file_id.as_deref(), Some("files/output_123"));
        assert_eq!(batch.error_file_id.as_deref(), Some("files/error_123"));
    }

    #[test]
    fn batch_object_from_value_drops_invalid_output_file_reference() {
        let value = serde_json::json!({
            "name": "batches/invalid-output-1",
            "state": "JOB_STATE_SUCCEEDED",
            "response": {
                "responsesFile": "batches/1234567890/some/non-file-reference-that-should-not-be-used"
            }
        });

        let batch = batch_object_from_value(
            value,
            &BatchEndpoint::ChatCompletions,
            &CompletionWindow::Hours24,
            "files/input-4",
        )
        .unwrap();

        assert_eq!(batch.output_file_id, None);
    }

    #[test]
    fn normalize_gemini_file_reference_rejects_oversized_ids() {
        let oversized = format!("files/{}", "a".repeat(GEMINI_MAX_FILE_ID_LEN + 1));
        assert_eq!(normalize_gemini_file_reference(&oversized), None);
    }

    #[test]
    fn normalize_gemini_file_reference_accepts_batch_output_file_id_over_40_chars() {
        let normalized =
            normalize_gemini_file_reference("files/batch-sy8vmuf1gsikc0qj8pm0dc2ptpjxsctwrekb");
        assert_eq!(
            normalized.as_deref(),
            Some("files/batch-sy8vmuf1gsikc0qj8pm0dc2ptpjxsctwrekb")
        );
    }

    #[test]
    fn batch_object_from_value_maps_nested_response_metadata_state() {
        let value = serde_json::json!({
            "name": "operations/abc",
            "response": {
                "name": "batches/abc",
                "metadata": {
                    "state": { "name": "JOB_STATE_SUCCEEDED" }
                }
            }
        });

        let batch = batch_object_from_value(
            value,
            &BatchEndpoint::Responses,
            &CompletionWindow::Hours24,
            "files/input-5",
        )
        .unwrap();

        assert_eq!(batch.status, BatchStatus::Completed);
    }

    #[test]
    fn batch_object_from_value_uses_done_fallback_when_state_is_missing() {
        let value = serde_json::json!({
            "name": "operations/done-1",
            "done": true,
            "response": {
                "name": "batches/done-1"
            }
        });

        let batch = batch_object_from_value(
            value,
            &BatchEndpoint::ChatCompletions,
            &CompletionWindow::Hours24,
            "files/input-6",
        )
        .unwrap();

        assert_eq!(batch.status, BatchStatus::Completed);
    }

    #[test]
    fn batch_object_from_value_prefers_batches_name_over_operation_name() {
        let value = serde_json::json!({
            "name": "operations/abc",
            "response": {
                "name": "batches/abc",
                "state": "JOB_STATE_RUNNING"
            }
        });

        let batch = batch_object_from_value(
            value,
            &BatchEndpoint::Responses,
            &CompletionWindow::Hours24,
            "files/input-7",
        )
        .unwrap();

        assert_eq!(batch.id, "batches/abc");
    }

    #[test]
    fn batch_object_from_value_extracts_top_level_responses_file() {
        let value = serde_json::json!({
            "name": "batches/top-level-1",
            "state": "JOB_STATE_SUCCEEDED",
            "responsesFile": "files/output_top_level_1"
        });

        let batch = batch_object_from_value(
            value,
            &BatchEndpoint::Responses,
            &CompletionWindow::Hours24,
            "files/input-8",
        )
        .unwrap();

        assert_eq!(
            batch.output_file_id.as_deref(),
            Some("files/output_top_level_1")
        );
    }

    #[test]
    fn batch_object_from_value_fallback_extracts_nested_unknown_output_path() {
        let value = serde_json::json!({
            "name": "batches/fallback-1",
            "state": "JOB_STATE_SUCCEEDED",
            "response": {
                "result": {
                    "artifact": "https://generativelanguage.googleapis.com/download/v1beta/files/output_fallback_1:download?alt=media"
                }
            }
        });

        let batch = batch_object_from_value(
            value,
            &BatchEndpoint::Responses,
            &CompletionWindow::Hours24,
            "files/input-9",
        )
        .unwrap();

        assert_eq!(
            batch.output_file_id.as_deref(),
            Some("files/output_fallback_1")
        );
    }

    #[test]
    fn batch_object_from_value_fallback_skips_input_file_reference() {
        let value = serde_json::json!({
            "name": "batches/fallback-2",
            "state": "JOB_STATE_SUCCEEDED",
            "inputConfig": {
                "fileName": "files/input_foo_1"
            },
            "response": {
                "somethingElse": "files/output_foo_1"
            }
        });

        let batch = batch_object_from_value(
            value,
            &BatchEndpoint::Responses,
            &CompletionWindow::Hours24,
            "files/input_foo_1",
        )
        .unwrap();

        assert_eq!(batch.output_file_id.as_deref(), Some("files/output_foo_1"));
    }

    #[test]
    fn normalize_gemini_file_reference_strips_download_suffix() {
        let normalized = normalize_gemini_file_reference(
            "https://generativelanguage.googleapis.com/download/v1beta/files/output_456:download?alt=media",
        );
        assert_eq!(normalized.as_deref(), Some("files/output_456"));
    }

    #[test]
    fn batch_object_from_value_fixture_operation_nested_state_and_files() {
        let value = fixture_value(include_str!(
            "fixtures/gemini_batch_operation_nested_state.json"
        ));

        let batch = batch_object_from_value(
            value,
            &BatchEndpoint::Responses,
            &CompletionWindow::Hours24,
            "files/input-fixture-op-1",
        )
        .unwrap();

        assert_eq!(batch.id, "batches/fixture-op-1");
        assert_eq!(batch.status, BatchStatus::InProgress);
        assert_eq!(
            batch.output_file_id.as_deref(),
            Some("files/output_fixture_op_1")
        );
        assert_eq!(
            batch.error_file_id.as_deref(),
            Some("files/error_fixture_op_1")
        );
        assert_eq!(batch.request_counts.as_ref().map(|v| v.total), Some(6));
        assert_eq!(batch.request_counts.as_ref().map(|v| v.completed), Some(2));
        assert_eq!(batch.request_counts.as_ref().map(|v| v.failed), Some(1));
    }

    #[test]
    fn batch_object_from_value_fixture_completed_nested_output_path() {
        let value = fixture_value(include_str!(
            "fixtures/gemini_batch_completed_nested_output.json"
        ));

        let batch = batch_object_from_value(
            value,
            &BatchEndpoint::Responses,
            &CompletionWindow::Hours24,
            "files/input-fixture-op-2",
        )
        .unwrap();

        assert_eq!(batch.id, "batches/fixture-op-2");
        assert_eq!(batch.status, BatchStatus::Completed);
        assert_eq!(
            batch.output_file_id.as_deref(),
            Some("files/output_fixture_nested_2")
        );
    }
}
