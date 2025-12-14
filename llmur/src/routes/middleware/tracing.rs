use axum::body::Body;
use tracing::Instrument;
use uuid::Uuid;
use crate::data::request_log::RequestLogId;
use axum::extract::Request;
use axum::response::Response;



//static X_REQUEST_ID: HeaderName = HeaderName::from_static("X-LLMur-Request-Id");

// Common tracing middleware that can be applied to all routes - adds request ID and tracing span
// Should be the outermost middleware (i.e., applied last) to wrap all other middlewares and handlers
pub(crate) async fn common_tracing_mw(mut req: Request<Body>, next: axum::middleware::Next) -> Response<Body> {
    let rid = RequestLogId(Uuid::now_v7());
    req.extensions_mut().insert(rid);

    //let res = next.run(req).instrument(span).await;
    let res = next.run(req).await;

    // TODO: Insert X-LLMur-Request-Id header into response

    res
}

