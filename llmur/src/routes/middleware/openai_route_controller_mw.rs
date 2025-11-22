use crate::LLMurState;
use crate::errors::LLMurError;
use crate::providers::{ExposesDeployment, ExposesUsage};
use crate::routes::extract::openai_request_data::OpenAiRequestData;
use crate::routes::response::openai_responder::OpenAiCompatibleResponse;

use crate::data::request_log::{RequestLogData, RequestLogId};
use axum::extract::FromRequest;
use axum::{
    body::Body,
    extract::State,
    http::{Request, header::CONTENT_LENGTH},
    middleware::Next,
};
use chrono::{DateTime, Utc};
use http_body_util::BodyExt;

use crate::data::graph::{ConnectionNode, Graph};
use log::trace;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tracing::Instrument;
use uuid::Timestamp;
use crate::data::connection::ConnectionId;

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
        return Err(LLMurError::NotAuthorized);
    }

    // --- Try each connection with a freshly-built Request each time ---
    for (attempt_number, connection) in request_data.graph.connections.iter().enumerate() {
        // Create a child span for this attempt
        let attempt_span = tracing::info_span!(
            "upstream.attempt",
            attempt = attempt_number,
            connection_id = ?connection.data.id
        );

        let result = async {
            println!(
                " == Attempting request via connection: {:?}",
                connection.data.connection_info
            );

            let mut attempt_req = Request::new(Body::empty());
            attempt_req
                .extensions_mut()
                .insert(connection.data.connection_info.clone());
            attempt_req
                .extensions_mut()
                .insert(request_data.clone());

            // Copy the RequestLogId to the new request
            attempt_req.extensions_mut().insert(*request_id.as_ref());

            println!(
                "Sending request to upstream. Payload type: {}",
                std::any::type_name::<I>()
            );

            let request_ts: DateTime<Utc> = Utc::now();
            let mut response = next.clone().run(attempt_req).await;
            let response_ts: DateTime<Utc> = Utc::now();

            println!(
                "Received response from upstream with status: {}",
                response.status()
            );

            let result = response
                .extensions()
                .get::<OpenAiCompatibleResponse<O>>()
                .ok_or(LLMurError::InternalServerError("Failed to load result".to_string()))
                .expect("If we get to this point layers aren't properly setup or route is not returning OpenAiCompatibleResponse<O>");

            submit_request_log::<I, O>(
                state.clone(),
                request_id.clone(),
                request_data.clone(),
                attempt_number,
                connection,
                &result,
                &response,
                &request_ts,
                &response_ts,
            );

            (response)
        }
            .instrument(attempt_span)
            .await;

        let (response) = result;

        // If status is OK and Upstream did not emit an error
        if response.status().is_success() && !response.extensions().get::<LLMurError>().is_some() {
            return Ok(response);
        }
    }

    Err(LLMurError::UpstreamUnavailable)
}

async fn load_request_details<I>(
    request: Request<Body>,
    state: Arc<LLMurState>,
) -> Result<(Arc<RequestLogId>, Arc<OpenAiRequestData<I>>), LLMurError>
where
    I: DeserializeOwned + ExposesDeployment + Send + Sync + 'static,
{
    println!("Executing OpenAI-compatible request");
    let request_id = request.extensions().get::<RequestLogId>().cloned().ok_or(
        LLMurError::InternalServerError("Missing RequestLogId in request extensions".to_string())
    )?;
    let request_data = OpenAiRequestData::<I>::from_request(request, &state).await?;

    Ok((Arc::new(request_id), Arc::new(request_data)))
}

fn submit_request_log<I, O>(
    state: Arc<LLMurState>,
    request_id: Arc<RequestLogId>,
    request_data: Arc<OpenAiRequestData<I>>,
    attempt_number: usize,
    connection: &ConnectionNode,
    result: &OpenAiCompatibleResponse<O>,
    response: &axum::response::Response,
    request_ts: &DateTime<Utc>,
    response_ts: &DateTime<Utc>
) -> ()
where
    I: DeserializeOwned + ExposesDeployment + Send + Sync + 'static,
    O: Serialize + ExposesUsage + Send + 'static + Sync,
{
    //let result = response.extensions().get::<OpenAiCompatibleResponse<O>>();
    //let error = response.extensions().get::<LLMurError>();

    let record_log = match &result.result {
        Ok(inner) => {
            RequestLogData {
                id: (*request_id).clone(),
                attempt_number: attempt_number as i16,
                graph: (request_data.graph).clone(),
                attempted_connection_id: connection.data.id,
                input_tokens: Some(inner.get_input_tokens() as i64),
                output_tokens: Some(inner.get_output_tokens() as i64),
                cost: None,
                http_status_code: response.status().as_u16() as i16,
                error: None,
                request_ts: *request_ts,
                response_ts: *response_ts,
                method: request_data.method.clone(),
                path: request_data.path.clone(),
            }
        }
        Err(error) => {
            RequestLogData {
                id: (*request_id).clone(),
                attempt_number: attempt_number as i16,
                graph: (request_data.graph).clone(),
                attempted_connection_id: connection.data.id,
                input_tokens: None,
                output_tokens: None,
                cost: None,
                http_status_code: response.status().as_u16() as i16,
                error: None,
                request_ts: *request_ts,
                response_ts: *response_ts,
                method: request_data.method.clone(),
                path: request_data.path.clone(),
            }
        }
    };

    match state.data.request_log_tx.try_send(record_log) {
        Ok(_) => {
            println!("### Successfully sent request log to logging channel");
        }
        Err(_) => {
            println!("### Failed to send request log to logging channel");
        }
    };
}
