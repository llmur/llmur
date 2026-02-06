use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::data::graph::{ConnectionNode, Graph};
use crate::data::request_log::{RequestLogData, RequestLogId};

#[derive(Clone, Debug)]
pub(crate) struct RequestLogContext {
    pub(crate) request_id: RequestLogId,
    pub(crate) graph: Graph,
    pub(crate) selected_connection_node: ConnectionNode,
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) attempt_number: i16,
    pub(crate) request_ts: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub(crate) struct RequestLogSenders {
    pub(crate) request_log_tx: mpsc::Sender<Arc<RequestLogData>>,
    pub(crate) usage_log_tx: mpsc::Sender<Arc<RequestLogData>>,
}

pub(crate) fn send_request_log(
    context: &RequestLogContext,
    senders: &RequestLogSenders,
    status_code: reqwest::StatusCode,
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
    error: Option<String>,
    response_ts: DateTime<Utc>,
) {
    let data = RequestLogData {
        id: context.request_id,
        attempt_number: context.attempt_number,
        graph: context.graph.clone(),
        selected_connection_node: context.selected_connection_node.clone(),
        input_tokens,
        output_tokens,
        cost: None,
        http_status_code: status_code.as_u16() as i16,
        error,
        request_ts: context.request_ts,
        response_ts,
        method: context.method.clone(),
        path: context.path.clone(),
    };
    let arc = Arc::new(data);
    let _ = senders.usage_log_tx.try_send(arc.clone());
    let _ = senders.request_log_tx.try_send(arc);
}
