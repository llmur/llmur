use crate::LLMurState;
use crate::data::request_log::RequestLogId;
use crate::metrics::RegisterHttpRequest;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::response::Response;
use chrono::Utc;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;
//static X_REQUEST_ID: HeaderName = HeaderName::from_static("X-LLMur-Request-Id");

// Common tracing middleware that can be applied to all routes - adds request ID and tracing span
// Should be the outermost middleware (i.e., applied last) to wrap all other middlewares and handlers
#[tracing::instrument(
    name = "request",
    skip(state, req, next),
    fields(
        method = ?req.method().to_string(),
        path = ?req.uri().path().clone()
    )
)]
pub(crate) async fn common_tracing_mw(
    State(state): State<Arc<LLMurState>>,
    mut req: Request<Body>,
    next: axum::middleware::Next,
) -> Response<Body> {
    let start = Instant::now();
    let now = Utc::now();
    let rid = RequestLogId(Uuid::now_v7());

    let method = req.method().clone();
    let uri = req.uri().clone();

    req.extensions_mut().insert(rid);
    req.extensions_mut().insert(now);

    let res = next.run(req).await;

    state.metrics.register_http_request(
        uri.path().to_string(),
        method.to_string(),
        start.elapsed().as_millis() as u64,
    );

    res
}
