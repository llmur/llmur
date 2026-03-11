use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::data::virtual_batch::VirtualBatchEndpoint;
use crate::errors::LLMurError;
use crate::providers::Transformer;
use crate::providers::gemini::v1beta::embed_content::response::Response as GeminiEmbedContentResponse;
use crate::providers::gemini::v1beta::embed_content::response::to_openai_transform::Context as GeminiEmbeddingsResponseContext;
use crate::providers::gemini::v1beta::generate_content::request::Request as GeminiGenerateContentRequest;
use crate::providers::gemini::v1beta::generate_content::response::Response as GeminiGenerateContentResponse;
use crate::providers::gemini::v1beta::generate_content::response::to_openai_responses_transform::Context as GeminiResponsesResponseContext;
use crate::providers::gemini::v1beta::generate_content::response::to_openai_transform::Context as GeminiChatCompletionsResponseContext;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchInputLine {
    pub key: String,
    pub request: GeminiGenerateContentRequest,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAiBatchResponseBody {
    pub status_code: u16,
    pub request_id: String,
    pub body: JsonValue,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAiBatchErrorBody {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAiBatchOutputLine {
    pub id: String,
    pub custom_id: String,
    pub response: Option<OpenAiBatchResponseBody>,
    pub error: Option<OpenAiBatchErrorBody>,
}

pub fn transform_batch_output_content(
    content: &str,
    batch_endpoint: &VirtualBatchEndpoint,
    model: &str,
) -> Result<String, LLMurError> {
    let mut output_lines: Vec<String> = Vec::new();
    for (index, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(trimmed).map_err(|err| {
            LLMurError::BadRequest(format!(
                "Gemini batch output line {} is not valid JSON: {}",
                index + 1,
                err
            ))
        })?;
        let custom_id = extract_batch_key(&value).ok_or_else(|| {
            LLMurError::BadRequest(format!(
                "Gemini batch output line {} is missing a key.",
                index + 1
            ))
        })?;

        let response_value = match value.get("response") {
            Some(response) if !response.is_null() => Some(response.clone()),
            _ => {
                if value.get("error").is_some() {
                    None
                } else {
                    Some(value.clone())
                }
            }
        };

        let (response, error) = if let Some(response_value) = response_value {
            let response_body =
                gemini_response_to_openai(response_value, batch_endpoint, model, index + 1)?;
            (
                Some(OpenAiBatchResponseBody {
                    status_code: 200,
                    request_id: format!("gemini-batch-{}", custom_id),
                    body: response_body,
                }),
                None,
            )
        } else {
            let error_value = value.get("error").cloned().unwrap_or_else(|| value.clone());
            (None, Some(parse_gemini_error(error_value)))
        };

        let line = OpenAiBatchOutputLine {
            id: format!("batch_req_{}", custom_id),
            custom_id,
            response,
            error,
        };
        let line_json = serde_json::to_string(&line).map_err(|err| {
            LLMurError::BadRequest(format!(
                "Gemini batch output line {} serialization failed: {}",
                index + 1,
                err
            ))
        })?;
        output_lines.push(line_json);
    }

    let mut content = output_lines.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    Ok(content)
}

fn extract_batch_key(value: &serde_json::Value) -> Option<String> {
    value
        .get("key")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .or_else(|| {
            value
                .get("metadata")
                .and_then(|meta| meta.get("key"))
                .and_then(|value| value.as_str())
                .map(|value| value.to_string())
        })
        .or_else(|| {
            value
                .get("request")
                .and_then(|req| req.get("metadata"))
                .and_then(|meta| meta.get("key"))
                .and_then(|value| value.as_str())
                .map(|value| value.to_string())
        })
}

fn parse_gemini_error(value: serde_json::Value) -> OpenAiBatchErrorBody {
    let (code, message) = if let Some(message) = value.as_str() {
        ("gemini_error".to_string(), message.to_string())
    } else {
        let code = value
            .get("code")
            .and_then(|value| value.as_str())
            .unwrap_or("gemini_error")
            .to_string();
        let message = value
            .get("message")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string())
            .unwrap_or_else(|| value.to_string());
        (code, message)
    };

    OpenAiBatchErrorBody { code, message }
}

fn gemini_response_to_openai(
    value: serde_json::Value,
    batch_endpoint: &VirtualBatchEndpoint,
    model: &str,
    line_number: usize,
) -> Result<JsonValue, LLMurError> {
    match batch_endpoint {
        VirtualBatchEndpoint::ChatCompletions => {
            let response: GeminiGenerateContentResponse =
                serde_json::from_value(value).map_err(|err| {
                    LLMurError::BadRequest(format!(
                        "Gemini batch output line {} response is invalid: {}",
                        line_number, err
                    ))
                })?;
            let transformed = response
                .transform(GeminiChatCompletionsResponseContext {
                    model: Some(model.to_string()),
                })
                .result;
            serde_json::to_value(transformed).map_err(|err| {
                LLMurError::BadRequest(format!(
                    "Gemini batch output line {} response serialization failed: {}",
                    line_number, err
                ))
            })
        }
        VirtualBatchEndpoint::Responses => {
            let response: GeminiGenerateContentResponse =
                serde_json::from_value(value).map_err(|err| {
                    LLMurError::BadRequest(format!(
                        "Gemini batch output line {} response is invalid: {}",
                        line_number, err
                    ))
                })?;
            let transformed = response
                .transform(GeminiResponsesResponseContext {
                    model: Some(model.to_string()),
                    parallel_tool_calls: None,
                    previous_response_id: None,
                    reasoning: None,
                    max_output_tokens: None,
                    instructions: None,
                    text: None,
                    tools: None,
                    tool_choice: None,
                    truncation: None,
                    metadata: None,
                    temperature: None,
                    top_p: None,
                    user: None,
                    service_tier: None,
                })
                .result;
            serde_json::to_value(transformed).map_err(|err| {
                LLMurError::BadRequest(format!(
                    "Gemini batch output line {} response serialization failed: {}",
                    line_number, err
                ))
            })
        }
        VirtualBatchEndpoint::Embeddings => {
            let response: GeminiEmbedContentResponse =
                serde_json::from_value(value).map_err(|err| {
                    LLMurError::BadRequest(format!(
                        "Gemini batch output line {} response is invalid: {}",
                        line_number, err
                    ))
                })?;
            let transformed = response
                .transform(GeminiEmbeddingsResponseContext {
                    model: Some(model.to_string()),
                })
                .result;
            serde_json::to_value(transformed).map_err(|err| {
                LLMurError::BadRequest(format!(
                    "Gemini batch output line {} response serialization failed: {}",
                    line_number, err
                ))
            })
        }
        VirtualBatchEndpoint::Completions => Err(LLMurError::BadRequest(format!(
            "Gemini does not support /v1/completions for batches (line {}).",
            line_number
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_batch_output_content_maps_error_line() {
        let content = "{\"key\":\"line-1\",\"error\":{\"code\":\"FAILED\",\"message\":\"boom\"}}\n";
        let transformed = transform_batch_output_content(
            content,
            &VirtualBatchEndpoint::Responses,
            "gemini-2.0-flash",
        )
        .unwrap();

        let first_line = transformed.lines().next().unwrap();
        let decoded: OpenAiBatchOutputLine = serde_json::from_str(first_line).unwrap();
        assert_eq!(decoded.custom_id, "line-1");
        assert!(decoded.response.is_none());
        assert_eq!(
            decoded.error.as_ref().map(|e| e.code.as_str()),
            Some("FAILED")
        );
        assert_eq!(
            decoded.error.as_ref().map(|e| e.message.as_str()),
            Some("boom")
        );
    }

    #[test]
    fn transform_batch_output_content_rejects_completions_endpoint() {
        let content = "{\"key\":\"line-1\",\"response\":{}}\n";
        let err = transform_batch_output_content(
            content,
            &VirtualBatchEndpoint::Completions,
            "gemini-2.0-flash",
        )
        .unwrap_err();

        match err {
            LLMurError::BadRequest(message) => {
                assert!(message.contains("does not support /v1/completions"));
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }
}
