use crate::LLMurState;
use crate::errors::{GraphError, LLMurError, MissingConnectionReason, ProxyError};
use crate::providers::{ExposesDeployment, ExposesUsage};
use crate::routes::openai::request::OpenAiRequestData;
use crate::routes::openai::response::{ProviderResponse, ProxyResponse};
use crate::routes::openai::logging::{RequestLogContext, RequestLogSenders, send_request_log};

use crate::data::request_log::RequestLogId;
use axum::extract::FromRequest;
use axum::{extract::State, http::Request, middleware::Next};

use crate::data::graph::{ConnectionNode, NodeLimitsChecker};
use log::debug;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use chrono::Utc;
use tracing::Instrument;

#[tracing::instrument(name = "controller", skip(state, request, next))]
pub(crate) async fn openai_route_controller_mw<I, O>(
    State(state): State<Arc<LLMurState>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<axum::response::Response, LLMurError>
where
    I: DeserializeOwned + ExposesDeployment + Send + Sync + 'static,
    O: Serialize + ExposesUsage + Send + 'static + Sync,
{
    println!("Executing openai_route_controller_mw");
    let (request_id, request_data) = load_request_details::<I>(request, state.clone()).await?;

    if request_data.graph.connections.is_empty() {
        Err(GraphError::NoConnectionAvailable(
            MissingConnectionReason::DeploymentConnectionsNotSetup,
        ))?;
    }

    validate_usage(Arc::clone(&request_data))?;

    let connection = state.data.get_next_connection(&request_data.graph)?;
    state
        .data
        .increment_opened_connection_count(&connection.data.id);

    // Create a child span for this attempt
    let primary_attempt_span = tracing::debug_span!(
        "attempt",
        attempt = 0,
        connection_id = ?connection.data.id.0
    );

    let result = async {
        debug!(
            "Attempting primary request via connection: {:?}",
            connection.data.connection_info
        );

        let mut attempt_req = Request::new(axum::body::Body::empty());
        attempt_req
                .extensions_mut()
                .insert(connection.data.connection_info.clone());
        attempt_req
                .extensions_mut()
                .insert(connection.data.id);
        attempt_req
                .extensions_mut()
                .insert(request_data.clone());

            // Copy the RequestLogId to the new request
        attempt_req.extensions_mut().insert(*request_id.as_ref());
        attempt_req.extensions_mut().insert(RequestLogContext {
            request_id: *request_id.as_ref(),
            graph: request_data.graph.clone(),
            selected_connection_node: connection.clone(),
            method: request_data.method.clone(),
            path: request_data.path.clone(),
            attempt_number: 0,
            request_ts: Utc::now(),
        });

        let response = next.clone().run(attempt_req).await;
        state.data.decrement_opened_connection_count(&connection.data.id);

        let result = response
            .extensions()
            .get::<Arc<ProxyResponse<O>>>()
            .ok_or(ProxyError::InternalError("Layers aren't properly setup or route is not returning OpenAiCompatibleResponse<O>".to_string()))?
            .clone();

        let is_stream = matches!(result.result, Ok(ProviderResponse::Stream { .. }));
        if !is_stream {
            let senders = RequestLogSenders {
                request_log_tx: state.data.request_log_tx.clone(),
                usage_log_tx: state.data.usage_log_tx.clone(),
            };
            let response_ts = result.response_ts;
            match &result.result {
                Ok(inner) => match inner {
                    ProviderResponse::DecodedResponse { data, status_code } => {
                        send_request_log(
                            &request_data_to_log_context(request_id.as_ref(), &request_data, &connection, result.request_ts),
                            &senders,
                            *status_code,
                            Some(data.get_input_tokens() as i64),
                            Some(data.get_output_tokens() as i64),
                            None,
                            response_ts,
                        );
                    }
                    ProviderResponse::JsonResponse { status_code, .. } => {
                        send_request_log(
                            &request_data_to_log_context(request_id.as_ref(), &request_data, &connection, result.request_ts),
                            &senders,
                            *status_code,
                            None,
                            None,
                            None,
                            response_ts,
                        );
                    }
                    ProviderResponse::Stream { status_code, .. } => {
                        send_request_log(
                            &request_data_to_log_context(request_id.as_ref(), &request_data, &connection, result.request_ts),
                            &senders,
                            *status_code,
                            None,
                            None,
                            None,
                            response_ts,
                        );
                    }
                },
                Err(error) => {
                    let status = match error {
                        ProxyError::ProxyReturnError(status_code, _) => *status_code,
                        _ => reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                    };
                    send_request_log(
                        &request_data_to_log_context(request_id.as_ref(), &request_data, &connection, result.request_ts),
                        &senders,
                        status,
                        None,
                        None,
                        Some(error.to_string()),
                        response_ts,
                    );
                }
            }
        }

        Ok::<axum::http::Response<axum::body::Body>, ProxyError>(response)
    }
        .instrument(primary_attempt_span)
        .await?;

    let mut response = result;
    let maybe_error = response.extensions_mut().remove::<LLMurError>();

    if let Some(error) = maybe_error {
        return Err(error);
    }

    Ok(response)
}

async fn load_request_details<I>(
    request: Request<axum::body::Body>,
    state: Arc<LLMurState>,
) -> Result<(Arc<RequestLogId>, Arc<OpenAiRequestData<I>>), LLMurError>
where
    I: DeserializeOwned + ExposesDeployment + Send + Sync + 'static,
{
    println!("Executing OpenAI-compatible request");
    let request_id =
        request
            .extensions()
            .get::<RequestLogId>()
            .cloned()
            .ok_or(ProxyError::InternalError(
                "Missing RequestLogId in request extensions".to_string(),
            ))?;
    let request_data = OpenAiRequestData::<I>::from_request(request, &state).await?;

    Ok((Arc::new(request_id), Arc::new(request_data)))
}

fn request_data_to_log_context<I>(
    request_id: &RequestLogId,
    request_data_arc: &Arc<OpenAiRequestData<I>>,
    selected_connection_node: &ConnectionNode,
    request_ts: chrono::DateTime<chrono::Utc>,
) -> RequestLogContext
where
    I: DeserializeOwned + ExposesDeployment + Send + Sync + 'static,
{
    RequestLogContext {
        request_id: *request_id,
        attempt_number: 0,
        graph: request_data_arc.graph.clone(),
        selected_connection_node: selected_connection_node.clone(),
        method: request_data_arc.method.clone(),
        path: request_data_arc.path.clone(),
        request_ts,
    }
}

fn validate_usage<I>(data: Arc<OpenAiRequestData<I>>) -> Result<(), GraphError>
where
    I: DeserializeOwned + ExposesDeployment + Send + Sync + 'static,
{
    data.graph.virtual_key.validate_limits()?;
    data.graph.project.validate_limits()?;
    data.graph.deployment.validate_limits()?;

    Ok(())
}

/*
pub(crate) fn generate_usage_increments_map<I>(
data: Arc<OpenAiRequestData<I>>,
used_connection: Arc<ConnectionNode>,
) -> Result<(), ()>
where
I: DeserializeOwned + ExposesDeployment + Send + Sync + 'static,
{
let mut map: BTreeMap<String, String> = BTreeMap::new();

todo!()

let mut keys = Vh_capacity((self.connections.len() * 3 * 4) + (3 * 3 * 4));
keys.extend(MetricsUsageStats::<VirtualKey>::generate_all_keys(&self.virtual_key.id, now_utc));
keys.extend(MetricsUsageStats::<Deployment>::generate_all_keys(&self.deployment.id, now_utc));
keys.extend(MetricsUsageStats::<Project>::generate_all_keys(&self.project.id, now_utc));
keys.extend(MetricsUsageStats::<Connection>::generate_all_keys_for_vector(
    self.connections.iter().map(|c| &c.id).collect(),
    now_utc
));
keys
}
*/
