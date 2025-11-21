use std::any::Any;
use crate::errors::LLMurError;
use crate::providers::{ExposesDeployment, ExposesUsage};
use crate::routes::extractors::graph_extractor::WithGraph;
use crate::routes::responders::openai_responder::OpenAiCompatibleResponse;
use crate::LLMurState;

use axum::{
    body::{Body},
    extract::State,
    http::{Request, header::CONTENT_LENGTH},
    middleware::Next,
};
use http_body_util::BodyExt; // for `.collect()`
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use axum::extract::FromRequest;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc::error::TrySendError;
use log::{trace, warn};
use crate::data::request_log::{RequestLog, RequestLogData, RequestLogId};

pub(crate) async fn openai_request_handler_mw<I, O>(
    State(state): State<Arc<LLMurState>>,
    request: Request<Body>,
    next: Next,
) -> Result<axum::response::Response, LLMurError>
where
    I: DeserializeOwned + ExposesDeployment + Send + Sync + 'static,
    O: Serialize + ExposesUsage + Send + 'static + std::marker::Sync,
{
    println!("Executing OpenAI-compatible request");
    let request_id = &request.extensions().get::<RequestLogId>().cloned().ok_or(LLMurError::InternalServerError(
        "Missing RequestLogId in response extensions".to_string(),
    ))?;

    // Extract WithGraph manually here
    let WithGraph { payload, graph, request } =
        WithGraph::<I>::from_request(request, &state)
            .await?;

    println!("Extracted graph with {} connections", graph.connections.len());
    if graph.connections.is_empty() {
        return Err(LLMurError::NotAuthorized);
    }

    let payload_arc = Arc::new(payload);

    // --- Save request head + body bytes so we can recreate requests ---
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();
    let mut headers = request.headers().clone();
    headers.remove(CONTENT_LENGTH);

    let (.., body) = request.into_parts();
    let body_bytes = body
        .collect()
        .await
        .map_err(|_| LLMurError::BadRequest("Bad Request".to_string()))?
        .to_bytes();

    println!("Attempting requests to {} connections", graph.connections.len());
    // --- Try each connection with a freshly-built Request each time ---
    for connection in &graph.connections {
        println!(" == Attempting request via connection: {:?}", connection.data.connection_info);
        let mut builder = Request::builder()
            .method(method.clone())
            .uri(uri.clone())
            .version(version);

        *builder.headers_mut().unwrap() = headers.clone();

        let mut attempt_req = builder
            .body(Body::from(body_bytes.clone()))
            .map_err(|_| LLMurError::BadRequest("Bad Request 2".to_string()))?;

        attempt_req
            .extensions_mut()
            .insert(connection.data.connection_info.clone());
        attempt_req
            .extensions_mut()
            .insert(Arc::clone(&payload_arc));

        println!("Sending request to upstream. Payload type: {}", std::any::type_name::<I>());

        let request_ts: DateTime<Utc> = Utc::now();
        let mut response = next.clone().run(attempt_req).await;
        let response_ts: DateTime<Utc> = Utc::now();

        println!("Received response from upstream with status: {}", response.status());

        let status = response.status();

        let mut attempt_number = 0;

        if let Some(r) = response.extensions().get::<OpenAiCompatibleResponse<O>>() {
            println!(
                "==== Input tokens: {}, Output tokens: {} ====",
                r.inner.get_input_tokens(),
                r.inner.get_output_tokens()
            );
        }

        let result = response.extensions().get::<OpenAiCompatibleResponse<O>>();
        let error = response.extensions().get::<LLMurError>();
        let had_error = response.extensions().get::<LLMurError>().is_some();


        let record_log = RequestLogData {
            id: request_id.clone(),
            attempt_number,
            graph: graph.clone(),
            attempted_connection_id: connection.data.id,
            input_tokens: result.map(|r| r.inner.get_input_tokens() as i64),
            output_tokens: result.map(|r| r.inner.get_output_tokens() as i64),
            total_tokens: result.map(|r| (r.inner.get_input_tokens() + r.inner.get_output_tokens()) as i64),
            cost: None,
            http_status_code: response.status().as_u16() as i16,
            error: error.map(|e| format!("{:?}", e)),
            request_ts,
            response_ts,
            method: method.to_string(),
            path: uri.to_string(),
        };

        match state.data.request_log_tx.try_send(record_log) {
            Ok(_) => {println!("### Successfully sent request log to logging channel");}
            Err(_) => {println!("### Failed to send request log to logging channel");}
        };

        if status.is_success() && !had_error {
            return Ok(response);
        }

        attempt_number += 1;
    }

    Err(LLMurError::UpstreamUnavailable)
}