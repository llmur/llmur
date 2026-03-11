use crate::LLMurState;
use crate::data::graph::Graph;
use crate::data::request_log::RequestLogId;
use crate::errors::{ProxyError, ProxyErrorMessage};
use crate::metrics::RegisterProxyRequest;
use crate::providers::ExposesUsage;
use crate::routes::openai::logging::{RequestLogContext, RequestLogSenders, send_request_log};
use crate::routes::openai::response::{ProviderResponse, ProxyResponse};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub(crate) async fn proxy_request_json<T>(
    request: reqwest::RequestBuilder,
) -> Result<ProviderResponse<T>, ProxyError>
where
    T: DeserializeOwned,
{
    let response = request.send().await?;
    let status = response.status();

    if status.is_success() {
        let json_value = response.json::<serde_json::Value>().await?;
        match serde_json::from_value::<T>(json_value.clone()) {
            Ok(decoded) => Ok(ProviderResponse::DecodedResponse {
                data: decoded,
                status_code: status,
            }),
            Err(_) => Ok(ProviderResponse::JsonResponse {
                data: json_value,
                status_code: status,
            }),
        }
    } else {
        let body_bytes = response.bytes().await?;
        match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
            Ok(value) => Err(ProxyError::ProxyReturnError(
                status,
                ProxyErrorMessage::Json(value),
            ))?,
            Err(_) => {
                let text = String::from_utf8_lossy(&body_bytes).to_string();
                Err(ProxyError::ProxyReturnError(
                    status,
                    ProxyErrorMessage::Text(text),
                ))?
            }
        }
    }
}

pub(crate) fn log_proxy_response<T>(
    state: &LLMurState,
    request_id: &RequestLogId,
    graph: &Graph,
    method: &str,
    path: &str,
    subrequest_index: i16,
    response: &ProxyResponse<T>,
) where
    T: Serialize + ExposesUsage + Clone + Send + Sync + 'static,
{
    let request_log_context = RequestLogContext {
        request_id: *request_id,
        graph: graph.clone(),
        selected_connection_node: graph.connection.clone(),
        method: method.to_string(),
        path: path.to_string(),
        subrequest_index,
        request_ts: response.request_ts,
    };
    let senders = RequestLogSenders {
        request_log_tx: state.data.request_log_tx.clone(),
        usage_log_tx: state.data.usage_log_tx.clone(),
    };

    match &response.result {
        Ok(inner) => match inner {
            ProviderResponse::DecodedResponse { data, status_code } => {
                send_request_log(
                    &request_log_context,
                    &senders,
                    *status_code,
                    Some(data.get_input_tokens() as i64),
                    Some(data.get_output_tokens() as i64),
                    None,
                    response.response_ts,
                );
            }
            ProviderResponse::JsonResponse { status_code, .. } => {
                send_request_log(
                    &request_log_context,
                    &senders,
                    *status_code,
                    None,
                    None,
                    None,
                    response.response_ts,
                );
            }
            ProviderResponse::Stream { status_code, .. } => {
                send_request_log(
                    &request_log_context,
                    &senders,
                    *status_code,
                    None,
                    None,
                    None,
                    response.response_ts,
                );
            }
        },
        Err(error) => {
            let status = match error {
                ProxyError::ProxyReturnError(status_code, _) => *status_code,
                _ => reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            };
            send_request_log(
                &request_log_context,
                &senders,
                status,
                None,
                None,
                Some(error.to_string()),
                response.response_ts,
            );
        }
    }
}

pub(crate) fn register_proxy_metrics<T>(
    state: &LLMurState,
    graph: &Graph,
    response: &ProxyResponse<T>,
    path: &str,
    duration_ms: u64,
) where
    T: Serialize + ExposesUsage + Clone + Send + Sync + 'static,
{
    state.metrics.register_proxy_request(
        &graph.deployment.data.id,
        &graph.connection.data.id,
        graph
            .connection
            .data
            .connection_info
            .get_provider_friendly_name()
            .to_string(),
        path.to_string(),
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
        duration_ms,
        response.result.as_ref().map(|r| r.get_status_code()).ok(),
    );
}
