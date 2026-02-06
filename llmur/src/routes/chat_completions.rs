use crate::LLMurState;
use crate::data::connection::{ConnectionId, ConnectionInfo};
use crate::metrics::RegisterProxyRequest;
use crate::providers::openai::chat_completions::request::Request as ChatCompletionsRequest;
use crate::providers::openai::chat_completions::response::Response as ChatCompletionsResponse;
use crate::routes::openai::logging::{RequestLogContext, RequestLogSenders, send_request_log};
use crate::routes::openai::request::OpenAiRequestData;
use crate::routes::openai::response::ProxyResponse;
use axum::Extension;
use axum::extract::State;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Instant;

// Connection is passed via extension
#[tracing::instrument(
    name = "handler.openai.v1.chat_completions",
    skip(state, connection_info, request),
    fields(
        model = %request.graph.deployment.data.name
    )
)]
pub(crate) async fn chat_completions_route(
    State(state): State<Arc<LLMurState>>,
    Extension(connection_info): Extension<ConnectionInfo>,
    Extension(connection_id): Extension<ConnectionId>,
    Extension(request): Extension<Arc<OpenAiRequestData<ChatCompletionsRequest>>>,
    Extension(request_log_context): Extension<RequestLogContext>,
) -> ProxyResponse<ChatCompletionsResponse> {
    let start = Instant::now();
    let is_stream = request.payload.stream.unwrap_or(false);
    let include_usage_requested = request
        .payload
        .stream_options
        .as_ref()
        .and_then(|options| options.include_usage)
        .unwrap_or(false);
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
            azure_openai_request::chat_completions(
                &state.data.http_client,
                deployment_name,
                api_key,
                api_endpoint,
                api_version,
                request.payload.clone(),
                is_stream,
                include_usage_requested,
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
            openai_v1_request::chat_completions(
                &state.data.http_client,
                model,
                api_key,
                api_endpoint,
                request.payload.clone(),
                is_stream,
                include_usage_requested,
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
            gemini_v1beta_request::chat_completions(
                &state.data.http_client,
                model,
                api_key,
                api_endpoint,
                api_version,
                request.payload.clone(),
                is_stream,
                include_usage_requested,
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
    use crate::providers::azure::openai::v1::chat_completions::request::from_openai_transform::Context as RequestContextV1;
    use crate::providers::azure::openai::v1::chat_completions::response::to_openai_transform::Context as ResponseContextV1;
    use crate::providers::openai::chat_completions::request::Request as OpenAiRequest;
    use crate::providers::openai::chat_completions::response::Response as OpenAiResponse;
    use crate::providers::utils::generic_post_proxy_request;
    use crate::routes::chat_completions::StreamLogHandler;
    use crate::routes::openai::logging::{RequestLogContext, RequestLogSenders};
    use crate::routes::openai::response::ProxyResponse;
    use bytes::Bytes;
    use chrono::Utc;
    use futures::StreamExt;
    use reqwest::header::HeaderMap;
    use std::sync::Arc;

    #[tracing::instrument(
        name = "proxy.azure.openai.chat_completions",
        skip(client, api_key, payload)
    )]
    pub(crate) async fn chat_completions(
        client: &reqwest::Client,
        deployment_name: &str,
        api_key: &str,
        api_endpoint: &str,
        api_version: &AzureOpenAiApiVersion,
        payload: OpenAiRequest,
        stream: bool,
        include_usage_requested: bool,
        request_log_context: Arc<RequestLogContext>,
        senders: RequestLogSenders,
    ) -> ProxyResponse<OpenAiResponse> {
        let mut headers = HeaderMap::new();
        headers.insert("api-key", api_key.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let request_context = RequestContextV1 {
            model: Some(deployment_name.to_string()),
            safety_identifier: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            user_security_context: None,
            stream_include_usage: true,
        };
        let response_context = ResponseContextV1 {
            model: Some(payload.model.clone()),
        };
        let api_base = api_endpoint.trim_end_matches('/');
        let api_base = if api_base.ends_with("/openai/v1") {
            api_base.to_string()
        } else {
            format!("{}/openai/v1", api_base)
        };
        let generate_url_fn = |_| format!("{}/chat/completions", api_base);

        let start_ts = Utc::now();
        if stream {
            headers.insert("Accept", "text/event-stream".parse().unwrap());
            return chat_completions_stream_v1(
                client,
                payload,
                request_context,
                generate_url_fn,
                headers,
                include_usage_requested,
                request_log_context,
                senders,
            )
            .await;
        } else {
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

    async fn chat_completions_stream_v1(
        client: &reqwest::Client,
        payload: OpenAiRequest,
        request_context: RequestContextV1,
        generate_url_fn: impl Fn(crate::providers::azure::openai::v1::chat_completions::request::from_openai_transform::Loss) -> String,
        headers: HeaderMap,
        include_usage_requested: bool,
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
        let stream = azure_filter_stream(response, include_usage_requested, log_handler);
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

    fn azure_filter_stream(
        response: reqwest::Response,
        include_usage_requested: bool,
        log_handler: StreamLogHandler,
    ) -> impl futures::Stream<Item = Result<Bytes, std::io::Error>> + Send {
        let upstream = response.bytes_stream();
        let state = AzureStreamState::new(upstream, include_usage_requested, log_handler);

        futures::stream::unfold(state, |mut state| async move {
            loop {
                if let Some(event) = state.next_event() {
                    if let Some(chunk) = state.filter_event(event) {
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

    struct AzureStreamState<S> {
        upstream: S,
        buffer: String,
        include_usage_requested: bool,
        log_handler: StreamLogHandler,
        log_sent: bool,
    }

    impl<S> AzureStreamState<S> {
        fn new(upstream: S, include_usage_requested: bool, log_handler: StreamLogHandler) -> Self {
            Self {
                upstream,
                buffer: String::new(),
                include_usage_requested,
                log_handler,
                log_sent: false,
            }
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

        fn filter_event(&mut self, event: String) -> Option<Result<Bytes, std::io::Error>> {
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
                return Some(Ok(Bytes::from("data: [DONE]\n\n")));
            }

            let value: serde_json::Value = match serde_json::from_str(&data) {
                Ok(value) => value,
                Err(_) => {
                    return Some(Ok(Bytes::from(format!("data: {}\n\n", data))));
                }
            };
            let choices_empty = value
                .get("choices")
                .and_then(|choices| choices.as_array())
                .map(|choices| choices.is_empty())
                .unwrap_or(true);
            let has_prompt_filter_results = value.get("prompt_filter_results").is_some();
            let has_usage = !value
                .get("usage")
                .unwrap_or(&serde_json::Value::Null)
                .is_null();

            if choices_empty && has_prompt_filter_results {
                return None;
            }
            if choices_empty && has_usage && !self.include_usage_requested {
                self.send_usage_log(&value);
                return None;
            }
            if choices_empty && has_usage {
                self.send_usage_log(&value);
            }

            let payload = match serde_json::to_string(&value) {
                Ok(value) => value,
                Err(err) => {
                    let io_err = std::io::Error::new(std::io::ErrorKind::Other, err);
                    return Some(Err(io_err));
                }
            };

            Some(Ok(Bytes::from(format!("data: {}\n\n", payload))))
        }

        fn send_usage_log(&mut self, value: &serde_json::Value) {
            if self.log_sent {
                return;
            }
            let usage = extract_usage(value);
            let (input_tokens, output_tokens) = match usage {
                Some((input_tokens, output_tokens)) => (Some(input_tokens), Some(output_tokens)),
                None => (None, None),
            };
            self.log_handler.send(input_tokens, output_tokens, None);
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

    fn extract_usage(value: &serde_json::Value) -> Option<(i64, i64)> {
        let usage = value.get("usage")?;
        let prompt_tokens = usage.get("prompt_tokens")?.as_i64()?;
        if let Some(completion_tokens) = usage.get("completion_tokens").and_then(|v| v.as_i64()) {
            return Some((prompt_tokens, completion_tokens));
        }
        let total_tokens = usage.get("total_tokens")?.as_i64()?;
        let completion_tokens = total_tokens.saturating_sub(prompt_tokens);
        Some((prompt_tokens, completion_tokens))
    }
}

mod openai_v1_request {
    use crate::providers::Transformer;
    use crate::providers::openai::chat_completions::request::Request as OpenAiRequest;
    use crate::providers::openai::chat_completions::request::to_self::Context as RequestContext;
    use crate::providers::openai::chat_completions::response::Response as OpenAiResponse;
    use crate::providers::openai::chat_completions::response::to_self::Context as ResponseContext;
    use crate::providers::utils::generic_post_proxy_request;
    use crate::routes::chat_completions::StreamLogHandler;
    use crate::routes::openai::logging::{RequestLogContext, RequestLogSenders};
    use crate::routes::openai::response::ProxyResponse;
    use bytes::Bytes;
    use chrono::Utc;
    use futures::StreamExt;
    use reqwest::header::HeaderMap;
    use std::sync::Arc;

    #[tracing::instrument(
        name = "proxy.openai.v1.chat_completions",
        skip(client, api_key, payload)
    )]
    pub(crate) async fn chat_completions(
        client: &reqwest::Client,
        model: &str,
        api_key: &str,
        api_endpoint: &str,
        payload: OpenAiRequest,
        stream: bool,
        include_usage_requested: bool,
        request_log_context: Arc<RequestLogContext>,
        senders: RequestLogSenders,
    ) -> ProxyResponse<OpenAiResponse> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", api_key).parse().unwrap(),
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());
        let generate_url_fn = |_| format!("{}/v1/chat/completions", api_endpoint);

        let request_context = RequestContext {
            model: Some(model.to_string()),
            stream_include_usage: true,
        };
        let response_context = ResponseContext {
            model: Some(payload.model.clone()),
        };

        let start_ts = Utc::now();
        if stream {
            headers.insert("Accept", "text/event-stream".parse().unwrap());
            return chat_completions_stream(
                client,
                payload,
                request_context,
                generate_url_fn,
                headers,
                include_usage_requested,
                request_log_context,
                senders,
            )
            .await;
        } else {
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

    async fn chat_completions_stream(
        client: &reqwest::Client,
        payload: OpenAiRequest,
        request_context: RequestContext,
        generate_url_fn: impl Fn(
            crate::providers::openai::chat_completions::request::to_self::Loss,
        ) -> String,
        headers: HeaderMap,
        include_usage_requested: bool,
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
        let stream = openai_filter_stream(response, include_usage_requested, log_handler);
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

    fn openai_filter_stream(
        response: reqwest::Response,
        include_usage_requested: bool,
        log_handler: StreamLogHandler,
    ) -> impl futures::Stream<Item = Result<Bytes, std::io::Error>> + Send {
        let upstream = response.bytes_stream();
        let state = OpenAiStreamState::new(upstream, include_usage_requested, log_handler);

        futures::stream::unfold(state, |mut state| async move {
            loop {
                if let Some(event) = state.next_event() {
                    if let Some(chunk) = state.filter_event(event) {
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

    struct OpenAiStreamState<S> {
        upstream: S,
        buffer: String,
        include_usage_requested: bool,
        log_handler: StreamLogHandler,
        log_sent: bool,
    }

    impl<S> OpenAiStreamState<S> {
        fn new(upstream: S, include_usage_requested: bool, log_handler: StreamLogHandler) -> Self {
            Self {
                upstream,
                buffer: String::new(),
                include_usage_requested,
                log_handler,
                log_sent: false,
            }
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

        fn filter_event(&mut self, event: String) -> Option<Result<Bytes, std::io::Error>> {
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
                return Some(Ok(Bytes::from("data: [DONE]\n\n")));
            }

            let value: serde_json::Value = match serde_json::from_str(&data) {
                Ok(value) => value,
                Err(_) => {
                    return Some(Ok(Bytes::from(format!("data: {}\n\n", data))));
                }
            };
            let choices_empty = value
                .get("choices")
                .and_then(|choices| choices.as_array())
                .map(|choices| choices.is_empty())
                .unwrap_or(true);
            let has_usage = !value
                .get("usage")
                .unwrap_or(&serde_json::Value::Null)
                .is_null();

            if choices_empty && has_usage && !self.include_usage_requested {
                self.send_usage_log(&value);
                return None;
            }
            if choices_empty && has_usage {
                self.send_usage_log(&value);
            }

            let payload = match serde_json::to_string(&value) {
                Ok(value) => value,
                Err(err) => {
                    let io_err = std::io::Error::new(std::io::ErrorKind::Other, err);
                    return Some(Err(io_err));
                }
            };

            Some(Ok(Bytes::from(format!("data: {}\n\n", payload))))
        }

        fn send_usage_log(&mut self, value: &serde_json::Value) {
            if self.log_sent {
                return;
            }
            let usage = extract_usage(value);
            let (input_tokens, output_tokens) = match usage {
                Some((input_tokens, output_tokens)) => (Some(input_tokens), Some(output_tokens)),
                None => (None, None),
            };
            self.log_handler.send(input_tokens, output_tokens, None);
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

    fn extract_usage(value: &serde_json::Value) -> Option<(i64, i64)> {
        let usage = value.get("usage")?;
        let prompt_tokens = usage.get("prompt_tokens")?.as_i64()?;
        if let Some(completion_tokens) = usage.get("completion_tokens").and_then(|v| v.as_i64()) {
            return Some((prompt_tokens, completion_tokens));
        }
        let total_tokens = usage.get("total_tokens")?.as_i64()?;
        let completion_tokens = total_tokens.saturating_sub(prompt_tokens);
        Some((prompt_tokens, completion_tokens))
    }
}

mod gemini_v1beta_request {
    use crate::data::connection::GeminiApiVersion;
    use crate::providers::Transformer;
    use crate::providers::gemini::v1beta::generate_content::request::from_openai_transform::Context as RequestContextV1Beta;
    use crate::providers::gemini::v1beta::generate_content::response::Response as GeminiResponse;
    use crate::providers::gemini::v1beta::generate_content::response::to_openai_transform::Context as ResponseContextV1Beta;
    use crate::providers::openai::chat_completions::request::Request as OpenAiRequest;
    use crate::providers::openai::chat_completions::response::Response as OpenAiResponse;
    use crate::providers::openai::chat_completions::stream as OpenAiStream;
    use crate::providers::utils::generic_post_proxy_request;
    use crate::routes::chat_completions::StreamLogHandler;
    use crate::routes::openai::logging::{RequestLogContext, RequestLogSenders};
    use crate::routes::openai::response::ProxyResponse;
    use bytes::Bytes;
    use chrono::Utc;
    use futures::StreamExt;
    use reqwest::header::HeaderMap;
    use std::collections::HashSet;
    use std::sync::Arc;

    #[tracing::instrument(
        name = "proxy.gemini.v1beta.chat_completions",
        skip(client, api_key, payload)
    )]
    pub(crate) async fn chat_completions(
        client: &reqwest::Client,
        model: &str,
        api_key: &str,
        api_endpoint: &str,
        api_version: &GeminiApiVersion,
        payload: OpenAiRequest,
        stream: bool,
        include_usage_requested: bool,
        request_log_context: Arc<RequestLogContext>,
        senders: RequestLogSenders,
    ) -> ProxyResponse<OpenAiResponse> {
        if stream {
            chat_completions_stream(
                client,
                model,
                api_key,
                api_endpoint,
                api_version,
                payload,
                include_usage_requested,
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
            let generate_url_fn = |loss: crate::providers::gemini::v1beta::generate_content::request::from_openai_transform::Loss| {
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
            let response_context = ResponseContextV1Beta {
                model: Some(model.to_string()),
            };

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

    async fn chat_completions_stream(
        client: &reqwest::Client,
        model: &str,
        api_key: &str,
        api_endpoint: &str,
        api_version: &GeminiApiVersion,
        payload: OpenAiRequest,
        _include_usage_requested: bool,
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
        let generate_url_fn = |loss: crate::providers::gemini::v1beta::generate_content::request::from_openai_transform::Loss| {
            format!(
                "{}/{}/models/{}:streamGenerateContent?alt=sse&key={}",
                endpoint,
                api_version_str,
                loss.model,
                api_key
            )
        };

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
        let stream = gemini_to_openai_stream(response, model.to_string(), log_handler);
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

    fn gemini_to_openai_stream(
        response: reqwest::Response,
        model: String,
        log_handler: StreamLogHandler,
    ) -> impl futures::Stream<Item = Result<Bytes, std::io::Error>> + Send {
        let upstream = response.bytes_stream();
        let state = GeminiStreamState::new(upstream, model, log_handler);

        futures::stream::unfold(state, |mut state| async move {
            loop {
                if let Some(event) = state.next_event() {
                    if let Some(chunk) = state.transform_event(event) {
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
                        if state.done_sent {
                            return None;
                        }
                        state.done_sent = true;
                        let done = Bytes::from("data: [DONE]\n\n");
                        return Some((Ok(done), state));
                    }
                }
            }
        })
    }

    struct GeminiStreamState<S> {
        upstream: S,
        buffer: String,
        model: String,
        response_id: String,
        role_sent: HashSet<u64>,
        done_sent: bool,
        log_handler: StreamLogHandler,
        log_sent: bool,
    }

    impl<S> GeminiStreamState<S> {
        fn new(upstream: S, model: String, log_handler: StreamLogHandler) -> Self {
            Self {
                upstream,
                buffer: String::new(),
                model,
                response_id: "gemini".to_string(),
                role_sent: HashSet::new(),
                done_sent: false,
                log_handler,
                log_sent: false,
            }
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

        fn transform_event(&mut self, event: String) -> Option<Result<Bytes, std::io::Error>> {
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
                return Some(Ok(Bytes::from("data: [DONE]\n\n")));
            }

            let gemini_response: GeminiResponse = match serde_json::from_str(&data) {
                Ok(value) => value,
                Err(err) => {
                    let io_err = std::io::Error::new(std::io::ErrorKind::InvalidData, err);
                    return Some(Err(io_err));
                }
            };
            let chunk = self.build_openai_chunk(gemini_response)?;
            Some(chunk)
        }

        fn build_openai_chunk(
            &mut self,
            response: GeminiResponse,
        ) -> Option<Result<Bytes, std::io::Error>> {
            let candidates = response.candidates?;
            let mut choices = Vec::new();

            for (idx, candidate) in candidates.into_iter().enumerate() {
                let index = candidate.index.unwrap_or(idx as u64);
                let (content, tool_calls) = extract_candidate_delta(&candidate, index);
                let finish_reason = transform_finish_reason(candidate.finish_reason);

                let mut delta = OpenAiStream::ResponseChoiceDelta::default();
                if !self.role_sent.contains(&index) {
                    delta.role = Some("assistant".to_string());
                    self.role_sent.insert(index);
                }
                if let Some(content) = content {
                    delta.content = Some(content);
                }
                if let Some(tool_calls) = tool_calls {
                    delta.tool_calls = Some(tool_calls);
                }

                let finish_reason = finish_reason.map(|value| value.to_string());
                if delta.role.is_none()
                    && delta.content.is_none()
                    && delta.tool_calls.is_none()
                    && finish_reason.is_none()
                {
                    continue;
                }

                choices.push(OpenAiStream::ResponseChoice {
                    index,
                    delta,
                    finish_reason,
                    logprobs: None,
                });
            }

            if choices.is_empty() {
                return None;
            }

            let chunk = OpenAiStream::Response {
                id: response
                    .response_id
                    .unwrap_or_else(|| self.response_id.clone()),
                object: "chat.completion.chunk".to_string(),
                created: 0,
                model: response.model_version.unwrap_or_else(|| self.model.clone()),
                choices,
                system_fingerprint: None,
                usage: None,
            };

            match serde_json::to_string(&chunk) {
                Ok(payload) => Some(Ok(Bytes::from(format!("data: {}\n\n", payload)))),
                Err(err) => Some(Err(std::io::Error::new(std::io::ErrorKind::Other, err))),
            }
        }

        fn finish_log(&mut self) {
            if self.log_sent {
                return;
            }
            self.log_handler.send(None, None, None);
            self.log_sent = true;
        }
    }

    fn extract_candidate_delta(
        candidate: &crate::providers::gemini::v1beta::generate_content::response::Candidate,
        candidate_index: u64,
    ) -> (
        Option<String>,
        Option<Vec<OpenAiStream::ResponseChoiceToolCall>>,
    ) {
        let content = match &candidate.content {
            Some(content) => content,
            None => return (None, None),
        };

        let mut text_parts: Vec<String> = Vec::new();
        let mut tool_calls: Vec<OpenAiStream::ResponseChoiceToolCall> = Vec::new();

        for (idx, part) in content.parts.iter().enumerate() {
            if let Some(text) = &part.text {
                text_parts.push(text.clone());
            }
            if let Some(function_call) = &part.function_call {
                let arguments =
                    serde_json::to_string(&function_call.args).unwrap_or_else(|_| "{}".to_string());
                tool_calls.push(OpenAiStream::ResponseChoiceToolCall {
                    id: format!("gemini-call-{}-{}", candidate_index, idx),
                    tool_type: "function".to_string(),
                    function: OpenAiStream::ResponseChoiceFunctionToolCall {
                        name: function_call.name.clone(),
                        arguments,
                    },
                });
            }
        }

        let content = if text_parts.is_empty() {
            None
        } else {
            Some(text_parts.join(""))
        };
        let tool_calls = if tool_calls.is_empty() {
            None
        } else {
            Some(tool_calls)
        };

        (content, tool_calls)
    }

    fn transform_finish_reason(reason: Option<String>) -> Option<String> {
        match reason.as_deref().map(|value| value.to_ascii_uppercase()) {
            Some(value) if value == "STOP" => Some("stop".to_string()),
            Some(value) if value == "MAX_TOKENS" => Some("length".to_string()),
            Some(value) if value == "SAFETY" => Some("content_filter".to_string()),
            Some(value) if value == "RECITATION" => Some("content_filter".to_string()),
            Some(value) => Some(value.to_ascii_lowercase()),
            None => None,
        }
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
