use crate::LLMurState;
use crate::errors::{GraphError, LLMurError, MissingConnectionReason, ProxyError};
use crate::providers::{ExposesDeployment, ExposesUsage};
use crate::routes::openai::request::OpenAiRequestData;
use crate::routes::openai::response::{ProviderResponse, ProxyResponse};
use std::collections::BTreeMap;

use crate::data::request_log::{RequestLogData, RequestLogId};
use axum::extract::FromRequest;
use axum::{body::Body, extract::State, http::Request, middleware::Next};

use crate::data::graph::{ConnectionNode, NodeLimitsChecker};
use log::debug;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tracing::Instrument;

#[tracing::instrument(name = "controller", skip(state, request, next))]
pub(crate) async fn openai_route_controller_mw<I, O>(
    State(state): State<Arc<LLMurState>>,
    request: Request<Body>,
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

        let mut attempt_req = Request::new(Body::empty());
        attempt_req
                .extensions_mut()
                .insert(connection.data.connection_info.clone());
        attempt_req
                .extensions_mut()
                .insert(connection.data.id.clone());
        attempt_req
                .extensions_mut()
                .insert(request_data.clone());

            // Copy the RequestLogId to the new request
        attempt_req.extensions_mut().insert(*request_id.as_ref());

        let response = next.clone().run(attempt_req).await;
        state.data.decrement_opened_connection_count(&connection.data.id);

        let result = response
            .extensions()
            .get::<Arc<ProxyResponse<O>>>()
            .ok_or(ProxyError::InternalError("Layers aren't properly setup or route is not returning OpenAiCompatibleResponse<O>".to_string()))?
            .clone();

        let request_log_data_arc = Arc::new(generate_request_log_data::<I, O>(
            *request_id,
            request_data.clone(),
            0,
            connection.clone(),
            result.clone()
        ));

        submit_request_log::<I, O>(
            state.clone(),
            request_log_data_arc,
        );

        Ok::<axum::http::Response<axum::body::Body>, ProxyError>(response)
    }
        .instrument(primary_attempt_span)
        .await?;

    let mut response = result;
    let mut maybe_error = response.extensions_mut().remove::<LLMurError>();

    // If status is OK and Upstream did not emit an error
    if response.status().is_success() && maybe_error.is_none() {
        return Ok(response);
    }

    if let Some(error) = maybe_error {
        return Err(error);
    }

    // Should never get to this point
    Err(ProxyError::InternalError(
        "Invalid controller path".to_string(),
    ))?
}

async fn load_request_details<I>(
    request: Request<Body>,
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

fn generate_request_log_data<I, O>(
    request_id: RequestLogId,
    request_data_arc: Arc<OpenAiRequestData<I>>,
    attempt_number: usize,
    selected_connection_node: ConnectionNode,
    result_arc: Arc<ProxyResponse<O>>,
) -> RequestLogData
where
    I: DeserializeOwned + ExposesDeployment + Send + Sync + 'static,
    O: Serialize + ExposesUsage + Send + 'static + Sync,
{
    match &result_arc.result {
        Ok(inner) => match inner {
            ProviderResponse::DecodedResponse { data, status_code } => RequestLogData {
                id: request_id,
                attempt_number: attempt_number as i16,
                graph: request_data_arc.graph.clone(),
                selected_connection_node,
                input_tokens: Some(data.get_input_tokens() as i64),
                output_tokens: Some(data.get_output_tokens() as i64),
                cost: None,
                http_status_code: status_code.as_u16() as i16,
                error: None,
                request_ts: result_arc.request_ts,
                response_ts: result_arc.response_ts,
                method: request_data_arc.method.clone(),
                path: request_data_arc.path.clone(),
            },
            ProviderResponse::JsonResponse { data, status_code } => RequestLogData {
                id: request_id,
                attempt_number: attempt_number as i16,
                graph: request_data_arc.graph.clone(),
                selected_connection_node,
                input_tokens: None,
                output_tokens: None,
                cost: None,
                http_status_code: status_code.as_u16() as i16,
                error: None,
                request_ts: result_arc.request_ts,
                response_ts: result_arc.response_ts,
                method: request_data_arc.method.clone(),
                path: request_data_arc.path.clone(),
            },
        },
        Err(error) => RequestLogData {
            id: request_id,
            attempt_number: attempt_number as i16,
            graph: request_data_arc.graph.clone(),
            selected_connection_node,
            input_tokens: None,
            output_tokens: None,
            cost: None,
            http_status_code: match error {
                ProxyError::ProxyReturnError(status_code, _) => status_code.as_u16() as i16,
                _ => reqwest::StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i16,
            },
            error: Some(error.to_string()),
            request_ts: result_arc.request_ts,
            response_ts: result_arc.response_ts,
            method: request_data_arc.method.clone(),
            path: request_data_arc.path.clone(),
        },
    }
}

fn submit_request_log<I, O>(state: Arc<LLMurState>, request_log_data_arc: Arc<RequestLogData>) -> ()
where
    I: DeserializeOwned + ExposesDeployment + Send + Sync + 'static,
    O: Serialize + ExposesUsage + Send + 'static + Sync,
{
    // Submit request log to 2 channels - One will add the record in the DB and the other will
    // update the usage stats

    match state
        .data
        .usage_log_tx
        .try_send(request_log_data_arc.clone())
    {
        Ok(_) => {
            println!("### Successfully sent request log to usage channel");
        }
        Err(_) => {
            println!("### Failed to send request log to usage channel");
        }
    };

    match state
        .data
        .request_log_tx
        .try_send(request_log_data_arc.clone())
    {
        Ok(_) => {
            println!("### Successfully sent request log to logging channel");
        }
        Err(_) => {
            println!("### Failed to send request log to logging channel");
        }
    };
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

pub(crate) fn generate_usage_increments_map<I>(
    data: Arc<OpenAiRequestData<I>>,
    used_connection: Arc<ConnectionNode>,
) -> Result<(), ()>
where
    I: DeserializeOwned + ExposesDeployment + Send + Sync + 'static,
{
    let mut map: BTreeMap<String, String> = BTreeMap::new();

    todo!()
    /*
    let mut keys = Vec::with_capacity((self.connections.len() * 3 * 4) + (3 * 3 * 4));
    keys.extend(MetricsUsageStats::<VirtualKey>::generate_all_keys(&self.virtual_key.id, now_utc));
    keys.extend(MetricsUsageStats::<Deployment>::generate_all_keys(&self.deployment.id, now_utc));
    keys.extend(MetricsUsageStats::<Project>::generate_all_keys(&self.project.id, now_utc));
    keys.extend(MetricsUsageStats::<Connection>::generate_all_keys_for_vector(
        self.connections.iter().map(|c| &c.id).collect(),
        now_utc
    ));
    keys
    */
}
