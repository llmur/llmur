use crate::LLMurState;
use crate::data::connection::{AzureOpenAiApiVersion, ConnectionInfo, GeminiApiVersion};
use crate::data::deployment::DeploymentId;
use crate::data::graph::NodeLimitsChecker;
use crate::data::provider_batch::{ProviderBatch, ProviderBatchId};
use crate::data::virtual_batch::{
    BatchRequestCounts as VirtualBatchRequestCounts, ListOrder, VirtualBatch,
    VirtualBatchCompletionWindow, VirtualBatchEndpoint, VirtualBatchId, VirtualBatchStatus,
};
use crate::data::virtual_file::{
    ProviderFileReference, VirtualFile, VirtualFileId, VirtualFileKind, VirtualFileLineMapEntry,
};
use crate::data::virtual_key::VirtualKeyId;
use crate::errors::{GraphError, LLMurError, ProxyError};
use crate::metrics::RegisterBatchNormalization;
use crate::providers::azure::openai::v1::batches::request::CreateBatchRequest as AzureCreateBatchRequest;
use crate::providers::azure::openai::v1::batches::types as AzureBatchTypes;
use crate::providers::azure::openai::v1::batches::types::BatchObject as AzureBatchObject;
use crate::providers::azure::openai::v1::files::types::FileObject as AzureFileObject;
use crate::providers::gemini::v1beta::batches::request::{
    BatchCreateBody as GeminiBatchCreateBody, BatchCreateRequest as GeminiBatchCreateRequest,
    BatchInputConfig as GeminiBatchInputConfig,
};
use crate::providers::gemini::v1beta::batches::response::{
    BatchResponse as GeminiBatchResponse,
    batch_object_from_value as gemini_batch_object_from_value, normalize_gemini_file_reference,
};
use crate::providers::gemini::v1beta::files::types::{
    FileResource as GeminiFileResource, file_resource_to_openai,
};
use crate::providers::openai::batches::request::{CreateBatchRequest, ListBatchesQuery};
use crate::providers::openai::batches::response::{ListBatchesResponse, ListObject};
use crate::providers::openai::batches::types::{
    BatchEndpoint, BatchObject, BatchObjectType, BatchRequestCounts, BatchStatus, CompletionWindow,
};
use crate::providers::openai::files::types::{FileObject, FilePurpose};
use crate::routes::middleware::auth::AuthorizationHeaderExtractionResult;
use crate::routes::openai::proxy_helpers::{
    log_proxy_response, proxy_request_json, register_proxy_metrics,
};
use crate::routes::openai::response::{ProviderResponse, ProxyResponse};
use crate::routes::utils::{extract_api_key, parse_virtual_batch_id, parse_virtual_file_id};
use axum::Extension;
use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use log::info;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI16, Ordering};
use std::time::Instant;
use uuid::Uuid;

#[tracing::instrument(
    name = "handler.openai.v1.batches.create",
    skip(state, auth, request_id, payload)
)]
pub(crate) async fn create_batch(
    State(state): State<Arc<LLMurState>>,
    Extension(auth): Extension<AuthorizationHeaderExtractionResult>,
    Extension(request_id): Extension<crate::data::request_log::RequestLogId>,
    Json(payload): Json<CreateBatchRequest>,
) -> Result<ProxyResponse<BatchObject>, LLMurError> {
    info!("Started create batch handler");
    let start = Instant::now();
    let start_ts = Utc::now();
    let log_attempts = Arc::new(AtomicI16::new(0));
    let api_key = extract_api_key(auth)?;
    let virtual_key_id = VirtualKeyId::from_decrypted_key(&api_key);

    let input_file_id = parse_virtual_file_id(&payload.input_file_id)?;
    let input_file = state
        .data
        .get_virtual_file(&input_file_id, &state.metrics)
        .await?
        .ok_or_else(|| LLMurError::BadRequest("Input file was not found.".to_string()))?;
    // Guardrails: the uploaded input file must belong to the caller's virtual key and be a
    // batch-purpose file previously expanded into provider-specific files.
    if input_file.virtual_key_id != virtual_key_id {
        return Err(LLMurError::BadRequest(
            "Input file does not belong to this virtual key.".to_string(),
        ));
    }
    if input_file.purpose != FilePurpose::Batch {
        return Err(LLMurError::BadRequest(
            "Input file is not a batch file.".to_string(),
        ));
    }

    let provider_files = input_file.provider_files.as_ref().ok_or_else(|| {
        LLMurError::BadRequest("Input file is missing provider file references.".to_string())
    })?;
    if provider_files.is_empty() {
        return Err(LLMurError::BadRequest(
            "Input file is missing provider file references.".to_string(),
        ));
    }

    let mut provider_batches: Vec<ProviderBatchCreate> = Vec::new();
    let expected_provider_request_totals = expected_provider_request_totals(&input_file);

    for provider_file in provider_files {
        // Fan out batch creation per provider/deployment using the provider-specific input file id
        // produced at file-upload time.
        let graph =
            load_graph_for_deployment_id(state.as_ref(), &api_key, provider_file.deployment_id)
                .await?;
        let connection = provider_connection(&graph, 0)?;
        let mut batch_object = match connection {
            ProviderConnection::OpenAi {
                api_key,
                api_endpoint,
            } => {
                let request_payload = AzureCreateBatchRequest {
                    input_file_id: provider_file.provider_file_id.clone(),
                    endpoint: payload.endpoint.clone(),
                    completion_window: payload.completion_window.clone(),
                    metadata: payload.metadata.clone(),
                };
                let url = format!("{}/v1/batches", api_endpoint.trim_end_matches('/'));
                let mut request_headers = reqwest::header::HeaderMap::new();
                request_headers.insert(
                    "Authorization",
                    format!("Bearer {}", api_key).parse().unwrap(),
                );
                request_headers.insert("Content-Type", "application/json".parse().unwrap());

                let provider_start = Instant::now();
                let provider_ts = Utc::now();
                let request = state
                    .data
                    .http_client
                    .post(url)
                    .headers(request_headers)
                    .json(&request_payload);
                let response = proxy_request_json::<BatchObject>(request).await;
                let proxy_response = ProxyResponse::new(response, provider_ts);

                log_proxy_response(
                    state.as_ref(),
                    &request_id,
                    &graph,
                    "POST",
                    "/v1/batches",
                    next_log_attempt(&log_attempts),
                    &proxy_response,
                );
                register_proxy_metrics(
                    state.as_ref(),
                    &graph,
                    &proxy_response,
                    "/v1/batches",
                    provider_start.elapsed().as_millis() as u64,
                );

                match proxy_response.result {
                    Ok(ProviderResponse::DecodedResponse { data, .. }) => data,
                    Ok(ProviderResponse::JsonResponse { .. }) => {
                        return Ok(ProxyResponse::new(
                            Err(ProxyError::InternalError(
                                "Batches API returned an unexpected response.".to_string(),
                            )),
                            provider_ts,
                        ));
                    }
                    Ok(ProviderResponse::Stream { .. }) => {
                        return Ok(ProxyResponse::new(
                            Err(ProxyError::InternalError(
                                "Batches API returned an unexpected stream response.".to_string(),
                            )),
                            provider_ts,
                        ));
                    }
                    Err(error) => return Ok(ProxyResponse::new(Err(error), provider_ts)),
                }
            }
            ProviderConnection::Azure {
                api_key,
                api_endpoint,
                api_version,
            } => {
                if !azure_batch_endpoint_supported(&payload.endpoint) {
                    return Err(LLMurError::BadRequest(
                        "Azure OpenAI batches support /v1/chat/completions and /v1/embeddings only."
                            .to_string(),
                    ));
                }
                let request_payload = AzureCreateBatchRequest {
                    input_file_id: provider_file.provider_file_id.clone(),
                    endpoint: payload.endpoint.clone(),
                    completion_window: payload.completion_window.clone(),
                    metadata: payload.metadata.clone(),
                };
                let api_base = azure_api_base(&api_endpoint);
                let url = format!("{}/batches?api-version={}", api_base, api_version);
                let mut request_headers = reqwest::header::HeaderMap::new();
                request_headers.insert("api-key", api_key.parse().unwrap());
                request_headers.insert("Content-Type", "application/json".parse().unwrap());

                let provider_start = Instant::now();
                let provider_ts = Utc::now();
                let request = state
                    .data
                    .http_client
                    .post(url)
                    .headers(request_headers)
                    .json(&request_payload);
                let response = proxy_request_json::<BatchObject>(request).await;
                let proxy_response = ProxyResponse::new(response, provider_ts);

                log_proxy_response(
                    state.as_ref(),
                    &request_id,
                    &graph,
                    "POST",
                    "/v1/batches",
                    next_log_attempt(&log_attempts),
                    &proxy_response,
                );
                register_proxy_metrics(
                    state.as_ref(),
                    &graph,
                    &proxy_response,
                    "/v1/batches",
                    provider_start.elapsed().as_millis() as u64,
                );

                match proxy_response.result {
                    Ok(ProviderResponse::DecodedResponse { data, .. }) => data,
                    Ok(ProviderResponse::JsonResponse { data, .. }) => {
                        match AzureBatchTypes::normalize_batch_response(
                            data,
                            &payload.endpoint,
                            &payload.completion_window,
                        ) {
                            Ok(normalized) => normalized,
                            Err(_err) => {
                                return Ok(ProxyResponse::new(
                                    Err(ProxyError::InternalError(
                                        "Batches API returned an unexpected response.".to_string(),
                                    )),
                                    provider_ts,
                                ));
                            }
                        }
                    }
                    Ok(ProviderResponse::Stream { .. }) => {
                        return Ok(ProxyResponse::new(
                            Err(ProxyError::InternalError(
                                "Batches API returned an unexpected stream response.".to_string(),
                            )),
                            provider_ts,
                        ));
                    }
                    Err(error) => return Ok(ProxyResponse::new(Err(error), provider_ts)),
                }
            }
            ProviderConnection::Gemini {
                api_key,
                api_endpoint,
                api_version,
                model,
            } => {
                if !gemini_batch_endpoint_supported(&payload.endpoint) {
                    return Err(LLMurError::BadRequest(
                        "Gemini batches support /v1/chat/completions and /v1/responses only."
                            .to_string(),
                    ));
                }
                let display_name = payload
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.get("display_name").cloned());
                let request_payload = GeminiBatchCreateRequest {
                    batch: GeminiBatchCreateBody {
                        display_name,
                        input_config: GeminiBatchInputConfig {
                            file_name: provider_file.provider_file_id.clone(),
                        },
                    },
                };
                let api_base = api_endpoint.trim_end_matches('/');
                let url = format!(
                    "{}/{}/models/{}:batchGenerateContent",
                    api_base,
                    gemini_api_version(&api_version),
                    model
                );
                let mut request_headers = reqwest::header::HeaderMap::new();
                request_headers.insert("x-goog-api-key", api_key.parse().unwrap());
                request_headers.insert("Content-Type", "application/json".parse().unwrap());

                let provider_start = Instant::now();
                let provider_ts = Utc::now();
                let request = state
                    .data
                    .http_client
                    .post(url)
                    .headers(request_headers)
                    .json(&request_payload);
                let response = proxy_request_json::<GeminiBatchResponse>(request).await;
                let proxy_response = ProxyResponse::new(response, provider_ts);

                log_proxy_response(
                    state.as_ref(),
                    &request_id,
                    &graph,
                    "POST",
                    "/v1/batches",
                    next_log_attempt(&log_attempts),
                    &proxy_response,
                );
                register_proxy_metrics(
                    state.as_ref(),
                    &graph,
                    &proxy_response,
                    "/v1/batches",
                    provider_start.elapsed().as_millis() as u64,
                );

                let response_value = match proxy_response.result {
                    Ok(ProviderResponse::DecodedResponse { data, .. }) => data.value,
                    Ok(ProviderResponse::JsonResponse { .. }) => {
                        return Ok(ProxyResponse::new(
                            Err(ProxyError::InternalError(
                                "Batches API returned an unexpected response.".to_string(),
                            )),
                            provider_ts,
                        ));
                    }
                    Ok(ProviderResponse::Stream { .. }) => {
                        return Ok(ProxyResponse::new(
                            Err(ProxyError::InternalError(
                                "Batches API returned an unexpected stream response.".to_string(),
                            )),
                            provider_ts,
                        ));
                    }
                    Err(error) => return Ok(ProxyResponse::new(Err(error), provider_ts)),
                };

                gemini_batch_object_from_value(
                    response_value,
                    &payload.endpoint,
                    &payload.completion_window,
                    &provider_file.provider_file_id,
                )?
            }
        };

        let provider_name = graph
            .connection
            .data
            .connection_info
            .get_provider_friendly_name()
            .to_string();

        // Keep request-count totals anchored to mapped input lines; providers can temporarily
        // report incomplete or inflated counters during lifecycle transitions.
        let expected_total = expected_provider_request_totals
            .get(&(provider_name.clone(), provider_file.deployment_id))
            .copied();
        batch_object.request_counts = normalize_request_counts_for_batch_status(
            &batch_object.status,
            &batch_object.request_counts,
            expected_total,
        );

        provider_batches.push(ProviderBatchCreate {
            graph,
            provider: provider_name,
            deployment_id: provider_file.deployment_id,
            batch_object,
        });
    }

    // Persist one virtual batch that aggregates all provider batches, then persist each provider
    // backing batch for future polling/cancel reconciliation.
    let virtual_batch_id = VirtualBatchId(Uuid::now_v7());
    let first_graph = provider_batches.first().map(|batch| batch.graph.clone());
    let status = aggregate_virtual_status(
        &provider_batches
            .iter()
            .map(|batch| (&batch.batch_object.status).into())
            .collect::<Vec<_>>(),
    );
    let request_counts = aggregate_virtual_request_counts(
        provider_batches
            .iter()
            .map(|batch| batch_request_counts_to_virtual(&batch.batch_object.request_counts))
            .collect::<Vec<_>>(),
    );
    let timestamps =
        aggregate_virtual_timestamps(provider_batches.iter().map(|batch| &batch.batch_object));

    let virtual_batch = state
        .data
        .create_virtual_batch(
            &virtual_batch_id,
            &virtual_key_id,
            &input_file_id,
            &(&payload.endpoint).into(),
            &(&payload.completion_window).into(),
            &status,
            &request_counts,
            &payload.metadata,
            &None,
            &None,
            &timestamps.in_progress_at,
            &timestamps.expires_at,
            &timestamps.finalizing_at,
            &timestamps.completed_at,
            &timestamps.failed_at,
            &timestamps.expired_at,
            &timestamps.cancelling_at,
            &timestamps.cancelled_at,
            &state.metrics,
        )
        .await?;

    for batch in provider_batches {
        let provider_batch_id = ProviderBatchId(Uuid::now_v7());
        let _ = state
            .data
            .create_provider_batch(
                &provider_batch_id,
                &virtual_batch_id,
                batch.provider.as_str(),
                &batch.deployment_id,
                &batch.batch_object.id,
                &(&batch.batch_object.status).into(),
                &batch_request_counts_to_virtual(&batch.batch_object.request_counts),
                &batch.batch_object.output_file_id,
                &batch.batch_object.error_file_id,
                &state.metrics,
            )
            .await?;
    }

    let response = ProxyResponse::new(
        Ok(ProviderResponse::DecodedResponse {
            data: normalize_batch_for_openai_surface_with_metrics(
                virtual_batch_to_batch_object(&virtual_batch),
                &state.metrics,
            ),
            status_code: reqwest::StatusCode::OK,
        }),
        start_ts,
    );

    if let Some(graph) = first_graph.as_ref() {
        log_proxy_response(
            state.as_ref(),
            &request_id,
            graph,
            "POST",
            "/v1/batches",
            next_log_attempt(&log_attempts),
            &response,
        );
        register_proxy_metrics(
            state.as_ref(),
            graph,
            &response,
            "/v1/batches",
            start.elapsed().as_millis() as u64,
        );
    }

    Ok(response)
}

#[tracing::instrument(
    name = "handler.openai.v1.batches.list",
    skip(state, auth, request_id, query)
)]
pub(crate) async fn list_batches(
    State(state): State<Arc<LLMurState>>,
    Extension(auth): Extension<AuthorizationHeaderExtractionResult>,
    Extension(request_id): Extension<crate::data::request_log::RequestLogId>,
    Query(query): Query<ListBatchesQuery>,
) -> Result<ProxyResponse<ListBatchesResponse>, LLMurError> {
    let start = Instant::now();
    let start_ts = Utc::now();
    let log_attempts = Arc::new(AtomicI16::new(0));
    let api_key = extract_api_key(auth)?;
    let virtual_key_id = VirtualKeyId::from_decrypted_key(&api_key);

    let after = match &query.after {
        Some(value) => Some(parse_virtual_batch_id(value)?),
        None => None,
    };
    let limit = query.limit.unwrap_or(20).max(1);
    let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
    let fetch_limit = limit_i64.saturating_add(1);

    let mut batches = state
        .data
        .search_virtual_batches(
            &virtual_key_id,
            &Some(fetch_limit),
            &None::<ListOrder>,
            &after,
            &state.metrics,
        )
        .await?;

    let has_more = (batches.len() as i64) > limit_i64;
    if has_more {
        batches.truncate(limit as usize);
    }

    let data: Vec<BatchObject> = batches
        .iter()
        .map(|batch| {
            normalize_batch_for_openai_surface_with_metrics(
                virtual_batch_to_batch_object(batch),
                &state.metrics,
            )
        })
        .collect();
    let first_id = data.first().map(|batch| batch.id.clone());
    let last_id = data.last().map(|batch| batch.id.clone());

    let response = ProxyResponse::new(
        Ok(ProviderResponse::DecodedResponse {
            data: ListBatchesResponse {
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

    if let Some(batch) = batches.first() {
        if let Some(graph) = load_graph_for_virtual_batch(state.as_ref(), &api_key, batch).await? {
            log_proxy_response(
                state.as_ref(),
                &request_id,
                &graph,
                "GET",
                "/v1/batches",
                next_log_attempt(&log_attempts),
                &response,
            );
            register_proxy_metrics(
                state.as_ref(),
                &graph,
                &response,
                "/v1/batches",
                start.elapsed().as_millis() as u64,
            );
        }
    }

    Ok(response)
}

#[tracing::instrument(
    name = "handler.openai.v1.batches.get",
    skip(state, auth, request_id, batch_id)
)]
pub(crate) async fn get_batch(
    State(state): State<Arc<LLMurState>>,
    Extension(auth): Extension<AuthorizationHeaderExtractionResult>,
    Extension(request_id): Extension<crate::data::request_log::RequestLogId>,
    Path(batch_id): Path<String>,
) -> Result<ProxyResponse<BatchObject>, LLMurError> {
    let start = Instant::now();
    let start_ts = Utc::now();
    let log_attempts = Arc::new(AtomicI16::new(0));
    let api_key = extract_api_key(auth)?;
    let virtual_batch_id = parse_virtual_batch_id(&batch_id)?;

    let virtual_batch = state
        .data
        .get_virtual_batch(&virtual_batch_id, &state.metrics)
        .await?
        .ok_or_else(|| LLMurError::BadRequest("Batch was not found.".to_string()))?;
    let virtual_key_id = VirtualKeyId::from_decrypted_key(&api_key);
    if virtual_batch.virtual_key_id != virtual_key_id {
        return Err(LLMurError::BadRequest(
            "Batch does not belong to this virtual key.".to_string(),
        ));
    }

    let updated = refresh_virtual_batch(
        state.as_ref(),
        &api_key,
        &request_id,
        &log_attempts,
        &virtual_batch,
    )
    .await?;
    let response = ProxyResponse::new(
        Ok(ProviderResponse::DecodedResponse {
            data: normalize_batch_for_openai_surface_with_metrics(
                virtual_batch_to_batch_object(&updated),
                &state.metrics,
            ),
            status_code: reqwest::StatusCode::OK,
        }),
        start_ts,
    );

    if let Some(graph) = load_graph_for_virtual_batch(state.as_ref(), &api_key, &updated).await? {
        let path = format!("/v1/batches/{}", updated.id.0);
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

#[tracing::instrument(
    name = "handler.openai.v1.batches.cancel",
    skip(state, auth, request_id, batch_id)
)]
pub(crate) async fn cancel_batch(
    State(state): State<Arc<LLMurState>>,
    Extension(auth): Extension<AuthorizationHeaderExtractionResult>,
    Extension(request_id): Extension<crate::data::request_log::RequestLogId>,
    Path(batch_id): Path<String>,
) -> Result<ProxyResponse<BatchObject>, LLMurError> {
    let start = Instant::now();
    let start_ts = Utc::now();
    let log_attempts = Arc::new(AtomicI16::new(0));
    let api_key = extract_api_key(auth)?;
    let virtual_batch_id = parse_virtual_batch_id(&batch_id)?;

    let virtual_batch = state
        .data
        .get_virtual_batch(&virtual_batch_id, &state.metrics)
        .await?
        .ok_or_else(|| LLMurError::BadRequest("Batch was not found.".to_string()))?;
    let virtual_key_id = VirtualKeyId::from_decrypted_key(&api_key);
    if virtual_batch.virtual_key_id != virtual_key_id {
        return Err(LLMurError::BadRequest(
            "Batch does not belong to this virtual key.".to_string(),
        ));
    }

    let updated = cancel_virtual_batch(
        state.as_ref(),
        &api_key,
        &request_id,
        &log_attempts,
        &virtual_batch,
    )
    .await?;

    let response = ProxyResponse::new(
        Ok(ProviderResponse::DecodedResponse {
            data: normalize_batch_for_openai_surface_with_metrics(
                virtual_batch_to_batch_object(&updated),
                &state.metrics,
            ),
            status_code: reqwest::StatusCode::OK,
        }),
        start_ts,
    );

    if let Some(graph) = load_graph_for_virtual_batch(state.as_ref(), &api_key, &updated).await? {
        let path = format!("/v1/batches/{}/cancel", updated.id.0);
        log_proxy_response(
            state.as_ref(),
            &request_id,
            &graph,
            "POST",
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

#[derive(Clone)]
enum ProviderConnection {
    OpenAi {
        api_key: String,
        api_endpoint: String,
    },
    Azure {
        api_key: String,
        api_endpoint: String,
        api_version: AzureOpenAiApiVersion,
    },
    Gemini {
        api_key: String,
        api_endpoint: String,
        api_version: GeminiApiVersion,
        model: String,
    },
}

struct ProviderBatchCreate {
    graph: crate::data::graph::Graph,
    provider: String,
    deployment_id: DeploymentId,
    batch_object: BatchObject,
}

#[derive(Default)]
struct AggregatedTimestamps {
    in_progress_at: Option<i64>,
    expires_at: Option<i64>,
    finalizing_at: Option<i64>,
    completed_at: Option<i64>,
    failed_at: Option<i64>,
    expired_at: Option<i64>,
    cancelling_at: Option<i64>,
    cancelled_at: Option<i64>,
}

impl From<&BatchEndpoint> for VirtualBatchEndpoint {
    fn from(value: &BatchEndpoint) -> Self {
        match value {
            BatchEndpoint::Responses => Self::Responses,
            BatchEndpoint::ChatCompletions => Self::ChatCompletions,
            BatchEndpoint::Embeddings => Self::Embeddings,
            BatchEndpoint::Completions => Self::Completions,
        }
    }
}

impl From<&VirtualBatchEndpoint> for BatchEndpoint {
    fn from(value: &VirtualBatchEndpoint) -> Self {
        match value {
            VirtualBatchEndpoint::Responses => Self::Responses,
            VirtualBatchEndpoint::ChatCompletions => Self::ChatCompletions,
            VirtualBatchEndpoint::Embeddings => Self::Embeddings,
            VirtualBatchEndpoint::Completions => Self::Completions,
        }
    }
}

impl From<&CompletionWindow> for VirtualBatchCompletionWindow {
    fn from(value: &CompletionWindow) -> Self {
        match value {
            CompletionWindow::Hours24 => Self::Hours24,
        }
    }
}

impl From<&VirtualBatchCompletionWindow> for CompletionWindow {
    fn from(value: &VirtualBatchCompletionWindow) -> Self {
        match value {
            VirtualBatchCompletionWindow::Hours24 => Self::Hours24,
        }
    }
}

impl From<&BatchStatus> for VirtualBatchStatus {
    fn from(value: &BatchStatus) -> Self {
        match value {
            BatchStatus::Validating => Self::Validating,
            BatchStatus::Failed => Self::Failed,
            BatchStatus::InProgress => Self::InProgress,
            BatchStatus::Finalizing => Self::Finalizing,
            BatchStatus::Completed => Self::Completed,
            BatchStatus::Expired => Self::Expired,
            BatchStatus::Cancelling => Self::Cancelling,
            BatchStatus::Cancelled => Self::Cancelled,
        }
    }
}

impl From<&VirtualBatchStatus> for BatchStatus {
    fn from(value: &VirtualBatchStatus) -> Self {
        match value {
            VirtualBatchStatus::Validating => Self::Validating,
            VirtualBatchStatus::Failed => Self::Failed,
            VirtualBatchStatus::InProgress => Self::InProgress,
            VirtualBatchStatus::Finalizing => Self::Finalizing,
            VirtualBatchStatus::Completed => Self::Completed,
            VirtualBatchStatus::Expired => Self::Expired,
            VirtualBatchStatus::Cancelling => Self::Cancelling,
            VirtualBatchStatus::Cancelled => Self::Cancelled,
        }
    }
}

fn batch_request_counts_to_virtual(
    counts: &Option<BatchRequestCounts>,
) -> Option<VirtualBatchRequestCounts> {
    counts.as_ref().map(|counts| VirtualBatchRequestCounts {
        total: counts.total as i64,
        completed: counts.completed as i64,
        failed: counts.failed as i64,
    })
}

fn virtual_request_counts_to_batch(
    counts: &Option<VirtualBatchRequestCounts>,
) -> Option<BatchRequestCounts> {
    counts.as_ref().map(|counts| BatchRequestCounts {
        total: counts.total.max(0) as u64,
        completed: counts.completed.max(0) as u64,
        failed: counts.failed.max(0) as u64,
    })
}

fn aggregate_virtual_status(statuses: &[VirtualBatchStatus]) -> VirtualBatchStatus {
    // Precedence is intentionally conservative: any failure dominates; all-completed is required
    // for completed; otherwise progress-like states win over validating.
    if statuses
        .iter()
        .any(|status| matches!(status, VirtualBatchStatus::Failed))
    {
        return VirtualBatchStatus::Failed;
    }
    if statuses
        .iter()
        .all(|status| matches!(status, VirtualBatchStatus::Completed))
    {
        return VirtualBatchStatus::Completed;
    }
    if statuses
        .iter()
        .any(|status| matches!(status, VirtualBatchStatus::Cancelling))
    {
        return VirtualBatchStatus::Cancelling;
    }
    if statuses
        .iter()
        .any(|status| matches!(status, VirtualBatchStatus::Cancelled))
    {
        return VirtualBatchStatus::Cancelled;
    }
    if statuses
        .iter()
        .any(|status| matches!(status, VirtualBatchStatus::Expired))
    {
        return VirtualBatchStatus::Expired;
    }
    if statuses
        .iter()
        .any(|status| matches!(status, VirtualBatchStatus::Finalizing))
    {
        return VirtualBatchStatus::Finalizing;
    }
    if statuses
        .iter()
        .any(|status| matches!(status, VirtualBatchStatus::InProgress))
    {
        return VirtualBatchStatus::InProgress;
    }
    VirtualBatchStatus::Validating
}

fn aggregate_virtual_request_counts(
    counts: Vec<Option<VirtualBatchRequestCounts>>,
) -> Option<VirtualBatchRequestCounts> {
    if counts.iter().any(|value| value.is_none()) {
        return None;
    }

    let mut total: i64 = 0;
    let mut completed: i64 = 0;
    let mut failed: i64 = 0;
    for value in counts.into_iter().flatten() {
        total = total.saturating_add(value.total);
        completed = completed.saturating_add(value.completed);
        failed = failed.saturating_add(value.failed);
    }

    Some(VirtualBatchRequestCounts {
        total,
        completed,
        failed,
    })
}

fn expected_provider_request_totals(
    input_file: &VirtualFile,
) -> HashMap<(String, DeploymentId), u64> {
    let mut totals: HashMap<(String, DeploymentId), u64> = HashMap::new();
    let Some(line_map) = input_file.line_map.as_ref() else {
        return totals;
    };

    for line in line_map {
        let key = (line.provider.clone(), line.deployment_id);
        totals
            .entry(key)
            .and_modify(|count| *count = count.saturating_add(1))
            .or_insert(1);
    }

    totals
}

fn normalize_request_counts_for_batch_status(
    status: &BatchStatus,
    counts: &Option<BatchRequestCounts>,
    expected_total: Option<u64>,
) -> Option<BatchRequestCounts> {
    let mut normalized = counts.clone().unwrap_or(BatchRequestCounts {
        total: expected_total.unwrap_or(0),
        completed: 0,
        failed: 0,
    });

    if let Some(expected_total) = expected_total {
        // Virtual batch totals should match the number of mapped input lines per provider.
        // Providers can temporarily over-report totals (for example, while validating).
        normalized.total = expected_total;
    }

    // Enforce terminal-state invariants (`completed + failed == total`) without rewriting
    // non-terminal provider progress.
    match status {
        BatchStatus::Completed => {
            if normalized.total > normalized.failed {
                let completed_floor = normalized.total.saturating_sub(normalized.failed);
                normalized.completed = normalized.completed.max(completed_floor);
            }
        }
        BatchStatus::Failed => {
            if normalized.total > normalized.completed {
                let failed_floor = normalized.total.saturating_sub(normalized.completed);
                normalized.failed = normalized.failed.max(failed_floor);
            }
        }
        _ => {}
    }

    let accounted = normalized.completed.saturating_add(normalized.failed);
    if normalized.total > 0 && accounted > normalized.total {
        match status {
            BatchStatus::Failed => {
                normalized.failed = normalized.total.saturating_sub(normalized.completed);
            }
            _ => {
                normalized.completed = normalized.total.saturating_sub(normalized.failed);
            }
        }
    }

    if normalized.total == 0 {
        None
    } else {
        Some(normalized)
    }
}

fn canonical_provider_batch_id_update(
    current_provider_batch_id: &str,
    parsed_batch_id: &str,
) -> Option<Option<String>> {
    let parsed_batch_id = parsed_batch_id.trim();
    // Gemini can return operation-style ids during create/cancel and canonical `batches/...` ids
    // later; promote to canonical to keep subsequent polling stable.
    if parsed_batch_id.starts_with("batches/") && parsed_batch_id != current_provider_batch_id {
        tracing::debug!(
            current_provider_batch_id,
            parsed_batch_id,
            "Promoting provider batch id to canonical Gemini batch resource id."
        );
        return Some(Some(parsed_batch_id.to_string()));
    }
    None
}

fn merge_provider_file_reference(
    kind: &'static str,
    current_reference: &Option<String>,
    persisted_reference: &Option<String>,
    provider: &str,
    deployment_id: DeploymentId,
    provider_batch_id: &str,
) -> Option<String> {
    match (current_reference, persisted_reference) {
        (Some(current), Some(persisted)) if current != persisted => {
            tracing::debug!(
                kind,
                provider,
                deployment_id = %deployment_id.0,
                provider_batch_id,
                current_reference = %current,
                persisted_reference = %persisted,
                "Provider file reference changed during refresh; using current provider value."
            );
            Some(current.clone())
        }
        (Some(current), _) => Some(current.clone()),
        (None, Some(persisted)) => {
            tracing::debug!(
                kind,
                provider,
                deployment_id = %deployment_id.0,
                provider_batch_id,
                persisted_reference = %persisted,
                "Preserving persisted provider file reference because current response omitted it."
            );
            Some(persisted.clone())
        }
        (None, None) => None,
    }
}

fn aggregate_virtual_timestamps<'a>(
    batches: impl Iterator<Item = &'a BatchObject>,
) -> AggregatedTimestamps {
    let mut result = AggregatedTimestamps::default();

    for batch in batches {
        result.in_progress_at = max_timestamp(result.in_progress_at, batch.in_progress_at);
        result.expires_at = max_timestamp(result.expires_at, batch.expires_at);
        result.finalizing_at = max_timestamp(result.finalizing_at, batch.finalizing_at);
        result.completed_at = max_timestamp(result.completed_at, batch.completed_at);
        result.failed_at = max_timestamp(result.failed_at, batch.failed_at);
        result.expired_at = max_timestamp(result.expired_at, batch.expired_at);
        result.cancelling_at = max_timestamp(result.cancelling_at, batch.cancelling_at);
        result.cancelled_at = max_timestamp(result.cancelled_at, batch.cancelled_at);
    }

    result
}

fn is_terminal_batch_status(status: &BatchStatus) -> bool {
    matches!(
        status,
        BatchStatus::Completed
            | BatchStatus::Failed
            | BatchStatus::Cancelled
            | BatchStatus::Expired
    )
}

fn is_terminal_virtual_status(status: &VirtualBatchStatus) -> bool {
    matches!(
        status,
        VirtualBatchStatus::Completed
            | VirtualBatchStatus::Failed
            | VirtualBatchStatus::Cancelled
            | VirtualBatchStatus::Expired
    )
}

fn batch_endpoint_metric_label(endpoint: &BatchEndpoint) -> &'static str {
    match endpoint {
        BatchEndpoint::Responses => "/v1/responses",
        BatchEndpoint::ChatCompletions => "/v1/chat/completions",
        BatchEndpoint::Embeddings => "/v1/embeddings",
        BatchEndpoint::Completions => "/v1/completions",
    }
}

fn batch_status_metric_label(status: &BatchStatus) -> &'static str {
    match status {
        BatchStatus::Validating => "validating",
        BatchStatus::Failed => "failed",
        BatchStatus::InProgress => "in_progress",
        BatchStatus::Finalizing => "finalizing",
        BatchStatus::Completed => "completed",
        BatchStatus::Expired => "expired",
        BatchStatus::Cancelling => "cancelling",
        BatchStatus::Cancelled => "cancelled",
    }
}

fn normalize_batch_for_openai_surface_with_metrics(
    batch: BatchObject,
    metrics: &Option<Arc<crate::metrics::Metrics>>,
) -> BatchObject {
    let original = batch.clone();
    let normalized = normalize_batch_for_openai_surface(batch);

    let endpoint = batch_endpoint_metric_label(&normalized.endpoint);
    let normalized_status = batch_status_metric_label(&normalized.status);
    let original_status = batch_status_metric_label(&original.status);

    if normalized.status != original.status {
        metrics.register_batch_surface_status_coercion(
            endpoint,
            original_status,
            normalized_status,
        );
    }

    if !is_terminal_batch_status(&normalized.status)
        && (original.output_file_id.is_some() || original.error_file_id.is_some())
    {
        metrics.register_batch_surface_file_id_suppression(
            endpoint,
            normalized_status,
            "non_terminal",
        );
    }

    let original_failed_count = original
        .request_counts
        .as_ref()
        .map(|value| value.failed)
        .unwrap_or(0);
    if is_terminal_batch_status(&normalized.status)
        && original.error_file_id.is_some()
        && normalized.error_file_id.is_none()
        && original_failed_count == 0
        && !matches!(normalized.status, BatchStatus::Failed)
    {
        metrics.register_batch_surface_error_file_suppression(
            endpoint,
            normalized_status,
            "terminal_zero_failures",
        );
    }

    normalized
}

fn normalize_batch_for_openai_surface(mut batch: BatchObject) -> BatchObject {
    let batch_id = batch.id.clone();

    if let Some(request_counts) = batch.request_counts.as_mut()
        && matches!(batch.status, BatchStatus::Validating)
    {
        let progressed = request_counts
            .completed
            .saturating_add(request_counts.failed)
            > 0;
        if progressed {
            // OpenAI surface semantics: once any line has progressed, `validating` should be
            // surfaced as `in_progress`.
            let original_status = batch.status.clone();
            batch.status = BatchStatus::InProgress;
            tracing::debug!(
                batch_id = %batch_id,
                original_status = ?original_status,
                normalized_status = ?batch.status,
                request_total = request_counts.total,
                request_completed = request_counts.completed,
                request_failed = request_counts.failed,
                "Normalized batch status based on request count progress."
            );
        } else {
            let had_non_zero_counts = request_counts.completed > 0 || request_counts.failed > 0;
            request_counts.completed = 0;
            request_counts.failed = 0;
            if had_non_zero_counts {
                tracing::debug!(
                    batch_id = %batch_id,
                    original_status = ?batch.status,
                    normalized_status = ?batch.status,
                    request_total = request_counts.total,
                    request_completed = request_counts.completed,
                    request_failed = request_counts.failed,
                    "Normalized validating batch counts to zero while status remains validating."
                );
            }
        }
    }

    if !is_terminal_batch_status(&batch.status) {
        // OpenAI-compatible lifecycle: never expose output/error file ids before terminal states.
        let had_output_file_id = batch.output_file_id.is_some();
        let had_error_file_id = batch.error_file_id.is_some();
        if had_output_file_id || had_error_file_id {
            tracing::debug!(
                batch_id = %batch_id,
                status = ?batch.status,
                had_output_file_id,
                had_error_file_id,
                "Suppressing batch file ids for non-terminal status."
            );
        }
        batch.output_file_id = None;
        batch.error_file_id = None;
        return batch;
    }

    let failed_count = batch
        .request_counts
        .as_ref()
        .map(|value| value.failed)
        .unwrap_or(0);
    // If the terminal state has no failures, suppress error file ids even if a provider surfaced
    // one transiently.
    if failed_count == 0 && !matches!(batch.status, BatchStatus::Failed) {
        if batch.error_file_id.is_some() {
            tracing::debug!(
                batch_id = %batch_id,
                status = ?batch.status,
                failed_count,
                "Suppressing batch error_file_id because terminal status has zero failures."
            );
        }
        batch.error_file_id = None;
    }

    batch
}

fn persisted_output_file_for_status(
    status: &VirtualBatchStatus,
    output_file_id: Option<VirtualFileId>,
) -> Option<VirtualFileId> {
    if is_terminal_virtual_status(status) {
        output_file_id
    } else {
        None
    }
}

fn max_timestamp(current: Option<i64>, value: Option<u64>) -> Option<i64> {
    let value = value.map(|value| value as i64)?;
    match current {
        Some(existing) => Some(existing.max(value)),
        None => Some(value),
    }
}

fn virtual_batch_to_batch_object(batch: &VirtualBatch) -> BatchObject {
    BatchObject {
        id: batch.id.0.to_string(),
        object: BatchObjectType::Batch,
        endpoint: (&batch.endpoint).into(),
        errors: None,
        input_file_id: batch.input_file_id.0.to_string(),
        completion_window: (&batch.completion_window).into(),
        status: (&batch.status).into(),
        output_file_id: batch.output_file_id.map(|value| value.0.to_string()),
        error_file_id: batch.error_file_id.map(|value| value.0.to_string()),
        created_at: batch.created_at.max(0) as u64,
        in_progress_at: batch.in_progress_at.map(|value| value.max(0) as u64),
        expires_at: batch.expires_at.map(|value| value.max(0) as u64),
        finalizing_at: batch.finalizing_at.map(|value| value.max(0) as u64),
        completed_at: batch.completed_at.map(|value| value.max(0) as u64),
        failed_at: batch.failed_at.map(|value| value.max(0) as u64),
        expired_at: batch.expired_at.map(|value| value.max(0) as u64),
        cancelling_at: batch.cancelling_at.map(|value| value.max(0) as u64),
        cancelled_at: batch.cancelled_at.map(|value| value.max(0) as u64),
        request_counts: virtual_request_counts_to_batch(&batch.request_counts),
        metadata: batch.metadata.clone(),
    }
}

async fn load_graph_for_virtual_batch(
    state: &LLMurState,
    api_key: &str,
    batch: &VirtualBatch,
) -> Result<Option<crate::data::graph::Graph>, LLMurError> {
    let input_file = state
        .data
        .get_virtual_file(&batch.input_file_id, &state.metrics)
        .await?;

    let input_file = match input_file {
        Some(file) => file,
        None => return Ok(None),
    };

    let provider_files = match input_file.provider_files.as_ref() {
        Some(files) if !files.is_empty() => files,
        _ => return Ok(None),
    };

    let deployment_id = provider_files[0].deployment_id;
    let graph = load_graph_for_deployment_id(state, api_key, deployment_id).await?;
    Ok(Some(graph))
}

async fn load_graph_for_deployment_id(
    state: &LLMurState,
    api_key: &str,
    deployment_id: DeploymentId,
) -> Result<crate::data::graph::Graph, LLMurError> {
    let deployment = state
        .data
        .get_deployment(&deployment_id, &state.metrics)
        .await?
        .ok_or_else(|| {
            LLMurError::BadRequest("Deployment not found for batch request.".to_string())
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

fn provider_connection(
    graph: &crate::data::graph::Graph,
    _line_number: usize,
) -> Result<ProviderConnection, LLMurError> {
    match &graph.connection.data.connection_info {
        ConnectionInfo::OpenAiApiKey {
            api_key,
            api_endpoint,
            ..
        } => Ok(ProviderConnection::OpenAi {
            api_key: api_key.clone(),
            api_endpoint: api_endpoint.clone(),
        }),
        ConnectionInfo::AzureOpenAiApiKey {
            api_key,
            api_endpoint,
            api_version,
            ..
        } => Ok(ProviderConnection::Azure {
            api_key: api_key.clone(),
            api_endpoint: api_endpoint.clone(),
            api_version: api_version.clone(),
        }),
        ConnectionInfo::GeminiApiKey {
            api_key,
            api_endpoint,
            api_version,
            model,
            ..
        } => Ok(ProviderConnection::Gemini {
            api_key: api_key.clone(),
            api_endpoint: api_endpoint.clone(),
            api_version: api_version.clone(),
            model: model.clone(),
        }),
    }
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

fn azure_batch_endpoint_supported(endpoint: &BatchEndpoint) -> bool {
    matches!(
        endpoint,
        BatchEndpoint::ChatCompletions | BatchEndpoint::Embeddings
    )
}

fn gemini_batch_endpoint_supported(endpoint: &BatchEndpoint) -> bool {
    matches!(
        endpoint,
        BatchEndpoint::ChatCompletions | BatchEndpoint::Responses
    )
}

async fn refresh_virtual_batch(
    state: &LLMurState,
    api_key: &str,
    request_id: &crate::data::request_log::RequestLogId,
    log_attempts: &Arc<AtomicI16>,
    batch: &VirtualBatch,
) -> Result<VirtualBatch, LLMurError> {
    let provider_batches = state
        .data
        .search_provider_batches(&batch.id, &None, &state.metrics)
        .await?;

    let input_file = state
        .data
        .get_virtual_file(&batch.input_file_id, &state.metrics)
        .await?
        .ok_or_else(|| LLMurError::BadRequest("Input file for batch was not found.".to_string()))?;
    let expected_provider_request_totals = expected_provider_request_totals(&input_file);

    let mut refreshed: Vec<(ProviderBatch, BatchObject, crate::data::graph::Graph)> = Vec::new();
    for provider_batch in provider_batches {
        // Poll each provider batch independently, then reconcile into a single virtual-batch view.
        let graph =
            load_graph_for_deployment_id(state, api_key, provider_batch.deployment_id).await?;
        let connection = provider_connection(&graph, 0)?;
        let provider_input_file_id = input_file
            .provider_files
            .as_ref()
            .and_then(|files| {
                files
                    .iter()
                    .find(|file| file.deployment_id == provider_batch.deployment_id)
                    .map(|file| file.provider_file_id.clone())
            })
            .unwrap_or_else(|| batch.input_file_id.0.to_string());
        let mut batch_object = match connection {
            ProviderConnection::OpenAi {
                api_key,
                api_endpoint,
            } => {
                let url = format!(
                    "{}/v1/batches/{}",
                    api_endpoint.trim_end_matches('/'),
                    provider_batch.provider_batch_id
                );
                let mut request_headers = reqwest::header::HeaderMap::new();
                request_headers.insert(
                    "Authorization",
                    format!("Bearer {}", api_key).parse().unwrap(),
                );
                request_headers.insert("Content-Type", "application/json".parse().unwrap());

                let provider_start = Instant::now();
                let provider_ts = Utc::now();
                let request = state.data.http_client.get(url).headers(request_headers);
                let response = proxy_request_json::<BatchObject>(request).await;
                let proxy_response = ProxyResponse::new(response, provider_ts);

                let path = format!("/v1/batches/{}", provider_batch.provider_batch_id);
                log_proxy_response(
                    state,
                    request_id,
                    &graph,
                    "GET",
                    &path,
                    next_log_attempt(log_attempts),
                    &proxy_response,
                );
                register_proxy_metrics(
                    state,
                    &graph,
                    &proxy_response,
                    &path,
                    provider_start.elapsed().as_millis() as u64,
                );

                match proxy_response.result {
                    Ok(ProviderResponse::DecodedResponse { data, .. }) => data,
                    Ok(ProviderResponse::JsonResponse { .. }) => {
                        return Err(LLMurError::BadRequest(
                            "Batch retrieval returned an unexpected response.".to_string(),
                        ));
                    }
                    Ok(ProviderResponse::Stream { .. }) => {
                        return Err(LLMurError::BadRequest(
                            "Batch retrieval returned an unexpected stream response.".to_string(),
                        ));
                    }
                    Err(error) => return Err(LLMurError::ProxyError(error)),
                }
            }
            ProviderConnection::Azure {
                api_key,
                api_endpoint,
                api_version,
            } => {
                let api_base = azure_api_base(&api_endpoint);
                let url = format!(
                    "{}/batches/{}?api-version={}",
                    api_base, provider_batch.provider_batch_id, api_version
                );
                let mut request_headers = reqwest::header::HeaderMap::new();
                request_headers.insert("api-key", api_key.parse().unwrap());

                let provider_start = Instant::now();
                let provider_ts = Utc::now();
                let request = state.data.http_client.get(url).headers(request_headers);
                let response = proxy_request_json::<AzureBatchObject>(request).await;
                let proxy_response = ProxyResponse::new(response, provider_ts);

                let path = format!("/v1/batches/{}", provider_batch.provider_batch_id);
                log_proxy_response(
                    state,
                    request_id,
                    &graph,
                    "GET",
                    &path,
                    next_log_attempt(log_attempts),
                    &proxy_response,
                );
                register_proxy_metrics(
                    state,
                    &graph,
                    &proxy_response,
                    &path,
                    provider_start.elapsed().as_millis() as u64,
                );

                match proxy_response.result {
                    Ok(ProviderResponse::DecodedResponse { data, .. }) => data,
                    Ok(ProviderResponse::JsonResponse { data, .. }) => {
                        AzureBatchTypes::normalize_batch_response(
                            data,
                            &(&batch.endpoint).into(),
                            &(&batch.completion_window).into(),
                        )
                        .map_err(|_| {
                            LLMurError::BadRequest(
                                "Batch retrieval returned an unexpected response.".to_string(),
                            )
                        })?
                    }
                    Ok(ProviderResponse::Stream { .. }) => {
                        return Err(LLMurError::BadRequest(
                            "Batch retrieval returned an unexpected stream response.".to_string(),
                        ));
                    }
                    Err(error) => return Err(LLMurError::ProxyError(error)),
                }
            }
            ProviderConnection::Gemini {
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
                    provider_batch.provider_batch_id
                );
                let mut request_headers = reqwest::header::HeaderMap::new();
                request_headers.insert("x-goog-api-key", api_key.parse().unwrap());

                let provider_start = Instant::now();
                let provider_ts = Utc::now();
                let request = state.data.http_client.get(url).headers(request_headers);
                let response = proxy_request_json::<GeminiBatchResponse>(request).await;
                let proxy_response = ProxyResponse::new(response, provider_ts);

                let path = format!("/v1/batches/{}", provider_batch.provider_batch_id);
                log_proxy_response(
                    state,
                    request_id,
                    &graph,
                    "GET",
                    &path,
                    next_log_attempt(log_attempts),
                    &proxy_response,
                );
                register_proxy_metrics(
                    state,
                    &graph,
                    &proxy_response,
                    &path,
                    provider_start.elapsed().as_millis() as u64,
                );

                let response_value = match proxy_response.result {
                    Ok(ProviderResponse::DecodedResponse { data, .. }) => data.value,
                    Ok(ProviderResponse::JsonResponse { .. }) => {
                        return Err(LLMurError::BadRequest(
                            "Batch retrieval returned an unexpected response.".to_string(),
                        ));
                    }
                    Ok(ProviderResponse::Stream { .. }) => {
                        return Err(LLMurError::BadRequest(
                            "Batch retrieval returned an unexpected stream response.".to_string(),
                        ));
                    }
                    Err(error) => return Err(LLMurError::ProxyError(error)),
                };

                gemini_batch_object_from_value(
                    response_value,
                    &(&batch.endpoint).into(),
                    &(&batch.completion_window).into(),
                    &provider_input_file_id,
                )?
            }
        };
        let expected_total = expected_provider_request_totals
            .get(&(
                provider_batch.provider.clone(),
                provider_batch.deployment_id,
            ))
            .copied();
        batch_object.request_counts = normalize_request_counts_for_batch_status(
            &batch_object.status,
            &batch_object.request_counts,
            expected_total,
        );
        let provider_batch_id_update =
            canonical_provider_batch_id_update(&provider_batch.provider_batch_id, &batch_object.id);
        // Preserve persisted provider file refs when a provider omits them in intermediate
        // responses, so output/error materialization can still converge.
        batch_object.output_file_id = merge_provider_file_reference(
            "output",
            &batch_object.output_file_id,
            &provider_batch.output_file_id,
            &provider_batch.provider,
            provider_batch.deployment_id,
            &provider_batch.provider_batch_id,
        );
        batch_object.error_file_id = merge_provider_file_reference(
            "error",
            &batch_object.error_file_id,
            &provider_batch.error_file_id,
            &provider_batch.provider,
            provider_batch.deployment_id,
            &provider_batch.provider_batch_id,
        );

        let _ = state
            .data
            .update_provider_batch(
                &provider_batch.id,
                &provider_batch_id_update,
                &Some((&batch_object.status).into()),
                &Some(batch_request_counts_to_virtual(
                    &batch_object.request_counts,
                )),
                &Some(batch_object.output_file_id.clone()),
                &Some(batch_object.error_file_id.clone()),
                &state.metrics,
            )
            .await?;

        refreshed.push((provider_batch, batch_object, graph));
    }

    let statuses: Vec<VirtualBatchStatus> = refreshed
        .iter()
        .map(|(_, batch, _)| (&batch.status).into())
        .collect();
    let request_counts = aggregate_virtual_request_counts(
        refreshed
            .iter()
            .map(|(_, batch, _)| batch_request_counts_to_virtual(&batch.request_counts))
            .collect::<Vec<_>>(),
    );
    let timestamps = aggregate_virtual_timestamps(refreshed.iter().map(|(_, batch, _)| batch));
    let status = aggregate_virtual_status(&statuses);

    // Virtual output/error files are materialized only from terminal, reconciled provider state.
    let output_file_id = ensure_virtual_output_file(
        state,
        api_key,
        request_id,
        log_attempts,
        batch,
        &input_file,
        &refreshed,
        &status,
    )
    .await?;
    let error_file_id = ensure_virtual_error_file(
        state,
        api_key,
        request_id,
        log_attempts,
        batch,
        &input_file,
        &refreshed,
        &status,
        &request_counts,
    )
    .await?;

    let updated = state
        .data
        .update_virtual_batch(
            &batch.id,
            &Some(status),
            &Some(request_counts),
            &Some(output_file_id),
            &Some(error_file_id),
            &Some(timestamps.in_progress_at),
            &Some(timestamps.expires_at),
            &Some(timestamps.finalizing_at),
            &Some(timestamps.completed_at),
            &Some(timestamps.failed_at),
            &Some(timestamps.expired_at),
            &Some(timestamps.cancelling_at),
            &Some(timestamps.cancelled_at),
            &state.metrics,
        )
        .await?;

    Ok(updated)
}

async fn cancel_virtual_batch(
    state: &LLMurState,
    api_key: &str,
    request_id: &crate::data::request_log::RequestLogId,
    log_attempts: &Arc<AtomicI16>,
    batch: &VirtualBatch,
) -> Result<VirtualBatch, LLMurError> {
    let provider_batches = state
        .data
        .search_provider_batches(&batch.id, &None, &state.metrics)
        .await?;
    let input_file = state
        .data
        .get_virtual_file(&batch.input_file_id, &state.metrics)
        .await?
        .ok_or_else(|| LLMurError::BadRequest("Input file for batch was not found.".to_string()))?;
    let expected_provider_request_totals = expected_provider_request_totals(&input_file);

    let mut refreshed: Vec<(ProviderBatch, BatchObject)> = Vec::new();
    for provider_batch in provider_batches {
        // Cancel is also fanout: issue provider-specific cancel calls and persist normalized
        // provider states back into the virtual batch.
        let graph =
            load_graph_for_deployment_id(state, api_key, provider_batch.deployment_id).await?;
        let connection = provider_connection(&graph, 0)?;
        let provider_input_file_id = input_file
            .provider_files
            .as_ref()
            .and_then(|files| {
                files
                    .iter()
                    .find(|file| file.deployment_id == provider_batch.deployment_id)
                    .map(|file| file.provider_file_id.clone())
            })
            .unwrap_or_else(|| batch.input_file_id.0.to_string());
        let mut batch_object = match connection {
            ProviderConnection::OpenAi {
                api_key,
                api_endpoint,
            } => {
                let url = format!(
                    "{}/v1/batches/{}/cancel",
                    api_endpoint.trim_end_matches('/'),
                    provider_batch.provider_batch_id
                );
                let mut request_headers = reqwest::header::HeaderMap::new();
                request_headers.insert(
                    "Authorization",
                    format!("Bearer {}", api_key).parse().unwrap(),
                );
                request_headers.insert("Content-Type", "application/json".parse().unwrap());

                let provider_start = Instant::now();
                let provider_ts = Utc::now();
                let request = state
                    .data
                    .http_client
                    .post(url)
                    .headers(request_headers)
                    .body("{}");
                let response = proxy_request_json::<BatchObject>(request).await;
                let proxy_response = ProxyResponse::new(response, provider_ts);

                let path = format!("/v1/batches/{}/cancel", provider_batch.provider_batch_id);
                log_proxy_response(
                    state,
                    request_id,
                    &graph,
                    "POST",
                    &path,
                    next_log_attempt(log_attempts),
                    &proxy_response,
                );
                register_proxy_metrics(
                    state,
                    &graph,
                    &proxy_response,
                    &path,
                    provider_start.elapsed().as_millis() as u64,
                );

                match proxy_response.result {
                    Ok(ProviderResponse::DecodedResponse { data, .. }) => data,
                    Ok(ProviderResponse::JsonResponse { .. }) => {
                        return Err(LLMurError::BadRequest(
                            "Batch cancel returned an unexpected response.".to_string(),
                        ));
                    }
                    Ok(ProviderResponse::Stream { .. }) => {
                        return Err(LLMurError::BadRequest(
                            "Batch cancel returned an unexpected stream response.".to_string(),
                        ));
                    }
                    Err(error) => return Err(LLMurError::ProxyError(error)),
                }
            }
            ProviderConnection::Azure {
                api_key,
                api_endpoint,
                api_version,
            } => {
                let api_base = azure_api_base(&api_endpoint);
                let url = format!(
                    "{}/batches/{}/cancel?api-version={}",
                    api_base, provider_batch.provider_batch_id, api_version
                );
                let mut request_headers = reqwest::header::HeaderMap::new();
                request_headers.insert("api-key", api_key.parse().unwrap());
                request_headers.insert("Content-Type", "application/json".parse().unwrap());

                let provider_start = Instant::now();
                let provider_ts = Utc::now();
                let request = state
                    .data
                    .http_client
                    .post(url)
                    .headers(request_headers)
                    .body("{}");
                let response = proxy_request_json::<AzureBatchObject>(request).await;
                let proxy_response = ProxyResponse::new(response, provider_ts);

                let path = format!("/v1/batches/{}/cancel", provider_batch.provider_batch_id);
                log_proxy_response(
                    state,
                    request_id,
                    &graph,
                    "POST",
                    &path,
                    next_log_attempt(log_attempts),
                    &proxy_response,
                );
                register_proxy_metrics(
                    state,
                    &graph,
                    &proxy_response,
                    &path,
                    provider_start.elapsed().as_millis() as u64,
                );

                match proxy_response.result {
                    Ok(ProviderResponse::DecodedResponse { data, .. }) => data,
                    Ok(ProviderResponse::JsonResponse { data, .. }) => {
                        AzureBatchTypes::normalize_batch_response(
                            data,
                            &(&batch.endpoint).into(),
                            &(&batch.completion_window).into(),
                        )
                        .map_err(|_| {
                            LLMurError::BadRequest(
                                "Batch cancel returned an unexpected response.".to_string(),
                            )
                        })?
                    }
                    Ok(ProviderResponse::Stream { .. }) => {
                        return Err(LLMurError::BadRequest(
                            "Batch cancel returned an unexpected stream response.".to_string(),
                        ));
                    }
                    Err(error) => return Err(LLMurError::ProxyError(error)),
                }
            }
            ProviderConnection::Gemini {
                api_key,
                api_endpoint,
                api_version,
                ..
            } => {
                let api_base = api_endpoint.trim_end_matches('/');
                let url = format!(
                    "{}/{}/{}:cancel",
                    api_base,
                    gemini_api_version(&api_version),
                    provider_batch.provider_batch_id
                );
                let mut request_headers = reqwest::header::HeaderMap::new();
                request_headers.insert("x-goog-api-key", api_key.parse().unwrap());
                request_headers.insert("Content-Type", "application/json".parse().unwrap());

                let provider_start = Instant::now();
                let provider_ts = Utc::now();
                let request = state
                    .data
                    .http_client
                    .post(url)
                    .headers(request_headers)
                    .body("{}");
                let response = proxy_request_json::<GeminiBatchResponse>(request).await;
                let proxy_response = ProxyResponse::new(response, provider_ts);

                let path = format!("/v1/batches/{}/cancel", provider_batch.provider_batch_id);
                log_proxy_response(
                    state,
                    request_id,
                    &graph,
                    "POST",
                    &path,
                    next_log_attempt(log_attempts),
                    &proxy_response,
                );
                register_proxy_metrics(
                    state,
                    &graph,
                    &proxy_response,
                    &path,
                    provider_start.elapsed().as_millis() as u64,
                );

                let response_value = match proxy_response.result {
                    Ok(ProviderResponse::DecodedResponse { data, .. }) => data.value,
                    Ok(ProviderResponse::JsonResponse { .. }) => {
                        return Err(LLMurError::BadRequest(
                            "Batch cancel returned an unexpected response.".to_string(),
                        ));
                    }
                    Ok(ProviderResponse::Stream { .. }) => {
                        return Err(LLMurError::BadRequest(
                            "Batch cancel returned an unexpected stream response.".to_string(),
                        ));
                    }
                    Err(error) => return Err(LLMurError::ProxyError(error)),
                };

                gemini_batch_object_from_value(
                    response_value,
                    &(&batch.endpoint).into(),
                    &(&batch.completion_window).into(),
                    &provider_input_file_id,
                )?
            }
        };
        let expected_total = expected_provider_request_totals
            .get(&(
                provider_batch.provider.clone(),
                provider_batch.deployment_id,
            ))
            .copied();
        batch_object.request_counts = normalize_request_counts_for_batch_status(
            &batch_object.status,
            &batch_object.request_counts,
            expected_total,
        );
        let provider_batch_id_update =
            canonical_provider_batch_id_update(&provider_batch.provider_batch_id, &batch_object.id);
        batch_object.output_file_id = merge_provider_file_reference(
            "output",
            &batch_object.output_file_id,
            &provider_batch.output_file_id,
            &provider_batch.provider,
            provider_batch.deployment_id,
            &provider_batch.provider_batch_id,
        );
        batch_object.error_file_id = merge_provider_file_reference(
            "error",
            &batch_object.error_file_id,
            &provider_batch.error_file_id,
            &provider_batch.provider,
            provider_batch.deployment_id,
            &provider_batch.provider_batch_id,
        );

        let _ = state
            .data
            .update_provider_batch(
                &provider_batch.id,
                &provider_batch_id_update,
                &Some((&batch_object.status).into()),
                &Some(batch_request_counts_to_virtual(
                    &batch_object.request_counts,
                )),
                &Some(batch_object.output_file_id.clone()),
                &Some(batch_object.error_file_id.clone()),
                &state.metrics,
            )
            .await?;

        refreshed.push((provider_batch, batch_object));
    }

    let statuses: Vec<VirtualBatchStatus> = refreshed
        .iter()
        .map(|(_, batch)| (&batch.status).into())
        .collect();
    let request_counts = aggregate_virtual_request_counts(
        refreshed
            .iter()
            .map(|(_, batch)| batch_request_counts_to_virtual(&batch.request_counts))
            .collect::<Vec<_>>(),
    );
    let timestamps = aggregate_virtual_timestamps(refreshed.iter().map(|(_, batch)| batch));
    let status = aggregate_virtual_status(&statuses);

    let updated = state
        .data
        .update_virtual_batch(
            &batch.id,
            &Some(status),
            &Some(request_counts),
            &None,
            &None,
            &Some(timestamps.in_progress_at),
            &Some(timestamps.expires_at),
            &Some(timestamps.finalizing_at),
            &Some(timestamps.completed_at),
            &Some(timestamps.failed_at),
            &Some(timestamps.expired_at),
            &Some(timestamps.cancelling_at),
            &Some(timestamps.cancelled_at),
            &state.metrics,
        )
        .await?;

    Ok(updated)
}

async fn ensure_virtual_output_file(
    state: &LLMurState,
    api_key: &str,
    request_id: &crate::data::request_log::RequestLogId,
    log_attempts: &Arc<AtomicI16>,
    batch: &VirtualBatch,
    input_file: &VirtualFile,
    refreshed: &[(ProviderBatch, BatchObject, crate::data::graph::Graph)],
    status: &VirtualBatchStatus,
) -> Result<Option<VirtualFileId>, LLMurError> {
    // Keep persisted output only for terminal states; non-terminal exposure is intentionally
    // suppressed to match OpenAI batch lifecycle expectations.
    let persisted_output_file_id = persisted_output_file_for_status(status, batch.output_file_id);
    if persisted_output_file_id.is_some() {
        return Ok(persisted_output_file_id);
    }
    if !is_terminal_virtual_status(status) {
        if batch.output_file_id.is_some() {
            tracing::debug!(
                batch_id = %batch.id.0,
                status = ?status,
                "Clearing persisted virtual output file reference because batch is non-terminal."
            );
        }
        return Ok(None);
    }

    let mut output_map: HashMap<(String, DeploymentId), String> = HashMap::new();
    for (provider_batch, batch_object, _) in refreshed {
        if let Some(output_id) = &batch_object.output_file_id {
            output_map.insert(
                (
                    provider_batch.provider.clone(),
                    provider_batch.deployment_id,
                ),
                output_id.clone(),
            );
        }
    }

    if output_map.len() != refreshed.len() {
        // Require output references from all provider batches before creating a unified virtual
        // output file mapping.
        return Ok(None);
    }

    let input_line_map = input_file.line_map.as_ref().ok_or_else(|| {
        LLMurError::BadRequest("Input file does not have a line map.".to_string())
    })?;

    let mut output_line_map: Vec<VirtualFileLineMapEntry> =
        Vec::with_capacity(input_line_map.len());
    for entry in input_line_map {
        let key = (entry.provider.clone(), entry.deployment_id);
        let provider_file_id = output_map.get(&key).ok_or_else(|| {
            LLMurError::BadRequest(
                "Missing provider output file for virtual output mapping.".to_string(),
            )
        })?;
        output_line_map.push(VirtualFileLineMapEntry {
            line_index: entry.line_index,
            provider: entry.provider.clone(),
            deployment_id: entry.deployment_id,
            provider_file_id: provider_file_id.clone(),
            provider_line_index: entry.provider_line_index,
        });
    }

    let mut provider_files: Vec<ProviderFileReference> = Vec::new();
    let mut bytes_total: i64 = 0;
    for ((provider, deployment_id), provider_file_id) in output_map.iter() {
        let graph = load_graph_for_deployment_id(state, api_key, *deployment_id).await?;
        let file_object = match fetch_provider_file_metadata(
            state,
            &graph,
            request_id,
            log_attempts,
            provider_file_id,
        )
        .await
        {
            Ok(file_object) => file_object,
            Err(error) => {
                tracing::debug!(
                    provider = provider.as_str(),
                    deployment_id = %deployment_id.0,
                    provider_file_id = provider_file_id.as_str(),
                    error = ?error,
                    "Continuing virtual batch output materialization with provider reference only after metadata fetch failure."
                );
                // Metadata fetch is best-effort; provider file ids are enough to reconstruct
                // content later via virtual file line mapping.
                provider_files.push(ProviderFileReference {
                    provider: provider.clone(),
                    deployment_id: *deployment_id,
                    provider_file_id: provider_file_id.clone(),
                });
                continue;
            }
        };
        bytes_total = bytes_total.saturating_add(file_object.bytes as i64);
        provider_files.push(ProviderFileReference {
            provider: provider.clone(),
            deployment_id: *deployment_id,
            provider_file_id: provider_file_id.clone(),
        });
    }

    let virtual_file_id = VirtualFileId(Uuid::now_v7());
    let _ = state
        .data
        .create_virtual_file(
            &virtual_file_id,
            &input_file.virtual_key_id,
            &FilePurpose::BatchOutput,
            &input_file.filename,
            bytes_total.max(0),
            &VirtualFileKind::Output,
            &Some(batch.id),
            &input_file.line_count,
            &Some(provider_files),
            &Some(output_line_map),
            &None,
            &None,
            &state.metrics,
        )
        .await?;

    Ok(Some(virtual_file_id))
}

async fn ensure_virtual_error_file(
    state: &LLMurState,
    api_key: &str,
    request_id: &crate::data::request_log::RequestLogId,
    log_attempts: &Arc<AtomicI16>,
    batch: &VirtualBatch,
    input_file: &VirtualFile,
    refreshed: &[(ProviderBatch, BatchObject, crate::data::graph::Graph)],
    status: &VirtualBatchStatus,
    request_counts: &Option<VirtualBatchRequestCounts>,
) -> Result<Option<VirtualFileId>, LLMurError> {
    // Error files are terminal-only and evidence-driven to avoid publishing stale/empty artifacts.
    if !is_terminal_virtual_status(status) {
        return Ok(None);
    }
    if !should_materialize_error_file(request_counts, refreshed) {
        return Ok(None);
    }
    if batch.error_file_id.is_some() {
        return Ok(batch.error_file_id);
    }

    let mut provider_files: Vec<ProviderFileReference> = Vec::new();
    let mut bytes_total: i64 = 0;
    for (provider_batch, batch_object, _) in refreshed {
        let Some(error_id) = &batch_object.error_file_id else {
            continue;
        };
        let graph =
            load_graph_for_deployment_id(state, api_key, provider_batch.deployment_id).await?;
        let file_object = match fetch_provider_file_metadata(
            state,
            &graph,
            request_id,
            log_attempts,
            error_id,
        )
        .await
        {
            Ok(file_object) => file_object,
            Err(error) => {
                tracing::debug!(
                    provider = provider_batch.provider.as_str(),
                    deployment_id = %provider_batch.deployment_id.0,
                    provider_file_id = error_id.as_str(),
                    error = ?error,
                    "Continuing virtual batch error materialization with provider reference only after metadata fetch failure."
                );
                provider_files.push(ProviderFileReference {
                    provider: provider_batch.provider.clone(),
                    deployment_id: provider_batch.deployment_id,
                    provider_file_id: error_id.clone(),
                });
                continue;
            }
        };
        bytes_total = bytes_total.saturating_add(file_object.bytes as i64);
        provider_files.push(ProviderFileReference {
            provider: provider_batch.provider.clone(),
            deployment_id: provider_batch.deployment_id,
            provider_file_id: error_id.clone(),
        });
    }

    if provider_files.is_empty() {
        return Ok(None);
    }

    let virtual_file_id = VirtualFileId(Uuid::now_v7());
    let _ = state
        .data
        .create_virtual_file(
            &virtual_file_id,
            &input_file.virtual_key_id,
            &FilePurpose::BatchOutput,
            &input_file.filename,
            bytes_total.max(0),
            &VirtualFileKind::Error,
            &Some(batch.id),
            &None,
            &Some(provider_files),
            &None,
            &None,
            &None,
            &state.metrics,
        )
        .await?;

    Ok(Some(virtual_file_id))
}

fn should_materialize_error_file(
    request_counts: &Option<VirtualBatchRequestCounts>,
    refreshed: &[(ProviderBatch, BatchObject, crate::data::graph::Graph)],
) -> bool {
    // Signal tuple: (provider, status, has_explicit_batch_errors, error_file_id).
    let signals: Vec<(String, BatchStatus, bool, Option<String>)> = refreshed
        .iter()
        .map(|(provider_batch, batch, _)| {
            (
                provider_batch.provider.clone(),
                batch.status.clone(),
                batch
                    .errors
                    .as_ref()
                    .map(|errors| !errors.data.is_empty())
                    .unwrap_or(false),
                batch.error_file_id.clone(),
            )
        })
        .collect();
    should_materialize_error_file_from_signals(request_counts, &signals)
}

fn should_materialize_error_file_from_signals(
    request_counts: &Option<VirtualBatchRequestCounts>,
    signals: &[(String, BatchStatus, bool, Option<String>)],
) -> bool {
    // Any failed line count is sufficient evidence; otherwise fall back to provider-level signals.
    if request_counts
        .as_ref()
        .map(|counts| counts.failed.max(0))
        .unwrap_or(0)
        > 0
    {
        return true;
    }

    signals
        .iter()
        .any(|(provider, status, has_errors, error_file_id)| {
            matches!(status, BatchStatus::Failed)
                || *has_errors
                || has_valid_provider_error_file_reference(provider, error_file_id.as_deref())
        })
}

fn has_valid_provider_error_file_reference(provider: &str, error_file_id: Option<&str>) -> bool {
    let Some(error_file_id) = error_file_id else {
        return false;
    };
    let error_file_id = error_file_id.trim();
    if error_file_id.is_empty() {
        return false;
    }

    // Gemini accepts `files/...` style references (and URL-derived variants normalized by helper),
    // while OpenAI/Azure references are constrained to conservative id-like tokens.
    if provider == "gemini" {
        return normalize_gemini_file_reference(error_file_id).is_some();
    }

    if error_file_id.len() > 256 {
        return false;
    }
    if error_file_id.starts_with("http://") || error_file_id.starts_with("https://") {
        return false;
    }
    if error_file_id
        .chars()
        .any(|ch| ch.is_control() || ch.is_whitespace())
    {
        return false;
    }
    if error_file_id.contains('/') {
        return false;
    }
    true
}

async fn fetch_provider_file_metadata(
    state: &LLMurState,
    graph: &crate::data::graph::Graph,
    request_id: &crate::data::request_log::RequestLogId,
    log_attempts: &Arc<AtomicI16>,
    provider_file_id: &str,
) -> Result<FileObject, LLMurError> {
    let connection = provider_connection(graph, 0)?;
    match connection {
        ProviderConnection::OpenAi {
            api_key,
            api_endpoint,
        } => {
            let url = format!(
                "{}/v1/files/{}",
                api_endpoint.trim_end_matches('/'),
                provider_file_id
            );
            let mut request_headers = reqwest::header::HeaderMap::new();
            request_headers.insert(
                "Authorization",
                format!("Bearer {}", api_key).parse().unwrap(),
            );
            let start = Instant::now();
            let request = state.data.http_client.get(url).headers(request_headers);
            let response = proxy_request_json::<AzureFileObject>(request).await;
            let proxy_response = ProxyResponse::new(response, Utc::now());

            let path = format!("/v1/files/{}", provider_file_id);
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
                Ok(ProviderResponse::DecodedResponse { data, .. }) => Ok(data),
                Ok(_) => Err(LLMurError::BadRequest(
                    "Provider file metadata response was unexpected.".to_string(),
                )),
                Err(error) => Err(LLMurError::ProxyError(error)),
            }
        }
        ProviderConnection::Azure {
            api_key,
            api_endpoint,
            api_version,
        } => {
            let api_base = azure_api_base(&api_endpoint);
            let url = format!(
                "{}/files/{}?api-version={}",
                api_base, provider_file_id, api_version
            );
            let mut request_headers = reqwest::header::HeaderMap::new();
            request_headers.insert("api-key", api_key.parse().unwrap());
            let start = Instant::now();
            let request = state.data.http_client.get(url).headers(request_headers);
            let response = proxy_request_json::<FileObject>(request).await;
            let proxy_response = ProxyResponse::new(response, Utc::now());

            let path = format!("/v1/files/{}", provider_file_id);
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
                Ok(ProviderResponse::DecodedResponse { data, .. }) => Ok(data),
                Ok(_) => Err(LLMurError::BadRequest(
                    "Provider file metadata response was unexpected.".to_string(),
                )),
                Err(error) => Err(LLMurError::ProxyError(error)),
            }
        }
        ProviderConnection::Gemini {
            api_key,
            api_endpoint,
            api_version,
            ..
        } => {
            let normalized_provider_file_id = normalize_gemini_provider_file_id(provider_file_id)?;
            let api_base = api_endpoint.trim_end_matches('/');
            let url = format!(
                "{}/{}/{}",
                api_base,
                gemini_api_version(&api_version),
                normalized_provider_file_id
            );
            let mut request_headers = reqwest::header::HeaderMap::new();
            request_headers.insert("x-goog-api-key", api_key.parse().unwrap());
            let start = Instant::now();
            let request = state.data.http_client.get(url).headers(request_headers);
            let response = proxy_request_json::<GeminiFileResource>(request).await;
            let proxy_response = ProxyResponse::new(response, Utc::now());

            let path = format!("/v1/files/{}", normalized_provider_file_id);
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
                Ok(ProviderResponse::DecodedResponse { data, .. }) => {
                    Ok(file_resource_to_openai(data, FilePurpose::BatchOutput)?)
                }
                Ok(_) => Err(LLMurError::BadRequest(
                    "Provider file metadata response was unexpected.".to_string(),
                )),
                Err(error) => Err(LLMurError::ProxyError(error)),
            }
        }
    }
}

fn normalize_gemini_provider_file_id(provider_file_id: &str) -> Result<String, LLMurError> {
    normalize_gemini_file_reference(provider_file_id).ok_or_else(|| {
        LLMurError::BadRequest("Gemini provider file reference is invalid.".to_string())
    })
}

fn next_log_attempt(counter: &Arc<AtomicI16>) -> i16 {
    counter.fetch_add(1, Ordering::Relaxed)
}

fn validate_graph_usage(graph: &crate::data::graph::Graph) -> Result<(), GraphError> {
    graph.virtual_key.validate_limits()?;
    graph.virtual_key_deployment.validate_limits()?;
    graph.project.validate_limits()?;
    graph.deployment.validate_limits()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_provider_batch_id_update, has_valid_provider_error_file_reference,
        merge_provider_file_reference, normalize_batch_for_openai_surface,
        normalize_batch_for_openai_surface_with_metrics, normalize_gemini_provider_file_id,
        normalize_request_counts_for_batch_status, persisted_output_file_for_status,
        should_materialize_error_file_from_signals,
    };
    use crate::data::deployment::DeploymentId;
    use crate::data::virtual_batch::BatchRequestCounts as VirtualBatchRequestCounts;
    use crate::data::virtual_batch::VirtualBatchStatus;
    use crate::data::virtual_file::VirtualFileId;
    use crate::providers::openai::batches::types::{
        BatchEndpoint, BatchObject, BatchObjectType, BatchRequestCounts, BatchStatus,
        CompletionWindow,
    };
    use uuid::Uuid;

    #[test]
    fn normalize_gemini_provider_file_id_accepts_valid_id() {
        let normalized = normalize_gemini_provider_file_id("files/output_abc-123").unwrap();
        assert_eq!(normalized, "files/output_abc-123");
    }

    #[test]
    fn normalize_gemini_provider_file_id_extracts_from_url() {
        let normalized = normalize_gemini_provider_file_id(
            "https://generativelanguage.googleapis.com/v1beta/files/output_abc123?alt=media",
        )
        .unwrap();
        assert_eq!(normalized, "files/output_abc123");
    }

    #[test]
    fn normalize_gemini_provider_file_id_rejects_oversized_ids() {
        let oversized = format!("files/{}", "a".repeat(129));
        assert!(normalize_gemini_provider_file_id(&oversized).is_err());
    }

    #[test]
    fn normalize_request_counts_for_completed_status_fills_missing_completed() {
        let counts = Some(BatchRequestCounts {
            total: 1,
            completed: 0,
            failed: 0,
        });

        let normalized =
            normalize_request_counts_for_batch_status(&BatchStatus::Completed, &counts, Some(1))
                .unwrap();

        assert_eq!(normalized.total, 1);
        assert_eq!(normalized.completed, 1);
        assert_eq!(normalized.failed, 0);
    }

    #[test]
    fn normalize_request_counts_for_completed_status_uses_expected_total_when_missing() {
        let normalized =
            normalize_request_counts_for_batch_status(&BatchStatus::Completed, &None, Some(2))
                .unwrap();

        assert_eq!(normalized.total, 2);
        assert_eq!(normalized.completed, 2);
        assert_eq!(normalized.failed, 0);
    }

    #[test]
    fn normalize_request_counts_clamps_overreported_total_to_expected() {
        let counts = Some(BatchRequestCounts {
            total: 4,
            completed: 0,
            failed: 0,
        });

        let normalized =
            normalize_request_counts_for_batch_status(&BatchStatus::Validating, &counts, Some(2))
                .unwrap();

        assert_eq!(normalized.total, 2);
        assert_eq!(normalized.completed, 0);
        assert_eq!(normalized.failed, 0);
    }

    fn make_batch_object(
        status: BatchStatus,
        request_counts: Option<BatchRequestCounts>,
        output_file_id: Option<&str>,
        error_file_id: Option<&str>,
    ) -> BatchObject {
        BatchObject {
            id: "batch_1".to_string(),
            object: BatchObjectType::Batch,
            endpoint: BatchEndpoint::ChatCompletions,
            errors: None,
            input_file_id: "file_1".to_string(),
            completion_window: CompletionWindow::Hours24,
            status,
            output_file_id: output_file_id.map(|value| value.to_string()),
            error_file_id: error_file_id.map(|value| value.to_string()),
            created_at: 0,
            in_progress_at: None,
            expires_at: None,
            finalizing_at: None,
            completed_at: None,
            failed_at: None,
            expired_at: None,
            cancelling_at: None,
            cancelled_at: None,
            request_counts,
            metadata: None,
        }
    }

    #[test]
    fn normalize_batch_surface_hides_non_terminal_file_ids() {
        let batch = make_batch_object(
            BatchStatus::InProgress,
            Some(BatchRequestCounts {
                total: 2,
                completed: 1,
                failed: 0,
            }),
            Some("file_output"),
            Some("file_error"),
        );

        let normalized = normalize_batch_for_openai_surface(batch);
        assert_eq!(normalized.status, BatchStatus::InProgress);
        assert_eq!(normalized.output_file_id, None);
        assert_eq!(normalized.error_file_id, None);
    }

    #[test]
    fn normalize_batch_surface_validating_zero_progress_contract() {
        let batch = make_batch_object(
            BatchStatus::Validating,
            Some(BatchRequestCounts {
                total: 3,
                completed: 0,
                failed: 0,
            }),
            Some("file_output"),
            Some("file_error"),
        );

        let normalized = normalize_batch_for_openai_surface(batch);
        assert_eq!(normalized.status, BatchStatus::Validating);
        assert_eq!(normalized.output_file_id, None);
        assert_eq!(normalized.error_file_id, None);
        assert_eq!(
            normalized
                .request_counts
                .as_ref()
                .map(|value| value.completed),
            Some(0)
        );
        assert_eq!(
            normalized.request_counts.as_ref().map(|value| value.failed),
            Some(0)
        );
    }

    #[test]
    fn normalize_batch_surface_promotes_validating_with_progress_to_in_progress() {
        let batch = make_batch_object(
            BatchStatus::Validating,
            Some(BatchRequestCounts {
                total: 4,
                completed: 2,
                failed: 0,
            }),
            None,
            Some("file_error"),
        );

        let normalized = normalize_batch_for_openai_surface(batch);
        assert_eq!(normalized.status, BatchStatus::InProgress);
        assert_eq!(normalized.output_file_id, None);
        assert_eq!(normalized.error_file_id, None);
        assert_eq!(
            normalized
                .request_counts
                .as_ref()
                .map(|value| value.completed),
            Some(2)
        );
    }

    #[test]
    fn normalize_batch_surface_finalizing_hides_file_ids_contract() {
        let batch = make_batch_object(
            BatchStatus::Finalizing,
            Some(BatchRequestCounts {
                total: 2,
                completed: 1,
                failed: 0,
            }),
            Some("file_output"),
            Some("file_error"),
        );

        let normalized = normalize_batch_for_openai_surface(batch);
        assert_eq!(normalized.status, BatchStatus::Finalizing);
        assert_eq!(normalized.output_file_id, None);
        assert_eq!(normalized.error_file_id, None);
    }

    #[test]
    fn normalize_batch_surface_cancelling_hides_file_ids_contract() {
        let batch = make_batch_object(
            BatchStatus::Cancelling,
            Some(BatchRequestCounts {
                total: 2,
                completed: 1,
                failed: 0,
            }),
            Some("file_output"),
            Some("file_error"),
        );

        let normalized = normalize_batch_for_openai_surface(batch);
        assert_eq!(normalized.status, BatchStatus::Cancelling);
        assert_eq!(normalized.output_file_id, None);
        assert_eq!(normalized.error_file_id, None);
    }

    #[test]
    fn normalize_batch_surface_hides_terminal_error_file_without_failures() {
        let batch = make_batch_object(
            BatchStatus::Completed,
            Some(BatchRequestCounts {
                total: 2,
                completed: 2,
                failed: 0,
            }),
            Some("file_output"),
            Some("file_error"),
        );

        let normalized = normalize_batch_for_openai_surface(batch);
        assert_eq!(normalized.status, BatchStatus::Completed);
        assert_eq!(normalized.output_file_id.as_deref(), Some("file_output"));
        assert_eq!(normalized.error_file_id, None);
    }

    #[test]
    fn normalize_batch_surface_failed_keeps_error_file_contract() {
        let batch = make_batch_object(
            BatchStatus::Failed,
            Some(BatchRequestCounts {
                total: 2,
                completed: 1,
                failed: 1,
            }),
            Some("file_output"),
            Some("file_error"),
        );

        let normalized = normalize_batch_for_openai_surface(batch);
        assert_eq!(normalized.status, BatchStatus::Failed);
        assert_eq!(normalized.output_file_id.as_deref(), Some("file_output"));
        assert_eq!(normalized.error_file_id.as_deref(), Some("file_error"));
    }

    #[test]
    fn normalize_batch_surface_list_non_terminal_hides_file_ids_contract() {
        let batch = make_batch_object(
            BatchStatus::InProgress,
            Some(BatchRequestCounts {
                total: 2,
                completed: 1,
                failed: 0,
            }),
            Some("file_output"),
            Some("file_error"),
        );

        let normalized = normalize_batch_for_openai_surface_with_metrics(batch, &None);
        assert_eq!(normalized.status, BatchStatus::InProgress);
        assert_eq!(normalized.output_file_id, None);
        assert_eq!(normalized.error_file_id, None);
    }

    #[test]
    fn normalize_batch_surface_cancel_terminal_success_hides_error_file_contract() {
        let batch = make_batch_object(
            BatchStatus::Cancelled,
            Some(BatchRequestCounts {
                total: 2,
                completed: 2,
                failed: 0,
            }),
            Some("file_output"),
            Some("file_error"),
        );

        let normalized = normalize_batch_for_openai_surface_with_metrics(batch, &None);
        assert_eq!(normalized.status, BatchStatus::Cancelled);
        assert_eq!(normalized.output_file_id.as_deref(), Some("file_output"));
        assert_eq!(normalized.error_file_id, None);
    }

    #[test]
    fn normalize_batch_surface_cancel_failed_keeps_error_file_contract() {
        let batch = make_batch_object(
            BatchStatus::Failed,
            Some(BatchRequestCounts {
                total: 2,
                completed: 1,
                failed: 1,
            }),
            Some("file_output"),
            Some("file_error"),
        );

        let normalized = normalize_batch_for_openai_surface_with_metrics(batch, &None);
        assert_eq!(normalized.status, BatchStatus::Failed);
        assert_eq!(normalized.output_file_id.as_deref(), Some("file_output"));
        assert_eq!(normalized.error_file_id.as_deref(), Some("file_error"));
    }

    #[test]
    fn canonical_provider_batch_id_update_promotes_batches_resource() {
        let update = canonical_provider_batch_id_update("operations/abc", "batches/xyz").unwrap();
        assert_eq!(update, Some("batches/xyz".to_string()));
    }

    #[test]
    fn merge_provider_file_reference_preserves_persisted_when_current_missing() {
        let deployment_id = DeploymentId(Uuid::nil());
        let merged = merge_provider_file_reference(
            "output",
            &None,
            &Some("files/output_1".to_string()),
            "gemini",
            deployment_id,
            "batches/123",
        );
        assert_eq!(merged.as_deref(), Some("files/output_1"));
    }

    #[test]
    fn persisted_output_file_for_status_hides_non_terminal_value() {
        let output_file_id = VirtualFileId(Uuid::nil());
        let persisted =
            persisted_output_file_for_status(&VirtualBatchStatus::Validating, Some(output_file_id));
        assert_eq!(persisted, None);
    }

    #[test]
    fn persisted_output_file_for_status_keeps_terminal_value() {
        let output_file_id = VirtualFileId(Uuid::nil());
        let persisted =
            persisted_output_file_for_status(&VirtualBatchStatus::Completed, Some(output_file_id));
        assert_eq!(persisted, Some(output_file_id));
    }

    #[test]
    fn has_valid_provider_error_file_reference_supports_gemini_and_openai_shapes() {
        assert!(has_valid_provider_error_file_reference(
            "gemini",
            Some("files/error_abc123")
        ));
        assert!(has_valid_provider_error_file_reference(
            "openai/v1",
            Some("file-abc123")
        ));
        assert!(!has_valid_provider_error_file_reference(
            "openai/v1",
            Some("https://example.com/file-abc123")
        ));
    }

    #[test]
    fn should_materialize_error_file_from_signals_true_for_valid_terminal_error_file_reference() {
        let signals = vec![(
            "gemini".to_string(),
            BatchStatus::Completed,
            false,
            Some("files/error_abc123".to_string()),
        )];

        assert!(should_materialize_error_file_from_signals(&None, &signals));
    }

    #[test]
    fn should_materialize_error_file_from_signals_false_for_invalid_reference_without_other_evidence()
     {
        let signals = vec![(
            "openai/v1".to_string(),
            BatchStatus::Completed,
            false,
            Some("https://example.com/file-abc123".to_string()),
        )];

        assert!(!should_materialize_error_file_from_signals(&None, &signals));
    }

    #[test]
    fn should_materialize_error_file_from_signals_true_when_failed_count_present() {
        let request_counts = Some(VirtualBatchRequestCounts {
            total: 2,
            completed: 1,
            failed: 1,
        });
        let signals = vec![("openai/v1".to_string(), BatchStatus::Completed, false, None)];

        assert!(should_materialize_error_file_from_signals(
            &request_counts,
            &signals
        ));
    }
}
