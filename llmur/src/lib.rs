//! # LLMUR

use crate::metrics::Metrics;
use crate::routes::{admin_routes, openai_v1_routes};
use axum::middleware::from_fn_with_state;
use axum::Router;
use data::DataAccess;
use std::collections::BTreeSet;
use std::option::Option;
use std::sync::Arc;
use uuid::Uuid;

pub mod providers;
pub mod data;
pub mod routes;
pub mod errors;
pub mod metrics;

#[derive(Clone)]
pub struct LLMurState {
    pub data: &'static DataAccess,
    pub application_secret: Uuid,
    pub master_keys: BTreeSet<String>,
    pub metrics: Option<Arc<Metrics>>
}

pub fn router(state: Arc<LLMurState>/*, application_secret: String, master_keys: Option<BTreeSet<String>>, meter: Option<Meter>*/) -> Router {
    /*
    let state: Arc<LLMurState> = Arc::new(LLMurState {
        data: Box::leak(Box::new(access)),
        application_secret: utils::new_uuid_v5_from_string(&application_secret),
        master_keys: master_keys.unwrap_or_default(),
        metrics: meter.map(|m| Arc::new(Metrics::new(m))),
    });
     */

    Router::new()
        .nest("/admin", admin_routes(state.clone()))
        .nest("/v1", openai_v1_routes(state.clone()))
        .layer(from_fn_with_state(state.clone(), crate::routes::middleware::common::common_tracing_mw))
        .with_state(state)
}