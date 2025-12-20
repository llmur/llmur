use crate::data::connection::{ConnectionId, ConnectionInfo};
use crate::providers::openai::chat_completions::request::Request as ChatCompletionsRequest;
use crate::providers::openai::chat_completions::response::Response as ChatCompletionsResponse;
use crate::routes::openai::request::OpenAiRequestData;
use crate::routes::openai::response::ProxyResponse;
use crate::LLMurState;
use axum::extract::State;
use axum::Extension;
use std::sync::Arc;
use std::time::Instant;
use crate::metrics::RegisterProxyRequest;

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
) -> ProxyResponse<ChatCompletionsResponse> {
    println!("== Executing Chat Completions request");
    let start = Instant::now();

    let response = match &connection_info {
        ConnectionInfo::AzureOpenAiApiKey { api_key, api_endpoint, api_version, deployment_name } => {
            azure_openai_request::chat_completions(
                &state.data.http_client,
                deployment_name,
                api_key,
                api_endpoint,
                api_version,
                request.payload.clone(),
            ).await
        }
        ConnectionInfo::OpenAiApiKey { api_key, api_endpoint, model } => {
            openai_v1_request::chat_completions(
                &state.data.http_client,
                model,
                api_key,
                api_endpoint,
                request.payload.clone(),
            ).await
        }
    };

    state.metrics.register_proxy_request(
        &request.graph.deployment.data.id,
        &connection_id,
        connection_info.get_provider_friendly_name().to_string(),
        request.path.clone(),
        response.result.as_ref().map(|r| r.get_input_tokens()).unwrap_or_default(),
        response.result.as_ref().map(|r| r.get_output_tokens()).unwrap_or_default(),
        start.elapsed().as_millis() as u64,
        response.result.as_ref().map(|r| r.get_status_code()).ok()
    );

    response
}

mod azure_openai_request {
    use crate::data::connection::AzureOpenAiApiVersion;
    use crate::providers::azure::openai::v2024_02_01::chat_completions::request::from_openai_transform::Context as RequestContextV2024_02_01;
    use crate::providers::azure::openai::v2024_02_01::chat_completions::response::to_openai_transform::Context as ResponseContextV2024_02_01;
    use crate::providers::openai::chat_completions::request::Request as OpenAiRequest;
    use crate::providers::openai::chat_completions::response::Response as OpenAiResponse;
    use crate::providers::utils::generic_post_proxy_request;
    use crate::routes::openai::response::ProxyResponse;
    use chrono::Utc;
    use reqwest::header::HeaderMap;
    use std::sync::Arc;

    #[tracing::instrument(
        name = "proxy.azure.openai.chat_completions",
        skip(client, api_key, payload)
    )]
    pub(crate) async fn chat_completions(client: &reqwest::Client, deployment_name: &str, api_key: &str, api_endpoint: &str, api_version: &AzureOpenAiApiVersion, payload: OpenAiRequest) -> ProxyResponse<OpenAiResponse> {
        let mut headers = HeaderMap::new();
        headers.insert("api-key", api_key.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        let generate_url_fn = |_| { format!("{}/openai/deployments/{}/chat/completions?api-version={}", api_endpoint, deployment_name, api_version.to_string()) };

        match api_version {
            AzureOpenAiApiVersion::V2024_02_01 => {
                let request_context = RequestContextV2024_02_01 { data_sources: None };
                let response_context = ResponseContextV2024_02_01 { model: Some(payload.model.clone()) };

                let start_ts = Utc::now();
                match generic_post_proxy_request(
                        client,
                        payload,
                        request_context,
                        generate_url_fn,
                        headers,
                        response_context,
                    ).await {
                    Ok(response) => {
                        ProxyResponse::new(Ok(response), start_ts)
                    }
                    Err(error) => {
                        ProxyResponse::new(
                            Err(error), start_ts
                        )
                    }
                }
            }
            AzureOpenAiApiVersion::V2024_06_01 => {
                let request_context = RequestContextV2024_02_01 { data_sources: None };
                let response_context = ResponseContextV2024_02_01 { model: Some(payload.model.clone()) };

                let start_ts = Utc::now();
                match generic_post_proxy_request(
                    client,
                    payload,
                    request_context,
                    generate_url_fn,
                    headers,
                    response_context,
                ).await {
                    Ok(response) => {
                        ProxyResponse::new(Ok(response), start_ts)
                    }
                    Err(error) => {
                        ProxyResponse::new(
                            Err(error), start_ts
                        )
                    }
                }
            }
        }
    }
}

mod openai_v1_request {
    use crate::providers::openai::chat_completions::request::to_self::Context as RequestContext;
    use crate::providers::openai::chat_completions::request::Request as OpenAiRequest;
    use crate::providers::openai::chat_completions::response::to_self::Context as ResponseContext;
    use crate::providers::openai::chat_completions::response::Response as OpenAiResponse;
    use crate::providers::utils::generic_post_proxy_request;
    use crate::routes::openai::response::ProxyResponse;
    use chrono::Utc;
    use reqwest::header::HeaderMap;
    use std::sync::Arc;


    #[tracing::instrument(
        name = "proxy.openai.v1.chat_completions",
        skip(client, api_key, payload)
    )]
    pub(crate) async fn chat_completions(client: &reqwest::Client, model: &str, api_key: &str, api_endpoint: &str, payload: OpenAiRequest) -> ProxyResponse<OpenAiResponse> {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", api_key).parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        let generate_url_fn = |_| { format!("{}/v1/chat/completions", api_endpoint) };

        let request_context = RequestContext { model: Some(model.to_string()) };
        let response_context = ResponseContext { model: Some(payload.model.clone()) };

        let start_ts = Utc::now();
        match generic_post_proxy_request(
            client,
            payload,
            request_context,
            generate_url_fn,
            headers,
            response_context,
        ).await {
            Ok(response) => {
                ProxyResponse::new(Ok(response), start_ts)
            }
            Err(error) => {
                ProxyResponse::new(
                    Err(error), start_ts
                )
            }
        }
    }
}