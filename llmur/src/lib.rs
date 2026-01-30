//! # LLMUR

use crate::data::DataAccess;
use crate::errors::{LLMurError, UnhealthyStateReason};
use crate::metrics::Metrics;
use crate::routes::{admin_routes, openai_v1_routes};
use axum::extract::State;
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::option::Option;
use std::sync::Arc;
use uuid::Uuid;

pub mod data;
pub mod errors;
pub mod metrics;
pub mod providers;
pub mod routes;

#[derive(Clone)]
pub struct LLMurState {
    pub data: &'static DataAccess,
    pub application_secret: Uuid,
    pub master_keys: BTreeSet<String>,
    pub metrics: Option<Arc<Metrics>>,
}

pub fn router(state: Arc<LLMurState>) -> Router {
    Router::new()
        .nest("/admin", admin_routes(state.clone()))
        .nest("/v1", openai_v1_routes(state.clone()))
        .route("/health", get(health_route))
        .layer(from_fn_with_state(
            state.clone(),
            routes::middleware::common::common_tracing_mw,
        ))
        .with_state(state)
}

#[tracing::instrument(name = "handler.health", skip(state))]
async fn health_route(State(state): State<Arc<LLMurState>>) -> Result<Json<Value>, LLMurError> {
    let poisoned = state.data.cache.local.session_tokens.is_poisoned()
        || state
            .data
            .cache
            .local
            .opened_connections_counter
            .is_poisoned()
        || state.data.cache.local.graphs.is_poisoned()
        || state.data.cache.local.session_tokens.is_poisoned();

    if poisoned {
        return Err(LLMurError::UnhealthyState(
            UnhealthyStateReason::PoisonedLock,
        ))?;
    }

    Ok(Json(json!({"status": "ok"})))
}
