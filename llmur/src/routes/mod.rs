use crate::data::graph::Graph;
use crate::errors::{GraphError, LLMurError};
use crate::routes::chat_completions::chat_completions_route;
use crate::routes::middleware::auth::auth_token_extraction_mw;
use crate::routes::middleware::user_context::user_context_load_mw;
use crate::routes::openai::controller::openai_route_controller_mw;
use crate::LLMurState;
use axum::extract::{Path, State};
use axum::middleware::{from_fn, from_fn_with_state};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

mod connection;
mod deployment;
mod virtual_key;
mod connection_deployment;
mod virtual_key_deployment;
mod project;
mod project_invite_code;
mod session_token;
mod user;
mod membership;
mod chat_completions;

mod macros;
pub(crate) mod middleware;
pub(crate) mod openai;

pub(crate) fn admin_routes(state: Arc<LLMurState>) -> Router<Arc<LLMurState>> {
    Router::new()
        .nest("/user", user::routes(state.clone()))
        .nest("/session-token", session_token::routes(state.clone()))
        .nest("/project", project::routes(state.clone()))
        .nest("/membership", membership::routes(state.clone()))
        .nest("/project-invite-code", project_invite_code::routes(state.clone()))
        .nest("/connection", connection::routes(state.clone()))
        .nest("/deployment", deployment::routes(state.clone()))
        .nest("/virtual-key", virtual_key::routes(state.clone()))
        .nest("/connection-deployment", connection_deployment::routes(state.clone()))
        .nest("/virtual-key-deployment", virtual_key_deployment::routes(state.clone()))
        .route("/graph/{key}/{deployment}", get(get_graph))
        // Add user context loading middleware - loads user context based on auth info
        .route_layer(from_fn_with_state(state.clone(), user_context_load_mw))
        .with_state(state.clone())
}

pub(crate) fn openai_v1_routes(state: Arc<LLMurState>) -> Router<Arc<LLMurState>> {
    Router::new()
        .route(
            "/chat/completions",
            post(chat_completions_route)
                .route_layer(from_fn_with_state(
                    state.clone(),
                    openai_route_controller_mw::<
                        crate::providers::openai::chat_completions::request::Request,
                        crate::providers::openai::chat_completions::response::Response,
                    >,
                ))
        )
        .with_state(state.clone())
        .route_layer(from_fn(auth_token_extraction_mw))
        //.layer(from_fn_with_state(state.clone(), common_tracing_mw))
}

#[derive(Serialize)]
pub(crate) struct StatusResponse {
    pub(crate) success: bool,
    pub(crate) message: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct Params {
    key: String,
    deployment: String,
}

pub(crate) async fn get_graph(
    State(state): State<Arc<LLMurState>>,
    Path(Params { key, deployment }): Path<Params>,
) -> Result<Json<Graph>, LLMurError> {
    let graph: Result<Graph, GraphError> = state
        .data
        .get_graph(&key, &deployment, false, 10000, &state.application_secret, &Utc::now(), &state.metrics)
        .await
        .map_err(|e| e.into());
    Ok(Json(graph?))
}