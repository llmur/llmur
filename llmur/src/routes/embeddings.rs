use crate::data::connection::{ConnectionId, ConnectionInfo};
use crate::errors::{ProxyError, ProxyErrorMessage};
use crate::metrics::RegisterProxyRequest;
use crate::providers::openai::embeddings::request::{EmbeddingsInput, Request as EmbeddingsRequest};
use crate::providers::openai::embeddings::response::Response as EmbeddingsResponse;
use crate::routes::openai::request::OpenAiRequestData;
use crate::routes::openai::response::ProxyResponse;
use crate::LLMurState;
use axum::extract::State;
use axum::Extension;
use chrono::Utc;
use std::sync::Arc;
use std::time::Instant;

#[tracing::instrument(
    name = "handler.openai.v1.embeddings",
    skip(state, connection_info, request),
    fields(
        model = %request.graph.deployment.data.name
    )
)]
pub(crate) async fn embeddings_route(
    State(state): State<Arc<LLMurState>>,
    Extension(connection_info): Extension<ConnectionInfo>,
    Extension(connection_id): Extension<ConnectionId>,
    Extension(request): Extension<Arc<OpenAiRequestData<EmbeddingsRequest>>>,
) -> ProxyResponse<EmbeddingsResponse> {
    let start = Instant::now();

    if matches!(
        &request.payload.input,
        EmbeddingsInput::TokenArray(_) | EmbeddingsInput::TokenArrayBatch(_)
    ) {
        if !matches!(&connection_info, ConnectionInfo::OpenAiApiKey { .. }) {
            return ProxyResponse::new(
                Err(ProxyError::ProxyReturnError(
                    reqwest::StatusCode::BAD_REQUEST,
                    ProxyErrorMessage::Text(
                        "Token array embeddings input is only supported for OpenAI connections."
                            .to_string(),
                    ),
                )),
                Utc::now(),
            );
        }
    }

    let response = match &connection_info {
        ConnectionInfo::AzureOpenAiApiKey {
            api_key,
            api_endpoint,
            api_version,
            deployment_name,
        } => {
            azure_openai_request::embeddings(
                &state.data.http_client,
                deployment_name,
                api_key,
                api_endpoint,
                api_version,
                request.payload.clone(),
            )
            .await
        }
        ConnectionInfo::OpenAiApiKey {
            api_key,
            api_endpoint,
            model,
        } => {
            openai_v1_request::embeddings(
                &state.data.http_client,
                model,
                api_key,
                api_endpoint,
                request.payload.clone(),
            )
            .await
        }
        ConnectionInfo::GeminiApiKey {
            api_key,
            api_endpoint,
            api_version,
            model,
        } => {
            gemini_v1beta_request::embeddings(
                &state.data.http_client,
                model,
                api_key,
                api_endpoint,
                api_version,
                request.payload.clone(),
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
    use crate::providers::azure::openai::v2024_10_21::embeddings::request::from_openai_transform::Context as RequestContextV2024_10_21;
    use crate::providers::azure::openai::v2024_10_21::embeddings::response::to_openai_transform::Context as ResponseContextV2024_10_21;
    use crate::providers::openai::embeddings::request::Request as OpenAiRequest;
    use crate::providers::openai::embeddings::response::Response as OpenAiResponse;
    use crate::providers::utils::generic_post_proxy_request;
    use crate::routes::openai::response::ProxyResponse;
    use chrono::Utc;
    use reqwest::header::HeaderMap;

    #[tracing::instrument(
        name = "proxy.azure.openai.embeddings",
        skip(client, api_key, payload)
    )]
    pub(crate) async fn embeddings(
        client: &reqwest::Client,
        deployment_name: &str,
        api_key: &str,
        api_endpoint: &str,
        api_version: &AzureOpenAiApiVersion,
        payload: OpenAiRequest,
    ) -> ProxyResponse<OpenAiResponse> {
        let mut headers = HeaderMap::new();
        headers.insert("api-key", api_key.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        let generate_url_fn = |_| {
            format!(
                "{}/openai/deployments/{}/embeddings?api-version={}",
                api_endpoint,
                deployment_name,
                api_version.to_string()
            )
        };

        match api_version {
            AzureOpenAiApiVersion::V2024_10_21 => {
                let request_context = RequestContextV2024_10_21 { input_type: None };
                let response_context = ResponseContextV2024_10_21 { model: Some(payload.model.clone()) };

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
    }
}

mod openai_v1_request {
    use crate::providers::openai::embeddings::request::to_self::Context as RequestContext;
    use crate::providers::openai::embeddings::request::Request as OpenAiRequest;
    use crate::providers::openai::embeddings::response::to_self::Context as ResponseContext;
    use crate::providers::openai::embeddings::response::Response as OpenAiResponse;
    use crate::providers::utils::generic_post_proxy_request;
    use crate::routes::openai::response::ProxyResponse;
    use chrono::Utc;
    use reqwest::header::HeaderMap;

    #[tracing::instrument(
        name = "proxy.openai.v1.embeddings",
        skip(client, api_key, payload)
    )]
    pub(crate) async fn embeddings(
        client: &reqwest::Client,
        model: &str,
        api_key: &str,
        api_endpoint: &str,
        payload: OpenAiRequest,
    ) -> ProxyResponse<OpenAiResponse> {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", api_key).parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        let generate_url_fn = |_| format!("{}/v1/embeddings", api_endpoint);

        let request_context = RequestContext {
            model: Some(model.to_string()),
        };
        let response_context = ResponseContext {
            model: Some(payload.model.clone()),
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

mod gemini_v1beta_request {
    use crate::data::connection::GeminiApiVersion;
    use crate::providers::gemini::v1beta::embed_content::request::from_openai_transform::Context as RequestContextV1Beta;
    use crate::providers::gemini::v1beta::embed_content::response::to_openai_transform::Context as ResponseContextV1Beta;
    use crate::providers::openai::embeddings::request::Request as OpenAiRequest;
    use crate::providers::openai::embeddings::response::Response as OpenAiResponse;
    use crate::providers::utils::generic_post_proxy_request;
    use crate::routes::openai::response::ProxyResponse;
    use chrono::Utc;
    use reqwest::header::HeaderMap;

    #[tracing::instrument(
        name = "proxy.gemini.v1beta.embeddings",
        skip(client, api_key, payload)
    )]
    pub(crate) async fn embeddings(
        client: &reqwest::Client,
        model: &str,
        api_key: &str,
        api_endpoint: &str,
        api_version: &GeminiApiVersion,
        payload: OpenAiRequest,
    ) -> ProxyResponse<OpenAiResponse> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        let api_version_str = match api_version {
            GeminiApiVersion::V1BETA => "v1beta",
        };
        let endpoint = api_endpoint.trim_end_matches('/');
        let generate_url_fn =
            |loss: crate::providers::gemini::v1beta::embed_content::request::from_openai_transform::Loss| {
                format!(
                    "{}/{}/models/{}:embedContent?key={}",
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
            model: Some(payload.model.clone()),
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
