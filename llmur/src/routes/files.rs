use crate::LLMurState;
use crate::data::connection::{ConnectionInfo, GeminiApiVersion};
use crate::data::deployment::DeploymentId;
use crate::data::graph::{Graph, NodeLimitsChecker};
use crate::data::virtual_batch::VirtualBatchEndpoint;
use crate::data::virtual_file::{
    ProviderFileReference, VirtualFile, VirtualFileId, VirtualFileKind, VirtualFileLineMapEntry,
};
use crate::data::virtual_key::VirtualKeyId;
use crate::errors::{GraphError, LLMurError, ProxyError, ProxyErrorMessage};
use crate::providers::ExposesDeployment;
use crate::providers::ExposesUsage;
use crate::providers::Transformer;
use crate::providers::azure::openai::v1::chat_completions::request::from_openai_transform::Context as AzureChatCompletionsContext;
use crate::providers::azure::openai::v1::chat_completions::response::{
    Response as AzureChatCompletionsResponse,
    to_openai_transform::Context as AzureChatCompletionsResponseContext,
};
use crate::providers::azure::openai::v1::embeddings::request::from_openai_transform::Context as AzureEmbeddingsContext;
use crate::providers::azure::openai::v1::embeddings::response::{
    Response as AzureEmbeddingsResponse,
    to_openai_transform::Context as AzureEmbeddingsResponseContext,
};
use crate::providers::azure::openai::v1::files::types::FileObject as AzureFileObject;
use crate::providers::gemini::v1beta::batches::types::{
    BatchInputLine as GeminiBatchInputLine, transform_batch_output_content,
};
use crate::providers::gemini::v1beta::files::request::ResumableUploadStartRequest;
use crate::providers::gemini::v1beta::files::response::FileUploadResponse;
use crate::providers::gemini::v1beta::files::types::UploadFileMetadata;
use crate::providers::gemini::v1beta::generate_content::request::from_openai_responses_transform::Context as GeminiResponsesContext;
use crate::providers::gemini::v1beta::generate_content::request::from_openai_transform::Context as GeminiChatCompletionsContext;
use crate::providers::openai::chat_completions::request::Request as ChatCompletionsRequest;
use crate::providers::openai::chat_completions::request::to_self::Context as ChatCompletionsContext;
use crate::providers::openai::chat_completions::response::{
    Response as OpenAiChatCompletionsResponse,
    to_self::Context as OpenAiChatCompletionsResponseContext,
};
use crate::providers::openai::completions::request::Request as CompletionsRequest;
use crate::providers::openai::completions::request::to_self::Context as CompletionsContext;
use crate::providers::openai::completions::response::{
    Response as OpenAiCompletionsResponse, to_self::Context as OpenAiCompletionsResponseContext,
};
use crate::providers::openai::embeddings::request::Request as EmbeddingsRequest;
use crate::providers::openai::embeddings::request::to_self::Context as EmbeddingsContext;
use crate::providers::openai::embeddings::response::{
    Response as OpenAiEmbeddingsResponse, to_self::Context as OpenAiEmbeddingsResponseContext,
};
use crate::providers::openai::files::request::ListFilesQuery;
use crate::providers::openai::files::response::{
    DeleteFileResponse, ListFilesResponse, ListObject,
};
use crate::providers::openai::files::types::{FileObject, FileObjectType, FilePurpose, FileStatus};
use crate::providers::openai::responses::request::Request as ResponsesRequest;
use crate::providers::openai::responses::request::to_self::Context as ResponsesContext;
use crate::providers::openai::responses::response::{
    Response as OpenAiResponsesResponse, to_self::Context as OpenAiResponsesResponseContext,
};
use crate::routes::middleware::auth::AuthorizationHeaderExtractionResult;
use crate::routes::openai::proxy_helpers::{
    log_proxy_response, proxy_request_json, register_proxy_metrics,
};
use crate::routes::openai::response::{ProviderResponse, ProxyResponse};
use crate::routes::utils::{extract_api_key, parse_virtual_file_id};
use axum::Extension;
use axum::body::Body;
use axum::extract::{Multipart, Path, Query, State};
use chrono::Utc;
use reqwest::header::HeaderMap as ReqwestHeaderMap;
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI16, Ordering};
use std::time::Instant;
use uuid::Uuid;

struct ParsedFileUpload {
    filename: String,
    bytes: Vec<u8>,
    purpose: FilePurpose,
}

#[derive(Clone, Serialize)]
pub(crate) struct NoUsage;

impl ExposesUsage for NoUsage {
    fn get_input_tokens(&self) -> u64 {
        0
    }

    fn get_output_tokens(&self) -> u64 {
        0
    }
}

#[derive(Debug, Deserialize)]
struct BatchInputLine {
    custom_id: String,
    method: String,
    url: String,
    body: JsonValue,
}

#[derive(Debug, Serialize)]
struct BatchOutputLine {
    custom_id: String,
    method: String,
    url: String,
    body: JsonValue,
}

struct ProviderFileGroup {
    graph: Graph,
    provider: String,
    deployment_id: DeploymentId,
    lines: Vec<String>,
    line_indices: Vec<usize>,
}

#[tracing::instrument(name = "handler.openai.v1.files.create", skip(state, auth, multipart))]
pub(crate) async fn create_file(
    State(state): State<Arc<LLMurState>>,
    Extension(auth): Extension<AuthorizationHeaderExtractionResult>,
    Extension(request_id): Extension<crate::data::request_log::RequestLogId>,
    mut multipart: Multipart,
) -> Result<ProxyResponse<FileObject>, LLMurError> {
    let upload = parse_multipart_upload(&mut multipart).await?;
    if upload.purpose == FilePurpose::Batch {
        return create_virtual_batch_file(state, auth, request_id, upload).await;
    }
    Err(LLMurError::BadRequest(
        "Files API is only supported for batch purpose.".to_string(),
    ))
}

#[tracing::instrument(
    name = "handler.openai.v1.files.create_batch",
    skip(state, auth, request_id, upload)
)]
async fn create_virtual_batch_file(
    state: Arc<LLMurState>,
    auth: AuthorizationHeaderExtractionResult,
    request_id: crate::data::request_log::RequestLogId,
    upload: ParsedFileUpload,
) -> Result<ProxyResponse<FileObject>, LLMurError> {
    let overall_start = Instant::now();
    let start_ts = Utc::now();
    let log_attempts = Arc::new(AtomicI16::new(0));
    let api_key = extract_api_key(auth)?;
    let content = std::str::from_utf8(&upload.bytes)
        .map_err(|_| LLMurError::BadRequest("Batch file must be valid UTF-8 JSONL.".to_string()))?;
    let lines = parse_batch_lines(content)?;
    if lines.is_empty() {
        return Err(LLMurError::BadRequest(
            "Batch file must include at least one JSONL line.".to_string(),
        ));
    }

    let mut graph_cache: HashMap<String, Graph> = HashMap::new();
    let mut groups: HashMap<(String, DeploymentId), ProviderFileGroup> = HashMap::new();
    let mut line_map_slots: Vec<Option<VirtualFileLineMapEntry>> = vec![None; lines.len()];
    let mut primary_graph: Option<Graph> = None;

    // Parse each client JSONL line, transform it into the provider-specific batch input shape,
    // then group lines by (provider, deployment) so each upstream gets one file upload.
    for (line_index, line) in lines.into_iter().enumerate() {
        let line_number = line_index + 1;
        let BatchInputLine {
            custom_id,
            method,
            url,
            body,
        } = line;

        if custom_id.trim().is_empty() {
            return Err(LLMurError::BadRequest(format!(
                "Batch line {} must include custom_id.",
                line_number
            )));
        }

        let method = method.to_uppercase();
        if method != "POST" {
            return Err(LLMurError::BadRequest(format!(
                "Batch line {} method must be POST.",
                line_number
            )));
        }

        let url_ref = url.as_str();
        let (line_json, graph) = match url_ref {
            "/v1/responses" => {
                let request: ResponsesRequest = serde_json::from_value(body).map_err(|err| {
                    LLMurError::BadRequest(format!(
                        "Batch line {} /v1/responses body is invalid: {}",
                        line_number, err
                    ))
                })?;
                let deployment_name = request.get_deployment_ref().to_string();
                let graph = load_graph_for_deployment(
                    state.as_ref(),
                    &api_key,
                    &deployment_name,
                    &mut graph_cache,
                )
                .await?;
                let line_json = match &graph.connection.data.connection_info {
                    ConnectionInfo::OpenAiApiKey { model, .. } => {
                        let transformed = request
                            .transform(ResponsesContext {
                                model: Some(model.clone()),
                            })
                            .result;
                        let body_value = serde_json::to_value(transformed).map_err(|err| {
                            LLMurError::BadRequest(format!(
                                "Batch line {} /v1/responses serialization failed: {}",
                                line_number, err
                            ))
                        })?;
                        let output_line = BatchOutputLine {
                            custom_id,
                            method,
                            url,
                            body: body_value,
                        };
                        let line_json = serde_json::to_string(&output_line).map_err(|err| {
                            LLMurError::BadRequest(format!(
                                "Batch line {} serialization failed: {}",
                                line_number, err
                            ))
                        })?;
                        line_json
                    }
                    ConnectionInfo::GeminiApiKey { model, .. } => {
                        // Gemini batch files use `key` + provider-native request payload.
                        let transformed = request
                            .transform(GeminiResponsesContext {
                                model: Some(model.clone()),
                            })
                            .result;
                        let input_line = GeminiBatchInputLine {
                            key: custom_id,
                            request: transformed,
                        };
                        let line_json = serde_json::to_string(&input_line).map_err(|err| {
                            LLMurError::BadRequest(format!(
                                "Batch line {} serialization failed: {}",
                                line_number, err
                            ))
                        })?;
                        line_json
                    }
                    ConnectionInfo::AzureOpenAiApiKey { .. } => {
                        return Err(LLMurError::BadRequest(format!(
                            "Batch line {} targets /v1/responses, which Azure OpenAI does not support for batches.",
                            line_number
                        )));
                    }
                };
                (line_json, graph)
            }
            "/v1/chat/completions" => {
                let request: ChatCompletionsRequest =
                    serde_json::from_value(body).map_err(|err| {
                        LLMurError::BadRequest(format!(
                            "Batch line {} /v1/chat/completions body is invalid: {}",
                            line_number, err
                        ))
                    })?;
                let deployment_name = request.get_deployment_ref().to_string();
                let graph = load_graph_for_deployment(
                    state.as_ref(),
                    &api_key,
                    &deployment_name,
                    &mut graph_cache,
                )
                .await?;
                let line_json = match &graph.connection.data.connection_info {
                    ConnectionInfo::OpenAiApiKey { model, .. } => {
                        let transformed = request
                            .transform(ChatCompletionsContext {
                                model: Some(model.clone()),
                                stream_include_usage: true,
                            })
                            .result;
                        let body_value = serde_json::to_value(transformed).map_err(|err| {
                            LLMurError::BadRequest(format!(
                                "Batch line {} /v1/chat/completions serialization failed: {}",
                                line_number, err
                            ))
                        })?;
                        let output_line = BatchOutputLine {
                            custom_id,
                            method,
                            url,
                            body: body_value,
                        };
                        let line_json = serde_json::to_string(&output_line).map_err(|err| {
                            LLMurError::BadRequest(format!(
                                "Batch line {} serialization failed: {}",
                                line_number, err
                            ))
                        })?;
                        line_json
                    }
                    ConnectionInfo::AzureOpenAiApiKey {
                        deployment_name, ..
                    } => {
                        let transformed = request
                            .transform(AzureChatCompletionsContext {
                                model: Some(deployment_name.clone()),
                                safety_identifier: None,
                                prompt_cache_key: None,
                                prompt_cache_retention: None,
                                user_security_context: None,
                                stream_include_usage: true,
                            })
                            .result;
                        let body_value = serde_json::to_value(transformed).map_err(|err| {
                            LLMurError::BadRequest(format!(
                                "Batch line {} /v1/chat/completions serialization failed: {}",
                                line_number, err
                            ))
                        })?;
                        let output_line = BatchOutputLine {
                            custom_id,
                            method,
                            url,
                            body: body_value,
                        };
                        let line_json = serde_json::to_string(&output_line).map_err(|err| {
                            LLMurError::BadRequest(format!(
                                "Batch line {} serialization failed: {}",
                                line_number, err
                            ))
                        })?;
                        line_json
                    }
                    ConnectionInfo::GeminiApiKey { model, .. } => {
                        // Gemini batch files use `key` + provider-native request payload.
                        let transformed = request
                            .transform(GeminiChatCompletionsContext {
                                model: Some(model.clone()),
                            })
                            .result;
                        let input_line = GeminiBatchInputLine {
                            key: custom_id,
                            request: transformed,
                        };
                        let line_json = serde_json::to_string(&input_line).map_err(|err| {
                            LLMurError::BadRequest(format!(
                                "Batch line {} serialization failed: {}",
                                line_number, err
                            ))
                        })?;
                        line_json
                    }
                };
                (line_json, graph)
            }
            "/v1/embeddings" => {
                let request: EmbeddingsRequest = serde_json::from_value(body).map_err(|err| {
                    LLMurError::BadRequest(format!(
                        "Batch line {} /v1/embeddings body is invalid: {}",
                        line_number, err
                    ))
                })?;
                let deployment_name = request.get_deployment_ref().to_string();
                let graph = load_graph_for_deployment(
                    state.as_ref(),
                    &api_key,
                    &deployment_name,
                    &mut graph_cache,
                )
                .await?;
                let line_json = match &graph.connection.data.connection_info {
                    ConnectionInfo::OpenAiApiKey { model, .. } => {
                        let transformed = request
                            .transform(EmbeddingsContext {
                                model: Some(model.clone()),
                            })
                            .result;
                        let body_value = serde_json::to_value(transformed).map_err(|err| {
                            LLMurError::BadRequest(format!(
                                "Batch line {} /v1/embeddings serialization failed: {}",
                                line_number, err
                            ))
                        })?;
                        let output_line = BatchOutputLine {
                            custom_id,
                            method,
                            url,
                            body: body_value,
                        };
                        let line_json = serde_json::to_string(&output_line).map_err(|err| {
                            LLMurError::BadRequest(format!(
                                "Batch line {} serialization failed: {}",
                                line_number, err
                            ))
                        })?;
                        line_json
                    }
                    ConnectionInfo::AzureOpenAiApiKey {
                        deployment_name, ..
                    } => {
                        let transformed = request
                            .transform(AzureEmbeddingsContext {
                                model: Some(deployment_name.clone()),
                            })
                            .result;
                        let body_value = serde_json::to_value(transformed).map_err(|err| {
                            LLMurError::BadRequest(format!(
                                "Batch line {} /v1/embeddings serialization failed: {}",
                                line_number, err
                            ))
                        })?;
                        let output_line = BatchOutputLine {
                            custom_id,
                            method,
                            url,
                            body: body_value,
                        };
                        let line_json = serde_json::to_string(&output_line).map_err(|err| {
                            LLMurError::BadRequest(format!(
                                "Batch line {} serialization failed: {}",
                                line_number, err
                            ))
                        })?;
                        line_json
                    }
                    ConnectionInfo::GeminiApiKey { .. } => {
                        return Err(LLMurError::BadRequest(format!(
                            "Batch line {} targets /v1/embeddings, which Gemini does not support for batches yet.",
                            line_number
                        )));
                    }
                };
                (line_json, graph)
            }
            "/v1/completions" => {
                let request: CompletionsRequest = serde_json::from_value(body).map_err(|err| {
                    LLMurError::BadRequest(format!(
                        "Batch line {} /v1/completions body is invalid: {}",
                        line_number, err
                    ))
                })?;
                let deployment_name = request.get_deployment_ref().to_string();
                let graph = load_graph_for_deployment(
                    state.as_ref(),
                    &api_key,
                    &deployment_name,
                    &mut graph_cache,
                )
                .await?;
                let line_json = match &graph.connection.data.connection_info {
                    ConnectionInfo::OpenAiApiKey { model, .. } => {
                        let transformed = request
                            .transform(CompletionsContext {
                                model: Some(model.clone()),
                            })
                            .result;
                        let body_value = serde_json::to_value(transformed).map_err(|err| {
                            LLMurError::BadRequest(format!(
                                "Batch line {} /v1/completions serialization failed: {}",
                                line_number, err
                            ))
                        })?;
                        let output_line = BatchOutputLine {
                            custom_id,
                            method,
                            url,
                            body: body_value,
                        };
                        let line_json = serde_json::to_string(&output_line).map_err(|err| {
                            LLMurError::BadRequest(format!(
                                "Batch line {} serialization failed: {}",
                                line_number, err
                            ))
                        })?;
                        line_json
                    }
                    ConnectionInfo::AzureOpenAiApiKey { .. } => {
                        return Err(LLMurError::BadRequest(format!(
                            "Batch line {} targets /v1/completions, which Azure OpenAI does not support for batches.",
                            line_number
                        )));
                    }
                    ConnectionInfo::GeminiApiKey { .. } => {
                        return Err(LLMurError::BadRequest(format!(
                            "Batch line {} targets /v1/completions, which Gemini does not support.",
                            line_number
                        )));
                    }
                };
                (line_json, graph)
            }
            _ => {
                return Err(LLMurError::BadRequest(format!(
                    "Batch line {} has unsupported url '{}'.",
                    line_number, url_ref
                )));
            }
        };

        let provider = graph
            .connection
            .data
            .connection_info
            .get_provider_friendly_name()
            .to_string();
        let deployment_id = graph.deployment.data.id;
        let group_key = (provider.clone(), deployment_id);
        let group = groups
            .entry(group_key)
            .or_insert_with(|| ProviderFileGroup {
                graph: graph.clone(),
                provider,
                deployment_id,
                lines: Vec::new(),
                line_indices: Vec::new(),
            });

        group.lines.push(line_json);
        group.line_indices.push(line_index);

        if primary_graph.is_none() {
            primary_graph = Some(graph);
        }
    }

    // Upload one provider file per (provider, deployment) group and record line-level mapping
    // so virtual content can later be reconstructed in original client line order.
    let mut provider_files: Vec<ProviderFileReference> = Vec::new();
    for ((_provider, _deployment_id), group) in groups.into_iter() {
        if group.lines.is_empty() {
            continue;
        }
        let mut file_content = group.lines.join("\n");
        file_content.push('\n');

        let provider_file_id = match &group.graph.connection.data.connection_info {
            ConnectionInfo::OpenAiApiKey {
                api_key,
                api_endpoint,
                ..
            } => {
                let file_part = Part::bytes(file_content.into_bytes())
                    .file_name(upload.filename.clone())
                    .mime_str("application/jsonl")
                    .map_err(|err| {
                        LLMurError::BadRequest(format!("Batch file MIME type is invalid: {}", err))
                    })?;
                let form = Form::new()
                    .part("file", file_part)
                    .text("purpose", FilePurpose::Batch.to_string());

                let url = format!("{}/v1/files", api_endpoint.trim_end_matches('/'));
                let mut request_headers = ReqwestHeaderMap::new();
                request_headers.insert(
                    "Authorization",
                    format!("Bearer {}", api_key).parse().unwrap(),
                );

                let start = Instant::now();
                let request_ts = Utc::now();
                let request = state
                    .data
                    .http_client
                    .post(url)
                    .headers(request_headers)
                    .multipart(form);
                let response = proxy_request_json::<AzureFileObject>(request).await;
                let proxy_response = ProxyResponse::new(response, request_ts);
                log_proxy_response(
                    state.as_ref(),
                    &request_id,
                    &group.graph,
                    "POST",
                    "/v1/files",
                    next_log_attempt(&log_attempts),
                    &proxy_response,
                );
                register_proxy_metrics(
                    state.as_ref(),
                    &group.graph,
                    &proxy_response,
                    "/v1/files",
                    start.elapsed().as_millis() as u64,
                );

                let provider_file_id = match proxy_response.result {
                    Ok(ProviderResponse::DecodedResponse { data, .. }) => data.id,
                    Ok(ProviderResponse::JsonResponse { .. }) => {
                        return Ok(ProxyResponse::new(
                            Err(ProxyError::InternalError(
                                "Files API returned an unexpected response.".to_string(),
                            )),
                            request_ts,
                        ));
                    }
                    Ok(ProviderResponse::Stream { .. }) => {
                        return Ok(ProxyResponse::new(
                            Err(ProxyError::InternalError(
                                "Files API returned an unexpected stream response.".to_string(),
                            )),
                            request_ts,
                        ));
                    }
                    Err(error) => return Ok(ProxyResponse::new(Err(error), request_ts)),
                };
                provider_file_id
            }
            ConnectionInfo::AzureOpenAiApiKey {
                api_key,
                api_endpoint,
                api_version,
                ..
            } => {
                let file_part = Part::bytes(file_content.into_bytes())
                    .file_name(upload.filename.clone())
                    .mime_str("application/json")
                    .map_err(|err| {
                        LLMurError::BadRequest(format!("Batch file MIME type is invalid: {}", err))
                    })?;
                let form = Form::new()
                    .part("file", file_part)
                    .text("purpose", FilePurpose::Batch.to_string());

                let api_base = azure_api_base(api_endpoint);
                let url = format!("{}/files?api-version={}", api_base, api_version);
                let mut request_headers = ReqwestHeaderMap::new();
                request_headers.insert("api-key", api_key.parse().unwrap());

                let start = Instant::now();
                let request_ts = Utc::now();
                let request = state
                    .data
                    .http_client
                    .post(url)
                    .headers(request_headers)
                    .multipart(form);
                let response = proxy_request_json::<FileObject>(request).await;
                let proxy_response = ProxyResponse::new(response, request_ts);
                log_proxy_response(
                    state.as_ref(),
                    &request_id,
                    &group.graph,
                    "POST",
                    "/v1/files",
                    next_log_attempt(&log_attempts),
                    &proxy_response,
                );
                register_proxy_metrics(
                    state.as_ref(),
                    &group.graph,
                    &proxy_response,
                    "/v1/files",
                    start.elapsed().as_millis() as u64,
                );

                let provider_file_id = match proxy_response.result {
                    Ok(ProviderResponse::DecodedResponse { data, .. }) => data.id,
                    Ok(ProviderResponse::JsonResponse { .. }) => {
                        return Ok(ProxyResponse::new(
                            Err(ProxyError::InternalError(
                                "Files API returned an unexpected response.".to_string(),
                            )),
                            request_ts,
                        ));
                    }
                    Ok(ProviderResponse::Stream { .. }) => {
                        return Ok(ProxyResponse::new(
                            Err(ProxyError::InternalError(
                                "Files API returned an unexpected stream response.".to_string(),
                            )),
                            request_ts,
                        ));
                    }
                    Err(error) => return Ok(ProxyResponse::new(Err(error), request_ts)),
                };
                provider_file_id
            }
            ConnectionInfo::GeminiApiKey {
                api_key,
                api_endpoint,
                api_version,
                ..
            } => {
                // Gemini file upload is resumable and two-step:
                // 1) start session to obtain upload URL, 2) upload+finalize bytes to that URL.
                let file_bytes = file_content.into_bytes();
                let file_len = file_bytes.len();
                let api_version = gemini_api_version(api_version);
                let api_base = api_endpoint.trim_end_matches('/');
                let start_url = format!("{}/upload/{}/files", api_base, api_version);
                let mut request_headers = ReqwestHeaderMap::new();
                request_headers.insert("x-goog-api-key", api_key.parse().unwrap());
                request_headers.insert("X-Goog-Upload-Protocol", "resumable".parse().unwrap());
                request_headers.insert("X-Goog-Upload-Command", "start".parse().unwrap());
                request_headers.insert(
                    "X-Goog-Upload-Header-Content-Length",
                    file_len.to_string().parse().unwrap(),
                );
                request_headers.insert(
                    "X-Goog-Upload-Header-Content-Type",
                    "application/jsonl".parse().unwrap(),
                );
                request_headers.insert("Content-Type", "application/jsonl".parse().unwrap());
                let start_payload = ResumableUploadStartRequest {
                    file: UploadFileMetadata {
                        display_name: Some(upload.filename.clone()),
                        mime_type: Some("application/jsonl".to_string()),
                    },
                };
                let start_body = serde_json::to_vec(&start_payload).map_err(ProxyError::from)?;

                let start_response = state
                    .data
                    .http_client
                    .post(start_url)
                    .headers(request_headers)
                    .body(start_body)
                    .send()
                    .await
                    .map_err(ProxyError::from)?;
                let start_status = start_response.status();
                let start_headers = start_response.headers().clone();
                if !start_status.is_success() {
                    let body = start_response
                        .text()
                        .await
                        .unwrap_or_else(|_| "".to_string());
                    return Ok(ProxyResponse::new(
                        Err(ProxyError::ProxyReturnError(
                            start_status,
                            ProxyErrorMessage::Text(body),
                        )),
                        Utc::now(),
                    ));
                }
                let upload_url = start_headers
                    .get("x-goog-upload-url")
                    .and_then(|value| value.to_str().ok())
                    .ok_or_else(|| {
                        LLMurError::BadRequest(
                            "Gemini file upload did not return an upload URL.".to_string(),
                        )
                    })?
                    .to_string();

                let mut upload_headers = ReqwestHeaderMap::new();
                upload_headers.insert("X-Goog-Upload-Command", "upload, finalize".parse().unwrap());
                upload_headers.insert("X-Goog-Upload-Offset", "0".parse().unwrap());
                upload_headers.insert(
                    reqwest::header::CONTENT_LENGTH,
                    file_len.to_string().parse().unwrap(),
                );

                let start = Instant::now();
                let request_ts = Utc::now();
                let request = state
                    .data
                    .http_client
                    .post(upload_url)
                    .headers(upload_headers)
                    .body(file_bytes);
                let response = proxy_request_json::<FileUploadResponse>(request).await;
                let proxy_response = ProxyResponse::new(response, request_ts);
                log_proxy_response(
                    state.as_ref(),
                    &request_id,
                    &group.graph,
                    "POST",
                    "/v1/files",
                    next_log_attempt(&log_attempts),
                    &proxy_response,
                );
                register_proxy_metrics(
                    state.as_ref(),
                    &group.graph,
                    &proxy_response,
                    "/v1/files",
                    start.elapsed().as_millis() as u64,
                );

                let provider_file_id = match proxy_response.result {
                    Ok(ProviderResponse::DecodedResponse { data, .. }) => data.file.name,
                    Ok(ProviderResponse::JsonResponse { .. }) => {
                        return Ok(ProxyResponse::new(
                            Err(ProxyError::InternalError(
                                "Files API returned an unexpected response.".to_string(),
                            )),
                            request_ts,
                        ));
                    }
                    Ok(ProviderResponse::Stream { .. }) => {
                        return Ok(ProxyResponse::new(
                            Err(ProxyError::InternalError(
                                "Files API returned an unexpected stream response.".to_string(),
                            )),
                            request_ts,
                        ));
                    }
                    Err(error) => return Ok(ProxyResponse::new(Err(error), request_ts)),
                };
                provider_file_id
            }
        };

        provider_files.push(ProviderFileReference {
            provider: group.provider.clone(),
            deployment_id: group.deployment_id,
            provider_file_id: provider_file_id.clone(),
        });

        for (provider_line_index, line_index) in group.line_indices.iter().enumerate() {
            line_map_slots[*line_index] = Some(VirtualFileLineMapEntry {
                line_index: *line_index as i32,
                provider: group.provider.clone(),
                deployment_id: group.deployment_id,
                provider_file_id: provider_file_id.clone(),
                provider_line_index: provider_line_index as i32,
            });
        }
    }

    let line_map: Vec<VirtualFileLineMapEntry> = line_map_slots
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            entry.ok_or_else(|| {
                LLMurError::BadRequest(format!(
                    "Batch line {} could not be mapped to a provider file.",
                    index + 1
                ))
            })
        })
        .collect::<Result<_, _>>()?;

    let virtual_key_id = VirtualKeyId::from_decrypted_key(&api_key);
    let virtual_file_id = VirtualFileId(Uuid::now_v7());
    let line_count = line_map.len() as i32;

    let _ = state
        .data
        .create_virtual_file(
            &virtual_file_id,
            &virtual_key_id,
            &FilePurpose::Batch,
            &upload.filename,
            upload.bytes.len() as i64,
            &VirtualFileKind::Input,
            &None,
            &Some(line_count),
            &Some(provider_files),
            &Some(line_map),
            &None,
            &None,
            &state.metrics,
        )
        .await?;

    let file_object = FileObject {
        id: virtual_file_id.0.to_string(),
        bytes: upload.bytes.len() as u64,
        created_at: start_ts.timestamp() as u64,
        expires_at: None,
        filename: upload.filename,
        object: FileObjectType::File,
        purpose: FilePurpose::Batch,
        status: FileStatus::Uploaded,
        status_details: None,
    };

    let response = ProxyResponse::new(
        Ok(ProviderResponse::DecodedResponse {
            data: file_object,
            status_code: reqwest::StatusCode::OK,
        }),
        start_ts,
    );

    if let Some(graph) = primary_graph {
        log_proxy_response(
            state.as_ref(),
            &request_id,
            &graph,
            "POST",
            "/v1/files",
            next_log_attempt(&log_attempts),
            &response,
        );
        register_proxy_metrics(
            state.as_ref(),
            &graph,
            &response,
            "/v1/files",
            overall_start.elapsed().as_millis() as u64,
        );
    }

    Ok(response)
}

#[tracing::instrument(name = "handler.openai.v1.files.list", skip(state, auth, query))]
pub(crate) async fn list_files(
    State(state): State<Arc<LLMurState>>,
    Extension(auth): Extension<AuthorizationHeaderExtractionResult>,
    Extension(request_id): Extension<crate::data::request_log::RequestLogId>,
    Query(query): Query<ListFilesQuery>,
) -> Result<ProxyResponse<ListFilesResponse>, LLMurError> {
    if !is_virtual_list_request(&query.purpose) {
        return Err(LLMurError::BadRequest(
            "Files API is only supported for batch purpose.".to_string(),
        ));
    }

    list_virtual_files(state, auth, request_id, query).await
}

#[tracing::instrument(name = "handler.openai.v1.files.retrieve", skip(state, auth))]
pub(crate) async fn get_file(
    State(state): State<Arc<LLMurState>>,
    Extension(auth): Extension<AuthorizationHeaderExtractionResult>,
    Extension(request_id): Extension<crate::data::request_log::RequestLogId>,
    Path(file_id): Path<String>,
) -> Result<ProxyResponse<FileObject>, LLMurError> {
    let api_key = extract_api_key(auth.clone())?;
    if let Some(virtual_file) = fetch_virtual_file(&state, &api_key, &file_id).await? {
        return get_virtual_file(state, request_id, &api_key, virtual_file).await;
    }
    Err(LLMurError::BadRequest(
        "Files API is only supported for batch purpose.".to_string(),
    ))
}

#[tracing::instrument(name = "handler.openai.v1.files.delete", skip(state, auth))]
pub(crate) async fn delete_file(
    State(state): State<Arc<LLMurState>>,
    Extension(auth): Extension<AuthorizationHeaderExtractionResult>,
    Extension(request_id): Extension<crate::data::request_log::RequestLogId>,
    Path(file_id): Path<String>,
) -> Result<ProxyResponse<DeleteFileResponse>, LLMurError> {
    let api_key = extract_api_key(auth.clone())?;
    if let Some(virtual_file) = fetch_virtual_file(&state, &api_key, &file_id).await? {
        return delete_virtual_file(state, request_id, &api_key, virtual_file).await;
    }
    Err(LLMurError::BadRequest(
        "Files API is only supported for batch purpose.".to_string(),
    ))
}

#[tracing::instrument(name = "handler.openai.v1.files.content", skip(state, auth))]
pub(crate) async fn get_file_content(
    State(state): State<Arc<LLMurState>>,
    Extension(auth): Extension<AuthorizationHeaderExtractionResult>,
    Extension(request_id): Extension<crate::data::request_log::RequestLogId>,
    Path(file_id): Path<String>,
) -> Result<ProxyResponse<NoUsage>, LLMurError> {
    let api_key = extract_api_key(auth.clone())?;
    if let Some(virtual_file) = fetch_virtual_file(&state, &api_key, &file_id).await? {
        return get_virtual_file_content(state, request_id, &api_key, virtual_file).await;
    }
    Err(LLMurError::BadRequest(
        "Files API is only supported for batch purpose.".to_string(),
    ))
}

#[tracing::instrument(
    name = "handler.openai.v1.files.list_virtual",
    skip(state, auth, request_id, query)
)]
async fn list_virtual_files(
    state: Arc<LLMurState>,
    auth: AuthorizationHeaderExtractionResult,
    request_id: crate::data::request_log::RequestLogId,
    query: ListFilesQuery,
) -> Result<ProxyResponse<ListFilesResponse>, LLMurError> {
    let start = Instant::now();
    let start_ts = Utc::now();
    let api_key = extract_api_key(auth)?;
    let virtual_key_id = VirtualKeyId::from_decrypted_key(&api_key);

    let after = match &query.after {
        Some(value) => Some(parse_virtual_file_id(value)?),
        None => None,
    };
    let limit = query.limit.unwrap_or(10_000).max(1);
    let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
    let fetch_limit = limit_i64.saturating_add(1);
    let order = query.order.clone();

    let mut files = state
        .data
        .search_virtual_files(
            &virtual_key_id,
            &query.purpose,
            &Some(fetch_limit),
            &order,
            &after,
            &state.metrics,
        )
        .await?;

    let has_more = (files.len() as i64) > limit_i64;
    if has_more {
        files.truncate(limit as usize);
    }

    let graph = match files.first() {
        Some(file) => load_graph_for_virtual_file(state.as_ref(), &api_key, file).await?,
        None => None,
    };

    let data: Vec<FileObject> = files.iter().map(virtual_file_to_file_object).collect();
    let first_id = data.first().map(|file| file.id.clone()).unwrap_or_default();
    let last_id = data.last().map(|file| file.id.clone()).unwrap_or_default();

    let response = ProxyResponse::new(
        Ok(ProviderResponse::DecodedResponse {
            data: ListFilesResponse {
                object: ListObject::List,
                data,
                first_id,
                last_id,
                has_more,
            },
            status_code: reqwest::StatusCode::OK,
        }),
        start_ts,
    );

    if let Some(graph) = graph {
        log_proxy_response(
            state.as_ref(),
            &request_id,
            &graph,
            "GET",
            "/v1/files",
            0,
            &response,
        );
        register_proxy_metrics(
            state.as_ref(),
            &graph,
            &response,
            "/v1/files",
            start.elapsed().as_millis() as u64,
        );
    }

    Ok(response)
}

#[tracing::instrument(
    name = "handler.openai.v1.files.get_virtual",
    skip(state, request_id, file)
)]
async fn get_virtual_file(
    state: Arc<LLMurState>,
    request_id: crate::data::request_log::RequestLogId,
    api_key: &str,
    file: VirtualFile,
) -> Result<ProxyResponse<FileObject>, LLMurError> {
    let start = Instant::now();
    let start_ts = Utc::now();

    let response = ProxyResponse::new(
        Ok(ProviderResponse::DecodedResponse {
            data: virtual_file_to_file_object(&file),
            status_code: reqwest::StatusCode::OK,
        }),
        start_ts,
    );

    let graph = load_graph_for_virtual_file(state.as_ref(), api_key, &file).await?;
    if let Some(graph) = graph {
        let path = format!("/v1/files/{}", file.id.0);
        log_proxy_response(
            state.as_ref(),
            &request_id,
            &graph,
            "GET",
            &path,
            0,
            &response,
        );
        register_proxy_metrics(
            state.as_ref(),
            &graph,
            &response,
            &path,
            start.elapsed().as_millis() as u64,
        );
    }

    Ok(response)
}

#[tracing::instrument(
    name = "handler.openai.v1.files.delete_virtual",
    skip(state, request_id, file)
)]
async fn delete_virtual_file(
    state: Arc<LLMurState>,
    request_id: crate::data::request_log::RequestLogId,
    api_key: &str,
    file: VirtualFile,
) -> Result<ProxyResponse<DeleteFileResponse>, LLMurError> {
    let start = Instant::now();
    let start_ts = Utc::now();
    let log_attempts = Arc::new(AtomicI16::new(0));
    if let Some(provider_files) = file.provider_files.as_ref() {
        for provider_file in provider_files {
            let graph =
                load_graph_for_deployment_id(state.as_ref(), api_key, provider_file.deployment_id)
                    .await?;
            let (url, request_headers) = match &graph.connection.data.connection_info {
                ConnectionInfo::OpenAiApiKey {
                    api_key,
                    api_endpoint,
                    ..
                } => {
                    let url = format!(
                        "{}/v1/files/{}",
                        api_endpoint.trim_end_matches('/'),
                        provider_file.provider_file_id
                    );
                    let mut request_headers = ReqwestHeaderMap::new();
                    request_headers.insert(
                        "Authorization",
                        format!("Bearer {}", api_key).parse().unwrap(),
                    );
                    (url, request_headers)
                }
                ConnectionInfo::AzureOpenAiApiKey {
                    api_key,
                    api_endpoint,
                    api_version,
                    ..
                } => {
                    let api_base = azure_api_base(&api_endpoint);
                    let url = format!(
                        "{}/files/{}?api-version={}",
                        api_base, provider_file.provider_file_id, api_version
                    );
                    let mut request_headers = ReqwestHeaderMap::new();
                    request_headers.insert("api-key", api_key.parse().unwrap());
                    (url, request_headers)
                }
                ConnectionInfo::GeminiApiKey {
                    api_key,
                    api_endpoint,
                    api_version,
                    ..
                } => {
                    let api_base = api_endpoint.trim_end_matches('/');
                    let url = format!(
                        "{}/{}/{}",
                        api_base,
                        gemini_api_version(&api_version),
                        provider_file.provider_file_id
                    );
                    let mut request_headers = ReqwestHeaderMap::new();
                    request_headers.insert("x-goog-api-key", api_key.parse().unwrap());
                    (url, request_headers)
                }
            };

            let provider_start = Instant::now();
            let provider_ts = Utc::now();
            let request = state.data.http_client.delete(url).headers(request_headers);
            let response = proxy_request_json::<DeleteFileResponse>(request).await;
            let proxy_response = ProxyResponse::new(response, provider_ts);
            log_proxy_response(
                state.as_ref(),
                &request_id,
                &graph,
                "DELETE",
                "/v1/files",
                next_log_attempt(&log_attempts),
                &proxy_response,
            );
            register_proxy_metrics(
                state.as_ref(),
                &graph,
                &proxy_response,
                "/v1/files",
                provider_start.elapsed().as_millis() as u64,
            );

            if let Err(error) = proxy_response.result {
                return Ok(ProxyResponse::new(Err(error), provider_ts));
            }
        }
    }

    let _ = state
        .data
        .delete_virtual_file(&file.id, &state.metrics)
        .await?;

    let response = ProxyResponse::new(
        Ok(ProviderResponse::DecodedResponse {
            data: DeleteFileResponse {
                id: file.id.0.to_string(),
                object: FileObjectType::File,
                deleted: true,
            },
            status_code: reqwest::StatusCode::OK,
        }),
        start_ts,
    );

    let graph = load_graph_for_virtual_file(state.as_ref(), api_key, &file).await?;
    if let Some(graph) = graph {
        let path = format!("/v1/files/{}", file.id.0);
        log_proxy_response(
            state.as_ref(),
            &request_id,
            &graph,
            "DELETE",
            &path,
            next_log_attempt(&log_attempts),
            &response,
        );
        register_proxy_metrics(
            state.as_ref(),
            &graph,
            &response,
            &path,
            start.elapsed().as_millis() as u64,
        );
    }

    Ok(response)
}

#[tracing::instrument(
    name = "handler.openai.v1.files.content_virtual",
    skip(state, request_id, file)
)]
async fn get_virtual_file_content(
    state: Arc<LLMurState>,
    request_id: crate::data::request_log::RequestLogId,
    api_key: &str,
    file: VirtualFile,
) -> Result<ProxyResponse<NoUsage>, LLMurError> {
    let start = Instant::now();
    let start_ts = Utc::now();
    let log_attempts = Arc::new(AtomicI16::new(0));
    let provider_files = file.provider_files.as_ref().ok_or_else(|| {
        LLMurError::BadRequest("Virtual file does not have provider file references.".to_string())
    })?;
    let file_kind = file.kind.clone();
    // Output/error virtual files need endpoint context to apply provider-specific output
    // normalization before returning OpenAI-compatible content.
    let batch_endpoint = if matches!(file_kind, VirtualFileKind::Output | VirtualFileKind::Error) {
        match file.batch_id {
            Some(batch_id) => state
                .data
                .get_virtual_batch(&batch_id, &state.metrics)
                .await?
                .map(|batch| batch.endpoint),
            None => None,
        }
    } else {
        None
    };

    if file.line_map.is_none() {
        // Legacy/simple mode: no explicit mapping, so concatenate provider files in fetch order.
        let mut lines: Vec<String> = Vec::new();
        for provider_file in provider_files {
            let graph =
                load_graph_for_deployment_id(state.as_ref(), api_key, provider_file.deployment_id)
                    .await?;
            let content = fetch_provider_file_content(
                state.as_ref(),
                &graph,
                &request_id,
                &log_attempts,
                &provider_file.provider_file_id,
            )
            .await?;
            let content = maybe_transform_provider_content(
                &graph,
                content,
                &file_kind,
                batch_endpoint.as_ref(),
            )?;
            for line in content.lines() {
                lines.push(line.to_string());
            }
        }

        let mut content = lines.join("\n");
        if !content.is_empty() {
            content.push('\n');
        }

        let body = Body::from(content);
        let response = ProxyResponse::new(
            Ok(ProviderResponse::Stream {
                body: Arc::new(std::sync::Mutex::new(Some(body))),
                status_code: reqwest::StatusCode::OK,
                content_type: Some("text/plain; charset=utf-8".to_string()),
            }),
            start_ts,
        );

        let graph = load_graph_for_virtual_file(state.as_ref(), api_key, &file).await?;
        if let Some(graph) = graph {
            let path = format!("/v1/files/{}/content", file.id.0);
            log_proxy_response(
                state.as_ref(),
                &request_id,
                &graph,
                "GET",
                &path,
                next_log_attempt(&log_attempts),
                &response,
            );
            register_proxy_metrics(
                state.as_ref(),
                &graph,
                &response,
                &path,
                start.elapsed().as_millis() as u64,
            );
        }

        return Ok(response);
    }

    // Mapped mode: reconstruct exact original line order using persisted provider line mapping.
    let line_map = file.line_map.as_ref().ok_or_else(|| {
        LLMurError::BadRequest("Virtual file does not have content mapping.".to_string())
    })?;

    let mut provider_content: HashMap<String, Vec<String>> = HashMap::new();
    for provider_file in provider_files {
        let graph =
            load_graph_for_deployment_id(state.as_ref(), api_key, provider_file.deployment_id)
                .await?;
        let content = fetch_provider_file_content(
            state.as_ref(),
            &graph,
            &request_id,
            &log_attempts,
            &provider_file.provider_file_id,
        )
        .await?;
        let content =
            maybe_transform_provider_content(&graph, content, &file_kind, batch_endpoint.as_ref())?;
        let lines = content.lines().map(|line| line.to_string()).collect();
        provider_content.insert(provider_file.provider_file_id.clone(), lines);
    }

    let line_count = file
        .line_count
        .map(|count| count.max(0) as usize)
        .unwrap_or(line_map.len());
    let mut output: Vec<Option<String>> = vec![None; line_count];

    for entry in line_map {
        let line_index = usize::try_from(entry.line_index).map_err(|_| {
            LLMurError::BadRequest("Invalid line index in virtual file mapping.".to_string())
        })?;
        let provider_line_index = usize::try_from(entry.provider_line_index).map_err(|_| {
            LLMurError::BadRequest(
                "Invalid provider line index in virtual file mapping.".to_string(),
            )
        })?;

        let provider_lines = provider_content
            .get(&entry.provider_file_id)
            .ok_or_else(|| {
                LLMurError::BadRequest(
                    "Provider file content missing for virtual file reconstruction.".to_string(),
                )
            })?;
        let line = provider_lines.get(provider_line_index).ok_or_else(|| {
            LLMurError::BadRequest(
                "Provider line index is out of range for virtual file.".to_string(),
            )
        })?;

        if line_index >= output.len() {
            return Err(LLMurError::BadRequest(
                "Virtual file line index is out of range.".to_string(),
            ));
        }
        output[line_index] = Some(line.clone());
    }

    let mut content_lines = Vec::with_capacity(output.len());
    for (index, entry) in output.into_iter().enumerate() {
        let line = entry.ok_or_else(|| {
            LLMurError::BadRequest(format!(
                "Virtual file reconstruction missing line {}.",
                index + 1
            ))
        })?;
        content_lines.push(line);
    }

    let mut content = content_lines.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }

    let body = Body::from(content);
    let response = ProxyResponse::new(
        Ok(ProviderResponse::Stream {
            body: Arc::new(std::sync::Mutex::new(Some(body))),
            status_code: reqwest::StatusCode::OK,
            content_type: Some("text/plain; charset=utf-8".to_string()),
        }),
        start_ts,
    );

    let graph = load_graph_for_virtual_file(state.as_ref(), api_key, &file).await?;
    if let Some(graph) = graph {
        let path = format!("/v1/files/{}/content", file.id.0);
        log_proxy_response(
            state.as_ref(),
            &request_id,
            &graph,
            "GET",
            &path,
            next_log_attempt(&log_attempts),
            &response,
        );
        register_proxy_metrics(
            state.as_ref(),
            &graph,
            &response,
            &path,
            start.elapsed().as_millis() as u64,
        );
    }

    Ok(response)
}

fn is_virtual_list_request(purpose: &Option<FilePurpose>) -> bool {
    matches!(
        purpose,
        Some(FilePurpose::Batch) | Some(FilePurpose::BatchOutput)
    )
}

async fn fetch_virtual_file(
    state: &LLMurState,
    api_key: &str,
    file_id: &str,
) -> Result<Option<VirtualFile>, LLMurError> {
    let virtual_file_id = match file_id.parse::<VirtualFileId>() {
        Ok(id) => id,
        Err(_) => return Ok(None),
    };

    let virtual_key_id = VirtualKeyId::from_decrypted_key(api_key);
    let file = state
        .data
        .get_virtual_file(&virtual_file_id, &state.metrics)
        .await?;

    Ok(file.filter(|value| value.virtual_key_id == virtual_key_id))
}

fn virtual_file_to_file_object(file: &VirtualFile) -> FileObject {
    let status = match file.kind {
        VirtualFileKind::Input => FileStatus::Uploaded,
        VirtualFileKind::Output => FileStatus::Processed,
        VirtualFileKind::Error => FileStatus::Error,
    };

    FileObject {
        id: file.id.0.to_string(),
        bytes: file.bytes as u64,
        created_at: file.created_at.max(0) as u64,
        expires_at: file.expires_at.map(|value| value.max(0) as u64),
        filename: file.filename.clone(),
        object: FileObjectType::File,
        purpose: file.purpose.clone(),
        status,
        status_details: None,
    }
}

async fn load_graph_for_deployment_id(
    state: &LLMurState,
    api_key: &str,
    deployment_id: DeploymentId,
) -> Result<Graph, LLMurError> {
    let deployment = state
        .data
        .get_deployment(&deployment_id, &state.metrics)
        .await?
        .ok_or_else(|| {
            LLMurError::BadRequest("Deployment not found for virtual file request.".to_string())
        })?;

    let graph = state
        .data
        .get_graph(
            api_key,
            &deployment.name,
            false,
            10_000,
            &state.application_secret,
            &Utc::now(),
            &state.metrics,
        )
        .await
        .map_err(GraphError::from)?;
    validate_graph_usage(&graph).map_err(LLMurError::from)?;
    Ok(graph)
}

async fn load_graph_for_virtual_file(
    state: &LLMurState,
    api_key: &str,
    file: &VirtualFile,
) -> Result<Option<Graph>, LLMurError> {
    let provider_files = match file.provider_files.as_ref() {
        Some(files) if !files.is_empty() => files,
        _ => return Ok(None),
    };

    let deployment_id = provider_files[0].deployment_id;
    let graph = load_graph_for_deployment_id(state, api_key, deployment_id).await?;
    Ok(Some(graph))
}

async fn fetch_provider_file_content(
    state: &LLMurState,
    graph: &Graph,
    request_id: &crate::data::request_log::RequestLogId,
    log_attempts: &Arc<AtomicI16>,
    provider_file_id: &str,
) -> Result<String, LLMurError> {
    // Resolve provider-specific file-content endpoint/auth from the graph's active connection.
    let (url, request_headers) = match &graph.connection.data.connection_info {
        ConnectionInfo::OpenAiApiKey {
            api_key,
            api_endpoint,
            ..
        } => {
            let url = format!(
                "{}/v1/files/{}/content",
                api_endpoint.trim_end_matches('/'),
                provider_file_id
            );
            let mut request_headers = ReqwestHeaderMap::new();
            request_headers.insert(
                "Authorization",
                format!("Bearer {}", api_key).parse().unwrap(),
            );
            (url, request_headers)
        }
        ConnectionInfo::AzureOpenAiApiKey {
            api_key,
            api_endpoint,
            api_version,
            ..
        } => {
            let api_base = azure_api_base(&api_endpoint);
            let url = format!(
                "{}/files/{}/content?api-version={}",
                api_base, provider_file_id, api_version
            );
            let mut request_headers = ReqwestHeaderMap::new();
            request_headers.insert("api-key", api_key.parse().unwrap());
            (url, request_headers)
        }
        ConnectionInfo::GeminiApiKey {
            api_key,
            api_endpoint,
            api_version,
            ..
        } => {
            let api_base = api_endpoint.trim_end_matches('/');
            let url = format!(
                "{}/download/{}/{}:download?alt=media",
                api_base,
                gemini_api_version(&api_version),
                provider_file_id
            );
            let mut request_headers = ReqwestHeaderMap::new();
            request_headers.insert("x-goog-api-key", api_key.parse().unwrap());
            (url, request_headers)
        }
    };

    let start = Instant::now();
    let start_ts = Utc::now();
    let response = state
        .data
        .http_client
        .get(url)
        .headers(request_headers)
        .send()
        .await
        .map_err(ProxyError::from)?;

    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let body_bytes = response.bytes().await.map_err(ProxyError::from)?;

    if status.is_success() {
        let body = Body::from(body_bytes.clone());
        let proxy_response: ProxyResponse<NoUsage> = ProxyResponse::new(
            Ok(ProviderResponse::Stream {
                body: Arc::new(std::sync::Mutex::new(Some(body))),
                status_code: status,
                content_type,
            }),
            start_ts,
        );
        let path = format!("/v1/files/{}/content", provider_file_id);
        log_proxy_response(
            state,
            request_id,
            graph,
            "GET",
            &path,
            next_log_attempt(log_attempts),
            &proxy_response,
        );
        register_proxy_metrics(
            state,
            graph,
            &proxy_response,
            &path,
            start.elapsed().as_millis() as u64,
        );

        let content = String::from_utf8(body_bytes.to_vec()).map_err(|_| {
            LLMurError::BadRequest("Provider file content was not UTF-8.".to_string())
        })?;
        Ok(content)
    } else {
        // Preserve provider error payload shape (JSON when possible) for logs and client-facing
        // proxy error propagation.
        let error = match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
            Ok(value) => ProxyError::ProxyReturnError(status, ProxyErrorMessage::Json(value)),
            Err(_) => {
                let text = String::from_utf8_lossy(&body_bytes).to_string();
                ProxyError::ProxyReturnError(status, ProxyErrorMessage::Text(text))
            }
        };
        let proxy_response: ProxyResponse<NoUsage> = ProxyResponse::new(Err(error), start_ts);
        let path = format!("/v1/files/{}/content", provider_file_id);
        log_proxy_response(
            state,
            request_id,
            graph,
            "GET",
            &path,
            next_log_attempt(log_attempts),
            &proxy_response,
        );
        register_proxy_metrics(
            state,
            graph,
            &proxy_response,
            &path,
            start.elapsed().as_millis() as u64,
        );

        match proxy_response.result {
            Err(error) => Err(LLMurError::ProxyError(error)),
            Ok(_) => Err(LLMurError::BadRequest(
                "Unexpected provider response for file content.".to_string(),
            )),
        }
    }
}

fn maybe_transform_provider_content(
    graph: &Graph,
    content: String,
    file_kind: &VirtualFileKind,
    batch_endpoint: Option<&VirtualBatchEndpoint>,
) -> Result<String, LLMurError> {
    let Some(batch_endpoint) = batch_endpoint else {
        return Ok(content);
    };
    if !matches!(file_kind, VirtualFileKind::Output | VirtualFileKind::Error) {
        return Ok(content);
    }

    // Gemini batch output files are provider-native JSONL and must be translated back to
    // OpenAI-style line schema before line reconstruction/return.
    let transformed = match &graph.connection.data.connection_info {
        ConnectionInfo::GeminiApiKey { model, .. } => {
            transform_batch_output_content(&content, batch_endpoint, model)
        }
        _ => Ok(content),
    }?;

    Ok(normalize_openai_batch_output_content(
        &transformed,
        Some(graph.deployment.data.name.as_str()),
        Some(batch_endpoint),
        Some(&graph.connection.data.connection_info),
    ))
}

fn normalize_batch_response_body_strict(
    body: JsonValue,
    connection_info: &ConnectionInfo,
    batch_endpoint: &VirtualBatchEndpoint,
    deployment_model_name: &str,
) -> Result<JsonValue, String> {
    // Strict mode reparses provider bodies through the canonical OpenAI response structs, which
    // strips provider-only fields and enforces deployment-model override behavior.
    match batch_endpoint {
        VirtualBatchEndpoint::ChatCompletions => match connection_info {
            ConnectionInfo::AzureOpenAiApiKey { .. } => {
                let response: AzureChatCompletionsResponse =
                    serde_json::from_value(body).map_err(|err| {
                        format!("azure chat-completions body deserialize failed: {}", err)
                    })?;
                let transformed = response
                    .transform(AzureChatCompletionsResponseContext {
                        model: Some(deployment_model_name.to_string()),
                    })
                    .result;
                serde_json::to_value(transformed)
                    .map_err(|err| format!("azure chat-completions body serialize failed: {}", err))
            }
            _ => {
                let response: OpenAiChatCompletionsResponse = serde_json::from_value(body)
                    .map_err(|err| {
                        format!("openai chat-completions body deserialize failed: {}", err)
                    })?;
                let transformed = response
                    .transform(OpenAiChatCompletionsResponseContext {
                        model: Some(deployment_model_name.to_string()),
                    })
                    .result;
                serde_json::to_value(transformed).map_err(|err| {
                    format!("openai chat-completions body serialize failed: {}", err)
                })
            }
        },
        VirtualBatchEndpoint::Responses => {
            let response: OpenAiResponsesResponse = serde_json::from_value(body)
                .map_err(|err| format!("openai responses body deserialize failed: {}", err))?;
            let transformed = response
                .transform(OpenAiResponsesResponseContext {
                    model: Some(deployment_model_name.to_string()),
                })
                .result;
            serde_json::to_value(transformed)
                .map_err(|err| format!("openai responses body serialize failed: {}", err))
        }
        VirtualBatchEndpoint::Embeddings => match connection_info {
            ConnectionInfo::AzureOpenAiApiKey { .. } => {
                let response: AzureEmbeddingsResponse = serde_json::from_value(body)
                    .map_err(|err| format!("azure embeddings body deserialize failed: {}", err))?;
                let transformed = response
                    .transform(AzureEmbeddingsResponseContext {
                        model: Some(deployment_model_name.to_string()),
                    })
                    .result;
                serde_json::to_value(transformed)
                    .map_err(|err| format!("azure embeddings body serialize failed: {}", err))
            }
            _ => {
                let response: OpenAiEmbeddingsResponse = serde_json::from_value(body)
                    .map_err(|err| format!("openai embeddings body deserialize failed: {}", err))?;
                let transformed = response
                    .transform(OpenAiEmbeddingsResponseContext {
                        model: Some(deployment_model_name.to_string()),
                    })
                    .result;
                serde_json::to_value(transformed)
                    .map_err(|err| format!("openai embeddings body serialize failed: {}", err))
            }
        },
        VirtualBatchEndpoint::Completions => {
            let response: OpenAiCompletionsResponse = serde_json::from_value(body)
                .map_err(|err| format!("openai completions body deserialize failed: {}", err))?;
            let transformed = response
                .transform(OpenAiCompletionsResponseContext {
                    model: Some(deployment_model_name.to_string()),
                })
                .result;
            serde_json::to_value(transformed)
                .map_err(|err| format!("openai completions body serialize failed: {}", err))
        }
    }
}

fn override_body_model(mut body: JsonValue, deployment_model_name: &str) -> JsonValue {
    if let Some(response_body) = body.as_object_mut() {
        response_body.insert(
            "model".to_string(),
            JsonValue::String(deployment_model_name.to_string()),
        );
    }
    body
}

fn ensure_usage_detail_field(details: &mut serde_json::Map<String, JsonValue>, key: &str) {
    let should_default = match details.get(key) {
        Some(value) => value.is_null(),
        None => true,
    };
    if should_default {
        details.insert(key.to_string(), JsonValue::from(0_u64));
    }
}

fn normalize_chat_completions_usage_details(body: &mut JsonValue) {
    let Some(body_obj) = body.as_object_mut() else {
        return;
    };
    let Some(usage_obj) = body_obj
        .get_mut("usage")
        .and_then(serde_json::Value::as_object_mut)
    else {
        return;
    };

    let completion_details = usage_obj
        .entry("completion_tokens_details".to_string())
        .or_insert_with(|| JsonValue::Object(serde_json::Map::new()));
    if let Some(completion_obj) = completion_details.as_object_mut() {
        ensure_usage_detail_field(completion_obj, "accepted_prediction_tokens");
        ensure_usage_detail_field(completion_obj, "audio_tokens");
        ensure_usage_detail_field(completion_obj, "reasoning_tokens");
        ensure_usage_detail_field(completion_obj, "rejected_prediction_tokens");
    }

    let prompt_details = usage_obj
        .entry("prompt_tokens_details".to_string())
        .or_insert_with(|| JsonValue::Object(serde_json::Map::new()));
    if let Some(prompt_obj) = prompt_details.as_object_mut() {
        ensure_usage_detail_field(prompt_obj, "audio_tokens");
        ensure_usage_detail_field(prompt_obj, "cached_tokens");
    }
}

fn normalize_openai_batch_output_content(
    content: &str,
    deployment_model_name: Option<&str>,
    batch_endpoint: Option<&VirtualBatchEndpoint>,
    connection_info: Option<&ConnectionInfo>,
) -> String {
    let mut output_lines: Vec<String> = Vec::new();

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut normalized_line = trimmed.to_string();
        if let Ok(mut value) = serde_json::from_str::<JsonValue>(trimmed) {
            let custom_id = value
                .get("custom_id")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());

            if let Some(custom_id) = custom_id
                && let Some(object) = value.as_object_mut()
            {
                let has_nonempty_id = object
                    .get("id")
                    .and_then(|value| value.as_str())
                    .map(|value| !value.trim().is_empty())
                    .unwrap_or(false);
                if !has_nonempty_id {
                    object.insert(
                        "id".to_string(),
                        JsonValue::String(format!("batch_req_{}", custom_id)),
                    );
                }
                object
                    .entry("response".to_string())
                    .or_insert(JsonValue::Null);
                object.entry("error".to_string()).or_insert(JsonValue::Null);
            }

            if let Some(response_body) = value
                .get_mut("response")
                .and_then(|response| response.as_object_mut())
                .and_then(|response| response.get_mut("body"))
            {
                let original_body = response_body.clone();
                // Prefer strict normalization when context is available; fall back to non-failing
                // model override so retrieval never hard-fails on shape drift.
                let normalized_body = match (deployment_model_name, batch_endpoint, connection_info)
                {
                    (Some(deployment_model_name), Some(batch_endpoint), Some(connection_info)) => {
                        match normalize_batch_response_body_strict(
                            original_body.clone(),
                            connection_info,
                            batch_endpoint,
                            deployment_model_name,
                        ) {
                            Ok(body) => body,
                            Err(error) => {
                                tracing::debug!(
                                    provider = connection_info.get_provider_friendly_name(),
                                    endpoint = ?batch_endpoint,
                                    %error,
                                    "Strict batch response body normalization failed; falling back to non-failing body normalization."
                                );
                                override_body_model(original_body, deployment_model_name)
                            }
                        }
                    }
                    (Some(deployment_model_name), _, _) => {
                        override_body_model(original_body, deployment_model_name)
                    }
                    _ => original_body,
                };
                *response_body = normalized_body;
                if matches!(batch_endpoint, Some(VirtualBatchEndpoint::ChatCompletions)) {
                    normalize_chat_completions_usage_details(response_body);
                }
            }

            if let Ok(serialized) = serde_json::to_string(&value) {
                normalized_line = serialized;
            }
        }

        output_lines.push(normalized_line);
    }

    let mut normalized = output_lines.join("\n");
    if !normalized.is_empty() {
        normalized.push('\n');
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::normalize_openai_batch_output_content;
    use crate::data::connection::{AzureOpenAiApiVersion, ConnectionInfo};
    use crate::data::virtual_batch::VirtualBatchEndpoint;
    use serde_json::Value as JsonValue;

    #[test]
    fn normalize_openai_batch_output_content_adds_missing_batch_fields() {
        let content = "{\"custom_id\":\"req-1\",\"response\":{\"status_code\":200,\"body\":{\"id\":\"abc\"}},\"error\":null}\n\n";
        let normalized = normalize_openai_batch_output_content(content, None, None, None);
        let line = normalized.lines().next().unwrap();
        let value: JsonValue = serde_json::from_str(line).unwrap();

        assert_eq!(
            value.get("id").and_then(|value| value.as_str()),
            Some("batch_req_req-1")
        );
        assert!(value.get("response").is_some());
        assert!(value.get("error").is_some());
    }

    #[test]
    fn normalize_openai_batch_output_content_overrides_response_model() {
        let content = "{\"id\":\"batch_req_req-1\",\"custom_id\":\"req-1\",\"response\":{\"status_code\":200,\"body\":{\"id\":\"chatcmpl-1\",\"model\":\"provider-model\"}},\"error\":null}\n";
        let normalized =
            normalize_openai_batch_output_content(content, Some("deployment-model"), None, None);
        let line = normalized.lines().next().unwrap();
        let value: JsonValue = serde_json::from_str(line).unwrap();

        let model = value
            .get("response")
            .and_then(|value| value.get("body"))
            .and_then(|value| value.get("model"))
            .and_then(|value| value.as_str());
        assert_eq!(model, Some("deployment-model"));
    }

    #[test]
    fn normalize_openai_batch_output_content_strips_azure_extra_fields_when_strict_enabled() {
        let content = "{\"custom_id\":\"req-1\",\"response\":{\"status_code\":200,\"body\":{\"id\":\"chatcmpl-1\",\"object\":\"chat.completion\",\"created\":1,\"model\":\"azure-provider-model\",\"choices\":[{\"finish_reason\":\"stop\",\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":\"hi\"},\"logprobs\":null,\"content_filter_results\":{\"hate\":{\"filtered\":false,\"severity\":\"safe\"}}}],\"usage\":{\"completion_tokens\":1,\"prompt_tokens\":1,\"total_tokens\":2},\"prompt_filter_results\":[{\"prompt_index\":0,\"content_filter_results\":{\"hate\":{\"filtered\":false,\"severity\":\"safe\"}}}] }},\"error\":null}\n";
        let connection_info = ConnectionInfo::AzureOpenAiApiKey {
            api_key: "key".to_string(),
            api_endpoint: "https://example.openai.azure.com".to_string(),
            api_version: AzureOpenAiApiVersion::V1,
            deployment_name: "azure-deployment".to_string(),
        };

        let normalized = normalize_openai_batch_output_content(
            content,
            Some("deployment-model"),
            Some(&VirtualBatchEndpoint::ChatCompletions),
            Some(&connection_info),
        );
        let line = normalized.lines().next().unwrap();
        let value: JsonValue = serde_json::from_str(line).unwrap();

        let body = value
            .get("response")
            .and_then(|value| value.get("body"))
            .expect("body");
        assert_eq!(
            body.get("model").and_then(|value| value.as_str()),
            Some("deployment-model")
        );
        assert!(
            body.get("prompt_filter_results").is_none(),
            "strict normalization should remove Azure-only prompt filter metadata"
        );
        assert!(
            body.get("choices")
                .and_then(|choices| choices.get(0))
                .and_then(|choice| choice.get("content_filter_results"))
                .is_none(),
            "strict normalization should remove Azure-only choice filter metadata"
        );
    }

    #[test]
    fn normalize_openai_batch_output_content_non_failing_when_body_parse_fails() {
        let content = "{\"custom_id\":\"req-1\",\"response\":{\"status_code\":200,\"body\":{\"unexpected\":\"shape\"}},\"error\":null}\n";
        let connection_info = ConnectionInfo::OpenAiApiKey {
            api_key: "key".to_string(),
            api_endpoint: "https://api.openai.com".to_string(),
            model: "gpt-4o-mini".to_string(),
        };

        let normalized = normalize_openai_batch_output_content(
            content,
            Some("deployment-model"),
            Some(&VirtualBatchEndpoint::ChatCompletions),
            Some(&connection_info),
        );
        let line = normalized.lines().next().unwrap();
        let value: JsonValue = serde_json::from_str(line).unwrap();

        let body = value
            .get("response")
            .and_then(|value| value.get("body"))
            .expect("body");
        assert_eq!(
            body.get("unexpected").and_then(|value| value.as_str()),
            Some("shape")
        );
        assert_eq!(
            body.get("model").and_then(|value| value.as_str()),
            Some("deployment-model")
        );
    }

    #[test]
    fn normalize_openai_batch_output_content_backfills_usage_detail_fields_for_chat_completions() {
        let content = "{\"custom_id\":\"req-1\",\"response\":{\"status_code\":200,\"body\":{\"id\":\"chatcmpl-1\",\"object\":\"chat.completion\",\"created\":1,\"model\":\"provider-model\",\"choices\":[{\"finish_reason\":\"stop\",\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":\"hi\"},\"logprobs\":null}],\"usage\":{\"completion_tokens\":1,\"completion_tokens_details\":{\"reasoning_tokens\":2},\"prompt_tokens\":3,\"total_tokens\":4}}},\"error\":null}\n";
        let connection_info = ConnectionInfo::OpenAiApiKey {
            api_key: "key".to_string(),
            api_endpoint: "https://api.openai.com".to_string(),
            model: "gpt-4o-mini".to_string(),
        };

        let normalized = normalize_openai_batch_output_content(
            content,
            Some("deployment-model"),
            Some(&VirtualBatchEndpoint::ChatCompletions),
            Some(&connection_info),
        );
        let line = normalized.lines().next().unwrap();
        let value: JsonValue = serde_json::from_str(line).unwrap();
        let usage = value
            .get("response")
            .and_then(|value| value.get("body"))
            .and_then(|value| value.get("usage"))
            .expect("usage");

        assert_eq!(
            usage["completion_tokens_details"]["accepted_prediction_tokens"],
            0
        );
        assert_eq!(usage["completion_tokens_details"]["audio_tokens"], 0);
        assert_eq!(usage["completion_tokens_details"]["reasoning_tokens"], 2);
        assert_eq!(
            usage["completion_tokens_details"]["rejected_prediction_tokens"],
            0
        );
        assert_eq!(usage["prompt_tokens_details"]["audio_tokens"], 0);
        assert_eq!(usage["prompt_tokens_details"]["cached_tokens"], 0);
    }
}

fn parse_batch_lines(content: &str) -> Result<Vec<BatchInputLine>, LLMurError> {
    let mut lines = Vec::new();
    for (index, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim();
        // Empty lines are rejected so virtual/provider line indices stay 1:1 stable.
        if trimmed.is_empty() {
            return Err(LLMurError::BadRequest(format!(
                "Batch line {} is empty.",
                index + 1
            )));
        }
        let line: BatchInputLine = serde_json::from_str(trimmed).map_err(|err| {
            LLMurError::BadRequest(format!(
                "Batch line {} is not valid JSON: {}",
                index + 1,
                err
            ))
        })?;
        lines.push(line);
    }
    Ok(lines)
}

async fn load_graph_for_deployment(
    state: &LLMurState,
    api_key: &str,
    deployment_name: &str,
    cache: &mut HashMap<String, Graph>,
) -> Result<Graph, LLMurError> {
    // Batch file parsing can reference the same deployment repeatedly; cache avoids repeated graph
    // lookups and repeated key/deployment limit validation for identical deployment names.
    if let Some(graph) = cache.get(deployment_name) {
        return Ok(graph.clone());
    }

    let graph = state
        .data
        .get_graph(
            api_key,
            deployment_name,
            false,
            10_000,
            &state.application_secret,
            &Utc::now(),
            &state.metrics,
        )
        .await
        .map_err(GraphError::from)?;
    validate_graph_usage(&graph).map_err(LLMurError::from)?;

    cache.insert(deployment_name.to_string(), graph.clone());
    Ok(graph)
}

fn azure_api_base(api_endpoint: &str) -> String {
    let api_base = api_endpoint.trim_end_matches('/');
    if api_base.ends_with("/openai/v1") {
        api_base.to_string()
    } else {
        format!("{}/openai/v1", api_base)
    }
}

fn gemini_api_version(api_version: &GeminiApiVersion) -> &'static str {
    match api_version {
        GeminiApiVersion::V1BETA => "v1beta",
    }
}

fn validate_graph_usage(graph: &Graph) -> Result<(), GraphError> {
    graph.virtual_key.validate_limits()?;
    graph.virtual_key_deployment.validate_limits()?;
    graph.project.validate_limits()?;
    graph.deployment.validate_limits()?;
    Ok(())
}

fn next_log_attempt(counter: &Arc<AtomicI16>) -> i16 {
    counter.fetch_add(1, Ordering::Relaxed)
}

async fn parse_multipart_upload(multipart: &mut Multipart) -> Result<ParsedFileUpload, LLMurError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut purpose_value: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| LLMurError::BadRequest(err.to_string()))?
    {
        let name = match field.name() {
            Some(value) => value,
            None => continue,
        };

        match name {
            "file" => {
                let name = field
                    .file_name()
                    .map(|value| value.to_string())
                    .ok_or_else(|| {
                        LLMurError::BadRequest("File upload requires a filename".to_string())
                    })?;
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|err| LLMurError::BadRequest(err.to_string()))?;
                file_name = Some(name);
                file_bytes = Some(bytes.to_vec());
            }
            "purpose" => {
                let value = field
                    .text()
                    .await
                    .map_err(|err| LLMurError::BadRequest(err.to_string()))?;
                purpose_value = Some(value);
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    let file_bytes = file_bytes
        .ok_or_else(|| LLMurError::BadRequest("File upload requires a file field".to_string()))?;
    let file_name = file_name
        .ok_or_else(|| LLMurError::BadRequest("File upload requires a filename".to_string()))?;
    let purpose_value = purpose_value.ok_or_else(|| {
        LLMurError::BadRequest("File upload requires a purpose field".to_string())
    })?;

    let purpose: FilePurpose = purpose_value.parse().map_err(LLMurError::BadRequest)?;

    Ok(ParsedFileUpload {
        filename: file_name,
        bytes: file_bytes,
        purpose,
    })
}
