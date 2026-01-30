use crate::LLMurState;
use crate::data::connection::{ConnectionId, ConnectionInfo};
use crate::metrics::RegisterProxyRequest;
use crate::providers::openai::responses::request::Request as ResponsesRequest;
use crate::providers::openai::responses::response::Response as ResponsesResponse;
use crate::routes::openai::logging::{RequestLogContext, RequestLogSenders, send_request_log};
use crate::routes::openai::request::OpenAiRequestData;
use crate::routes::openai::response::ProxyResponse;
use axum::Extension;
use axum::extract::State;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use std::sync::Arc;
use std::time::Instant;

// Connection is passed via extension
#[tracing::instrument(
    name = "handler.openai.v1.responses",
    skip(state, connection_info, request),
    fields(
        model = %request.graph.deployment.data.name
    )
)]
pub(crate) async fn responses_route(
    State(state): State<Arc<LLMurState>>,
    Extension(connection_info): Extension<ConnectionInfo>,
    Extension(connection_id): Extension<ConnectionId>,
    Extension(request): Extension<Arc<OpenAiRequestData<ResponsesRequest>>>,
    Extension(request_log_context): Extension<RequestLogContext>,
) -> ProxyResponse<ResponsesResponse> {
    let start = Instant::now();
    let is_stream = request.payload.stream.unwrap_or(false);
    let request_log_context = Arc::new(request_log_context);
    let senders = RequestLogSenders {
        request_log_tx: state.data.request_log_tx.clone(),
        usage_log_tx: state.data.usage_log_tx.clone(),
    };

    let response = match &connection_info {
        ConnectionInfo::AzureOpenAiApiKey {
            api_key,
            api_endpoint,
            api_version,
            deployment_name,
        } => {
            azure_openai_request::responses(
                &state.data.http_client,
                deployment_name,
                api_key,
                api_endpoint,
                api_version,
                request.payload.clone(),
                is_stream,
                request_log_context.clone(),
                senders.clone(),
            )
            .await
        }
        ConnectionInfo::OpenAiApiKey {
            api_key,
            api_endpoint,
            model,
        } => {
            openai_v1_request::responses(
                &state.data.http_client,
                model,
                api_key,
                api_endpoint,
                request.payload.clone(),
                is_stream,
                request_log_context.clone(),
                senders.clone(),
            )
            .await
        }
        ConnectionInfo::GeminiApiKey {
            api_key,
            api_endpoint,
            api_version,
            model,
        } => {
            gemini_v1beta_request::responses(
                &state.data.http_client,
                model,
                api_key,
                api_endpoint,
                api_version,
                request.payload.clone(),
                is_stream,
                request_log_context.clone(),
                senders.clone(),
            )
            .await
        }
    };

    state.metrics.register_proxy_request(
        &request.graph.deployment.data.id,
        &connection_id,
        connection_info.get_provider_friendly_name().to_string(),
        request.path.clone(),
        response
            .result
            .as_ref()
            .map(|r| r.get_input_tokens())
            .unwrap_or_default(),
        response
            .result
            .as_ref()
            .map(|r| r.get_output_tokens())
            .unwrap_or_default(),
        start.elapsed().as_millis() as u64,
        response.result.as_ref().map(|r| r.get_status_code()).ok(),
    );

    response
}

mod azure_openai_request {
    use crate::data::connection::AzureOpenAiApiVersion;
    use crate::providers::Transformer;
    use crate::providers::openai::responses::request::Request as OpenAiRequest;
    use crate::providers::openai::responses::request::to_self::Context as RequestContext;
    use crate::providers::openai::responses::response::Response as OpenAiResponse;
    use crate::providers::openai::responses::response::to_self::Context as ResponseContext;
    use crate::providers::utils::generic_post_proxy_request;
    use crate::routes::openai::logging::{RequestLogContext, RequestLogSenders};
    use crate::routes::openai::response::ProxyResponse;
    use crate::routes::responses::StreamLogHandler;
    use chrono::Utc;
    use reqwest::header::HeaderMap;
    use std::sync::Arc;

    #[tracing::instrument(name = "proxy.azure.openai.responses", skip(client, api_key, payload))]
    pub(crate) async fn responses(
        client: &reqwest::Client,
        deployment_name: &str,
        api_key: &str,
        api_endpoint: &str,
        api_version: &AzureOpenAiApiVersion,
        payload: OpenAiRequest,
        stream: bool,
        request_log_context: Arc<RequestLogContext>,
        senders: RequestLogSenders,
    ) -> ProxyResponse<OpenAiResponse> {
        let mut headers = HeaderMap::new();
        headers.insert("api-key", api_key.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let request_context = RequestContext {
            model: Some(deployment_name.to_string()),
        };
        let response_context = ResponseContext {
            model: Some(payload.model.clone()),
        };
        let api_base = api_endpoint.trim_end_matches('/');
        let api_base = if api_base.ends_with("/openai/v1") {
            api_base.to_string()
        } else {
            format!("{}/openai/v1", api_base)
        };
        let generate_url_fn = |_| {
            format!(
                "{}/responses",
                api_base
            )
        };

        let start_ts = Utc::now();
        if stream {
            headers.insert("Accept", "text/event-stream".parse().unwrap());
            return responses_stream(
                client,
                payload,
                request_context,
                generate_url_fn,
                headers,
                request_log_context,
                senders,
            )
            .await;
        }

        match generic_post_proxy_request(
            client,
            payload,
            request_context,
            generate_url_fn,
            headers,
            response_context,
        )
        .await
        {
            Ok(response) => ProxyResponse::new(Ok(response), start_ts),
            Err(error) => ProxyResponse::new(Err(error), start_ts),
        }
    }

    async fn responses_stream(
        client: &reqwest::Client,
        payload: OpenAiRequest,
        request_context: RequestContext,
        generate_url_fn: impl Fn(crate::providers::openai::responses::request::to_self::Loss) -> String,
        headers: HeaderMap,
        request_log_context: Arc<RequestLogContext>,
        senders: RequestLogSenders,
    ) -> ProxyResponse<OpenAiResponse> {
        let request_ts = Utc::now();
        let request_transformation = payload.transform(request_context);
        let body = match serde_json::to_value(request_transformation.result) {
            Ok(value) => value,
            Err(error) => {
                return ProxyResponse::new(Err(error.into()), request_ts);
            }
        };
        let url = generate_url_fn(request_transformation.loss);
        let response = match client.post(url).headers(headers).json(&body).send().await {
            Ok(resp) => resp,
            Err(error) => return ProxyResponse::new(Err(error.into()), request_ts),
        };

        let status = response.status();
        if !status.is_success() {
            let body_bytes = match response.bytes().await {
                Ok(bytes) => bytes,
                Err(error) => return ProxyResponse::new(Err(error.into()), request_ts),
            };
            let error = match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                Ok(value) => crate::errors::ProxyError::ProxyReturnError(
                    status,
                    crate::errors::ProxyErrorMessage::Json(value),
                ),
                Err(_) => {
                    let text = String::from_utf8_lossy(&body_bytes).to_string();
                    crate::errors::ProxyError::ProxyReturnError(
                        status,
                        crate::errors::ProxyErrorMessage::Text(text),
                    )
                }
            };
            return ProxyResponse::new(Err(error), request_ts);
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());

        let log_handler = StreamLogHandler {
            context: request_log_context,
            senders,
            status_code: status,
            request_ts,
        };
        let stream = super::responses_filter_stream(response, log_handler);
        let body = axum::body::Body::from_stream(stream);

        ProxyResponse::new(
            Ok(crate::routes::openai::response::ProviderResponse::Stream {
                body: std::sync::Arc::new(std::sync::Mutex::new(Some(body))),
                status_code: status,
                content_type,
            }),
            request_ts,
        )
    }
}

mod openai_v1_request {
    use crate::providers::Transformer;
    use crate::providers::openai::responses::request::Request as OpenAiRequest;
    use crate::providers::openai::responses::request::to_self::Context as RequestContext;
    use crate::providers::openai::responses::response::Response as OpenAiResponse;
    use crate::providers::openai::responses::response::to_self::Context as ResponseContext;
    use crate::providers::utils::generic_post_proxy_request;
    use crate::routes::openai::logging::{RequestLogContext, RequestLogSenders};
    use crate::routes::openai::response::ProxyResponse;
    use crate::routes::responses::StreamLogHandler;
    use chrono::Utc;
    use reqwest::header::HeaderMap;
    use std::sync::Arc;

    #[tracing::instrument(name = "proxy.openai.v1.responses", skip(client, api_key, payload))]
    pub(crate) async fn responses(
        client: &reqwest::Client,
        model: &str,
        api_key: &str,
        api_endpoint: &str,
        payload: OpenAiRequest,
        stream: bool,
        request_log_context: Arc<RequestLogContext>,
        senders: RequestLogSenders,
    ) -> ProxyResponse<OpenAiResponse> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", api_key).parse().unwrap(),
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());
        let generate_url_fn = |_| format!("{}/v1/responses", api_endpoint);

        let request_context = RequestContext {
            model: Some(model.to_string()),
        };
        let response_context = ResponseContext {
            model: Some(payload.model.clone()),
        };

        let start_ts = Utc::now();
        if stream {
            headers.insert("Accept", "text/event-stream".parse().unwrap());
            return responses_stream(
                client,
                payload,
                request_context,
                generate_url_fn,
                headers,
                request_log_context,
                senders,
            )
            .await;
        }

        match generic_post_proxy_request(
            client,
            payload,
            request_context,
            generate_url_fn,
            headers,
            response_context,
        )
        .await
        {
            Ok(response) => ProxyResponse::new(Ok(response), start_ts),
            Err(error) => ProxyResponse::new(Err(error), start_ts),
        }
    }

    async fn responses_stream(
        client: &reqwest::Client,
        payload: OpenAiRequest,
        request_context: RequestContext,
        generate_url_fn: impl Fn(crate::providers::openai::responses::request::to_self::Loss) -> String,
        headers: HeaderMap,
        request_log_context: Arc<RequestLogContext>,
        senders: RequestLogSenders,
    ) -> ProxyResponse<OpenAiResponse> {
        let request_ts = Utc::now();
        let request_transformation = payload.transform(request_context);
        let body = match serde_json::to_value(request_transformation.result) {
            Ok(value) => value,
            Err(error) => {
                return ProxyResponse::new(Err(error.into()), request_ts);
            }
        };
        let url = generate_url_fn(request_transformation.loss);
        let response = match client.post(url).headers(headers).json(&body).send().await {
            Ok(resp) => resp,
            Err(error) => return ProxyResponse::new(Err(error.into()), request_ts),
        };

        let status = response.status();
        if !status.is_success() {
            let body_bytes = match response.bytes().await {
                Ok(bytes) => bytes,
                Err(error) => return ProxyResponse::new(Err(error.into()), request_ts),
            };
            let error = match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                Ok(value) => crate::errors::ProxyError::ProxyReturnError(
                    status,
                    crate::errors::ProxyErrorMessage::Json(value),
                ),
                Err(_) => {
                    let text = String::from_utf8_lossy(&body_bytes).to_string();
                    crate::errors::ProxyError::ProxyReturnError(
                        status,
                        crate::errors::ProxyErrorMessage::Text(text),
                    )
                }
            };
            return ProxyResponse::new(Err(error), request_ts);
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());

        let log_handler = StreamLogHandler {
            context: request_log_context,
            senders,
            status_code: status,
            request_ts,
        };
        let stream = super::responses_filter_stream(response, log_handler);
        let body = axum::body::Body::from_stream(stream);

        ProxyResponse::new(
            Ok(crate::routes::openai::response::ProviderResponse::Stream {
                body: std::sync::Arc::new(std::sync::Mutex::new(Some(body))),
                status_code: status,
                content_type,
            }),
            request_ts,
        )
    }
}

mod gemini_v1beta_request {
    use crate::data::connection::GeminiApiVersion;
    use crate::providers::Transformer;
    use crate::providers::gemini::v1beta::generate_content::request::from_openai_responses_transform::Context as RequestContextV1Beta;
    use crate::providers::gemini::v1beta::generate_content::response::Response as GeminiResponse;
    use crate::providers::gemini::v1beta::generate_content::response::UsageMetadata as GeminiUsageMetadata;
    use crate::providers::gemini::v1beta::generate_content::response::to_openai_responses_transform::Context as ResponseContextV1Beta;
    use crate::providers::openai::responses::request::Request as OpenAiRequest;
    use crate::providers::openai::responses::response::{
        Response as OpenAiResponse, ResponseError, ResponseErrorCode,
        ResponseIncompleteDetails, ResponseIncompleteReason, ResponseInputTokensDetails,
        ResponseObject, ResponseOutputTokensDetails, ResponseStatus, ResponseUsage,
    };
    use crate::providers::openai::responses::stream::ResponseStreamEvent as OpenAiResponseStreamEvent;
    use crate::providers::openai::responses::types::{
        FunctionToolCall, FunctionToolCallType, ItemStatus, OutputContent, OutputItem, OutputMessage,
        OutputMessageRole, ToolChoice, ToolChoiceMode,
    };
    use crate::providers::utils::generic_post_proxy_request;
    use crate::routes::openai::logging::{RequestLogContext, RequestLogSenders};
    use crate::routes::openai::response::ProxyResponse;
    use crate::routes::responses::StreamLogHandler;
    use bytes::Bytes;
    use chrono::Utc;
    use futures::StreamExt;
    use reqwest::header::HeaderMap;
    use std::collections::{HashSet, VecDeque};
    use std::sync::Arc;

    #[tracing::instrument(name = "proxy.gemini.v1beta.responses")]
    pub(crate) async fn responses(
        client: &reqwest::Client,
        model: &str,
        api_key: &str,
        api_endpoint: &str,
        api_version: &GeminiApiVersion,
        payload: OpenAiRequest,
        stream: bool,
        request_log_context: Arc<RequestLogContext>,
        senders: RequestLogSenders,
    ) -> ProxyResponse<OpenAiResponse> {
        if stream {
            responses_stream(
                client,
                model,
                api_key,
                api_endpoint,
                api_version,
                payload,
                request_log_context,
                senders,
            )
            .await
        } else {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/json".parse().unwrap());
            let api_version_str = match api_version {
                GeminiApiVersion::V1BETA => "v1beta",
            };
            let endpoint = api_endpoint.trim_end_matches('/');
            let generate_url_fn = |loss: crate::providers::gemini::v1beta::generate_content::request::from_openai_responses_transform::Loss| {
                format!(
                    "{}/{}/models/{}:generateContent?key={}",
                    endpoint,
                    api_version_str,
                    loss.model,
                    api_key
                )
            };

            let request_context = RequestContextV1Beta {
                model: Some(model.to_string()),
            };
            let response_context = build_response_context(model, &payload);

            let start_ts = Utc::now();
            match generic_post_proxy_request(
                client,
                payload,
                request_context,
                generate_url_fn,
                headers,
                response_context,
            )
            .await
            {
                Ok(response) => ProxyResponse::new(Ok(response), start_ts),
                Err(error) => ProxyResponse::new(Err(error), start_ts),
            }
        }
    }

    async fn responses_stream(
        client: &reqwest::Client,
        model: &str,
        api_key: &str,
        api_endpoint: &str,
        api_version: &GeminiApiVersion,
        payload: OpenAiRequest,
        request_log_context: Arc<RequestLogContext>,
        senders: RequestLogSenders,
    ) -> ProxyResponse<OpenAiResponse> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("Accept", "text/event-stream".parse().unwrap());
        let api_version_str = match api_version {
            GeminiApiVersion::V1BETA => "v1beta",
        };
        let endpoint = api_endpoint.trim_end_matches('/');
        let generate_url_fn = |loss: crate::providers::gemini::v1beta::generate_content::request::from_openai_responses_transform::Loss| {
            format!(
                "{}/{}/models/{}:streamGenerateContent?alt=sse&key={}",
                endpoint,
                api_version_str,
                loss.model,
                api_key
            )
        };

        let response_context = build_response_context(model, &payload);
        let request_context = RequestContextV1Beta {
            model: Some(model.to_string()),
        };
        let request_transformation = payload.transform(request_context);
        let request_ts = Utc::now();
        let body = match serde_json::to_value(request_transformation.result) {
            Ok(value) => value,
            Err(error) => {
                return ProxyResponse::new(Err(error.into()), request_ts);
            }
        };

        let url = generate_url_fn(request_transformation.loss);
        let response = match client.post(url).headers(headers).json(&body).send().await {
            Ok(resp) => resp,
            Err(error) => return ProxyResponse::new(Err(error.into()), request_ts),
        };

        let status = response.status();
        if !status.is_success() {
            let body_bytes = match response.bytes().await {
                Ok(bytes) => bytes,
                Err(error) => return ProxyResponse::new(Err(error.into()), request_ts),
            };
            let error = match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                Ok(value) => crate::errors::ProxyError::ProxyReturnError(
                    status,
                    crate::errors::ProxyErrorMessage::Json(value),
                ),
                Err(_) => {
                    let text = String::from_utf8_lossy(&body_bytes).to_string();
                    crate::errors::ProxyError::ProxyReturnError(
                        status,
                        crate::errors::ProxyErrorMessage::Text(text),
                    )
                }
            };
            return ProxyResponse::new(Err(error), request_ts);
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());

        let log_handler = StreamLogHandler {
            context: request_log_context,
            senders,
            status_code: status,
            request_ts,
        };
        let stream = gemini_to_openai_stream(response, response_context, log_handler);
        let body = axum::body::Body::from_stream(stream);

        ProxyResponse::new(
            Ok(crate::routes::openai::response::ProviderResponse::Stream {
                body: std::sync::Arc::new(std::sync::Mutex::new(Some(body))),
                status_code: status,
                content_type,
            }),
            request_ts,
        )
    }

    fn build_response_context(model: &str, payload: &OpenAiRequest) -> ResponseContextV1Beta {
        ResponseContextV1Beta {
            model: Some(model.to_string()),
            parallel_tool_calls: payload.parallel_tool_calls,
            previous_response_id: payload.previous_response_id.clone(),
            reasoning: payload.reasoning.clone(),
            max_output_tokens: payload.max_output_tokens,
            instructions: payload.instructions.clone(),
            text: payload.text.clone(),
            tools: payload.tools.clone(),
            tool_choice: payload.tool_choice.clone(),
            truncation: payload.truncation.clone(),
            metadata: payload.metadata.clone(),
            temperature: payload.temperature,
            top_p: payload.top_p,
            user: payload.user.clone(),
            service_tier: payload.service_tier.clone(),
        }
    }

    fn gemini_to_openai_stream(
        response: reqwest::Response,
        context: ResponseContextV1Beta,
        log_handler: StreamLogHandler,
    ) -> impl futures::Stream<Item = Result<Bytes, std::io::Error>> + Send {
        let upstream = response.bytes_stream();
        let state = GeminiResponsesStreamState::new(upstream, context, log_handler);

        futures::stream::unfold(state, |mut state| async move {
            loop {
                if let Some(chunk) = state.next_pending() {
                    return Some((chunk, state));
                }
                if let Some(event) = state.next_event() {
                    if let Some(chunk) = state.handle_event(event) {
                        return Some((chunk, state));
                    }
                    continue;
                }

                match state.upstream.next().await {
                    Some(Ok(bytes)) => {
                        state.push_bytes(bytes);
                        continue;
                    }
                    Some(Err(err)) => {
                        state.finish_log(None);
                        let io_err = std::io::Error::new(std::io::ErrorKind::Other, err);
                        return Some((Err(io_err), state));
                    }
                    None => {
                        state.finish_stream();
                        if let Some(chunk) = state.next_pending() {
                            return Some((chunk, state));
                        }
                        return None;
                    }
                }
            }
        })
    }

    struct GeminiResponsesStreamState<S> {
        upstream: S,
        buffer: String,
        pending: VecDeque<Result<Bytes, std::io::Error>>,
        response_id: String,
        model: String,
        output_text: String,
        tool_calls: Vec<FunctionToolCall>,
        tool_call_ids: HashSet<String>,
        usage: Option<GeminiUsageMetadata>,
        incomplete_reason: Option<ResponseIncompleteReason>,
        error: Option<ResponseError>,
        created_sent: bool,
        completed_sent: bool,
        context: ResponseContextV1Beta,
        log_handler: StreamLogHandler,
        log_sent: bool,
    }

    impl<S> GeminiResponsesStreamState<S> {
        fn new(upstream: S, context: ResponseContextV1Beta, log_handler: StreamLogHandler) -> Self {
            let model = context
                .model
                .clone()
                .unwrap_or_else(|| "gemini".to_string());
            Self {
                upstream,
                buffer: String::new(),
                pending: VecDeque::new(),
                response_id: "gemini".to_string(),
                model,
                output_text: String::new(),
                tool_calls: Vec::new(),
                tool_call_ids: HashSet::new(),
                usage: None,
                incomplete_reason: None,
                error: None,
                created_sent: false,
                completed_sent: false,
                context,
                log_handler,
                log_sent: false,
            }
        }

        fn next_pending(&mut self) -> Option<Result<Bytes, std::io::Error>> {
            self.pending.pop_front()
        }

        fn push_bytes(&mut self, bytes: Bytes) {
            let chunk = String::from_utf8_lossy(&bytes);
            self.buffer.push_str(&chunk);
            if self.buffer.contains("\r\n") {
                self.buffer = self.buffer.replace("\r\n", "\n");
            }
        }

        fn next_event(&mut self) -> Option<String> {
            if let Some(idx) = self.buffer.find("\n\n") {
                let event = self.buffer[..idx].to_string();
                self.buffer = self.buffer[idx + 2..].to_string();
                return Some(event);
            }
            None
        }

        fn handle_event(&mut self, event: String) -> Option<Result<Bytes, std::io::Error>> {
            let mut data_lines = Vec::new();
            for line in event.lines() {
                if let Some(rest) = line.strip_prefix("data:") {
                    data_lines.push(rest.trim_start().to_string());
                }
            }
            if data_lines.is_empty() {
                return None;
            }
            let data = data_lines.join("\n");
            if data == "[DONE]" {
                self.finish_stream();
                return self.next_pending();
            }

            let gemini_response: GeminiResponse = match serde_json::from_str(&data) {
                Ok(value) => value,
                Err(err) => {
                    let io_err = std::io::Error::new(std::io::ErrorKind::InvalidData, err);
                    return Some(Err(io_err));
                }
            };

            self.process_response(gemini_response);
            self.next_pending()
        }

        fn process_response(&mut self, response: GeminiResponse) {
            if let Some(response_id) = response.response_id {
                self.response_id = response_id;
            }
            if let Some(usage) = response.usage_metadata {
                self.usage = Some(usage);
            }
            if let Some(feedback) = response.prompt_feedback {
                if let Some(reason) = feedback.block_reason {
                    self.error = Some(ResponseError {
                        code: ResponseErrorCode::InvalidPrompt,
                        message: format!("Prompt blocked: {}", reason),
                    });
                }
            }

            if !self.created_sent {
                let created = self.build_openai_response(ResponseStatus::InProgress);
                self.enqueue_event(OpenAiResponseStreamEvent::Created { response: created });
                self.created_sent = true;
            }

            if let Some(candidates) = response.candidates {
                for (candidate_index, candidate) in candidates.into_iter().enumerate() {
                    if let Some(reason) = candidate.finish_reason.as_ref() {
                        match reason.to_ascii_uppercase().as_str() {
                            "MAX_TOKENS" => {
                                self.incomplete_reason =
                                    Some(ResponseIncompleteReason::MaxOutputTokens);
                            }
                            "SAFETY" | "RECITATION" => {
                                self.incomplete_reason =
                                    Some(ResponseIncompleteReason::ContentFilter);
                            }
                            _ => {}
                        }
                    }

                    if let Some(content) = candidate.content {
                        for (part_index, part) in content.parts.into_iter().enumerate() {
                            if let Some(text) = part.text {
                                self.output_text.push_str(&text);
                                self.enqueue_event(OpenAiResponseStreamEvent::TextDelta {
                                    item_id: self.item_id(),
                                    output_index: candidate_index as u64,
                                    content_index: 0,
                                    delta: text,
                                });
                            }
                            if let Some(function_call) = part.function_call {
                                let call_id =
                                    format!("gemini-call-{}-{}", candidate_index, part_index);
                                if self.tool_call_ids.insert(call_id.clone()) {
                                    let arguments = serde_json::to_string(&function_call.args)
                                        .unwrap_or_else(|_| "{}".to_string());
                                    self.tool_calls.push(FunctionToolCall {
                                        id: Some(call_id.clone()),
                                        tool_type: FunctionToolCallType::FunctionCall,
                                        call_id,
                                        name: function_call.name,
                                        arguments,
                                        status: Some(ItemStatus::Completed),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        fn finish_stream(&mut self) {
            if self.completed_sent {
                return;
            }
            let status = if self.error.is_some() {
                ResponseStatus::Failed
            } else if self.incomplete_reason.is_some() {
                ResponseStatus::Incomplete
            } else {
                ResponseStatus::Completed
            };
            let completed = self.build_openai_response(status);
            let event = match completed.status {
                ResponseStatus::Failed => OpenAiResponseStreamEvent::Failed {
                    response: completed,
                },
                ResponseStatus::Incomplete => OpenAiResponseStreamEvent::Incomplete {
                    response: completed,
                },
                _ => OpenAiResponseStreamEvent::Completed {
                    response: completed,
                },
            };
            self.enqueue_event(event);
            self.completed_sent = true;
            self.finish_log(self.error.as_ref().map(|err| {
                format!("{:?}: {}", err.code, err.message)
            }).or_else(|| self.incomplete_reason.as_ref().map(|reason| format!("incomplete: {:?}", reason))));
        }

        fn finish_log(&mut self, error: Option<String>) {
            if self.log_sent {
                return;
            }
            let (input_tokens, output_tokens) = match self.usage.as_ref() {
                Some(usage) => (
                    usage.prompt_token_count.map(|v| v as i64),
                    usage.candidates_token_count.map(|v| v as i64),
                ),
                None => (None, None),
            };
            self.log_handler.send(input_tokens, output_tokens, error);
            self.log_sent = true;
        }

        fn build_openai_response(&self, status: ResponseStatus) -> OpenAiResponse {
            let output_text = if self.output_text.is_empty() {
                None
            } else {
                Some(self.output_text.clone())
            };
            let mut output = Vec::new();
            if let Some(text) = output_text.clone() {
                output.push(OutputItem::OutputMessage(OutputMessage {
                    id: format!("gemini-msg-{}", self.response_id),
                    message_type: None,
                    role: OutputMessageRole::Assistant,
                    content: vec![OutputContent::OutputText {
                        text,
                        annotations: Vec::new(),
                    }],
                    status: ItemStatus::Completed,
                }));
            }
            for call in &self.tool_calls {
                output.push(OutputItem::FunctionToolCall(call.clone()));
            }

            let usage = self.usage.as_ref().map(|usage| {
                let input_tokens = usage.prompt_token_count.unwrap_or(0);
                let output_tokens = usage.candidates_token_count.unwrap_or(0);
                let total_tokens =
                    usage.total_token_count.unwrap_or(input_tokens + output_tokens);
                let cached_tokens = usage.cached_content_token_count.unwrap_or(0);
                let reasoning_tokens = usage.thoughts_token_count.unwrap_or(0);
                ResponseUsage {
                    input_tokens,
                    input_tokens_details: ResponseInputTokensDetails { cached_tokens },
                    output_tokens,
                    output_tokens_details: ResponseOutputTokensDetails { reasoning_tokens },
                    total_tokens,
                }
            });

            let tools = self.context.tools.clone().unwrap_or_default();
            let tool_choice = self
                .context
                .tool_choice
                .clone()
                .unwrap_or_else(|| default_tool_choice(&tools));
            let incomplete_details = self
                .incomplete_reason
                .as_ref()
                .map(|reason| ResponseIncompleteDetails { reason: reason.clone() });

            OpenAiResponse {
                id: self.response_id.clone(),
                object: ResponseObject::Response,
                created_at: 0,
                status,
                error: self.error.clone(),
                incomplete_details,
                output,
                output_text,
                usage,
                parallel_tool_calls: self.context.parallel_tool_calls.unwrap_or(false),
                previous_response_id: self.context.previous_response_id.clone(),
                model: self.model.clone(),
                reasoning: self.context.reasoning.clone(),
                max_output_tokens: self.context.max_output_tokens,
                instructions: self.context.instructions.clone(),
                text: self.context.text.clone(),
                tools,
                tool_choice,
                truncation: self.context.truncation.clone(),
                metadata: self.context.metadata.clone(),
                temperature: self.context.temperature,
                top_p: self.context.top_p,
                user: self.context.user.clone(),
                service_tier: self.context.service_tier.clone(),
            }
        }

        fn enqueue_event(&mut self, event: OpenAiResponseStreamEvent) {
            let event_name = event_name(&event);
            let payload = match serde_json::to_string(&event) {
                Ok(value) => value,
                Err(err) => {
                    let io_err = std::io::Error::new(std::io::ErrorKind::Other, err);
                    self.pending.push_back(Err(io_err));
                    return;
                }
            };
            let data = format!("event: {}\ndata: {}\n\n", event_name, payload);
            self.pending.push_back(Ok(Bytes::from(data)));
        }

        fn item_id(&self) -> String {
            format!("gemini-msg-{}", self.response_id)
        }
    }

    fn default_tool_choice(tools: &[crate::providers::openai::responses::types::Tool]) -> ToolChoice {
        if tools.is_empty() {
            ToolChoice::Mode(ToolChoiceMode::None)
        } else {
            ToolChoice::Mode(ToolChoiceMode::Auto)
        }
    }

    fn event_name(event: &OpenAiResponseStreamEvent) -> &'static str {
        match event {
            OpenAiResponseStreamEvent::Created { .. } => "response.created",
            OpenAiResponseStreamEvent::TextDelta { .. } => "response.output_text.delta",
            OpenAiResponseStreamEvent::Completed { .. } => "response.completed",
            OpenAiResponseStreamEvent::Failed { .. } => "response.failed",
            OpenAiResponseStreamEvent::Incomplete { .. } => "response.incomplete",
            _ => "response.output_text.delta",
        }
    }
}

fn responses_filter_stream(
    response: reqwest::Response,
    log_handler: StreamLogHandler,
) -> impl futures::Stream<Item = Result<bytes::Bytes, std::io::Error>> + Send {
    let upstream = response.bytes_stream();
    let state = ResponsesStreamState::new(upstream, log_handler);

    futures::stream::unfold(state, |mut state| async move {
        loop {
            if let Some(event) = state.next_event() {
                if let Some(chunk) = state.handle_event(event) {
                    return Some((chunk, state));
                }
                continue;
            }

            match state.upstream.next().await {
                Some(Ok(bytes)) => {
                    state.push_bytes(bytes);
                    continue;
                }
                Some(Err(err)) => {
                    state.finish_log();
                    let io_err = std::io::Error::new(std::io::ErrorKind::Other, err);
                    return Some((Err(io_err), state));
                }
                None => {
                    state.finish_log();
                    return None;
                }
            }
        }
    })
}

struct ResponsesStreamState<S> {
    upstream: S,
    buffer: String,
    log_handler: StreamLogHandler,
    log_sent: bool,
}

impl<S> ResponsesStreamState<S> {
    fn new(upstream: S, log_handler: StreamLogHandler) -> Self {
        Self {
            upstream,
            buffer: String::new(),
            log_handler,
            log_sent: false,
        }
    }

    fn push_bytes(&mut self, bytes: bytes::Bytes) {
        let chunk = String::from_utf8_lossy(&bytes);
        self.buffer.push_str(&chunk);
        if self.buffer.contains("\r\n") {
            self.buffer = self.buffer.replace("\r\n", "\n");
        }
    }

    fn next_event(&mut self) -> Option<String> {
        if let Some(idx) = self.buffer.find("\n\n") {
            let event = self.buffer[..idx].to_string();
            self.buffer = self.buffer[idx + 2..].to_string();
            return Some(event);
        }
        None
    }

    fn handle_event(&mut self, event: String) -> Option<Result<bytes::Bytes, std::io::Error>> {
        let mut data_lines = Vec::new();
        for line in event.lines() {
            if let Some(rest) = line.strip_prefix("data:") {
                data_lines.push(rest.trim_start().to_string());
            }
        }
        if data_lines.is_empty() {
            return None;
        }
        let data = data_lines.join("\n");
        if data == "[DONE]" {
            self.finish_log();
            return Some(Ok(bytes::Bytes::from(format!("{}\n\n", event))));
        }

        if let Ok(stream_event) = serde_json::from_str::<
            crate::providers::openai::responses::stream::ResponseStreamEvent,
        >(&data)
        {
            self.handle_stream_event(&stream_event);
        }

        Some(Ok(bytes::Bytes::from(format!("{}\n\n", event))))
    }

    fn handle_stream_event(
        &mut self,
        event: &crate::providers::openai::responses::stream::ResponseStreamEvent,
    ) {
        use crate::providers::openai::responses::stream::ResponseStreamEvent;
        match event {
            ResponseStreamEvent::Completed { response } => {
                self.send_usage_log(response, None);
            }
            ResponseStreamEvent::Failed { response } => {
                let error = response
                    .error
                    .as_ref()
                    .map(|err| format!("{:?}: {}", err.code, err.message))
                    .or_else(|| Some("response failed".to_string()));
                self.send_usage_log(response, error);
            }
            ResponseStreamEvent::Incomplete { response } => {
                let error = response
                    .error
                    .as_ref()
                    .map(|err| format!("{:?}: {}", err.code, err.message))
                    .or_else(|| {
                        response
                            .incomplete_details
                            .as_ref()
                            .map(|details| format!("incomplete: {:?}", details.reason))
                    })
                    .or_else(|| Some("response incomplete".to_string()));
                self.send_usage_log(response, error);
            }
            ResponseStreamEvent::Error { message, .. } => {
                self.send_error_log(message.clone());
            }
            _ => {}
        }
    }

    fn send_usage_log(
        &mut self,
        response: &crate::providers::openai::responses::response::Response,
        error: Option<String>,
    ) {
        if self.log_sent {
            return;
        }
        let (input_tokens, output_tokens) = match response.usage.as_ref() {
            Some(usage) => (
                Some(usage.input_tokens as i64),
                Some(usage.output_tokens as i64),
            ),
            None => (None, None),
        };
        self.log_handler.send(input_tokens, output_tokens, error);
        self.log_sent = true;
    }

    fn send_error_log(&mut self, message: String) {
        if self.log_sent {
            return;
        }
        self.log_handler.send(None, None, Some(message));
        self.log_sent = true;
    }

    fn finish_log(&mut self) {
        if self.log_sent {
            return;
        }
        self.log_handler.send(None, None, None);
        self.log_sent = true;
    }
}

#[derive(Clone)]
struct StreamLogHandler {
    context: Arc<RequestLogContext>,
    senders: RequestLogSenders,
    status_code: reqwest::StatusCode,
    request_ts: DateTime<Utc>,
}

impl StreamLogHandler {
    fn send(&self, input_tokens: Option<i64>, output_tokens: Option<i64>, error: Option<String>) {
        send_request_log(
            &self.context,
            &self.senders,
            self.status_code,
            input_tokens,
            output_tokens,
            error,
            Utc::now(),
        );
    }
}
