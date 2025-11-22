use std::sync::Arc;
use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::middleware::{from_fn, from_fn_with_state};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use crate::data::graph::Graph;
use crate::errors::LLMurError;
use crate::LLMurState;
use crate::routes::chat_completions::chat_completions_route;
use crate::routes::middlewares::auth_token_extraction_mw::auth_token_extraction_mw;
use crate::routes::middlewares::common_tracing_mw::common_tracing_mw;
use crate::routes::middlewares::openai_route_controller_mw::openai_route_controller_mw;
use crate::routes::middlewares::user_context_load_mw::user_context_load_mw;

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
mod middlewares;
mod extractors;
mod responders;

pub(crate) fn all(state: Arc<LLMurState>) -> Router<Arc<LLMurState>> {
    Router::new()
        .nest("/admin", admin_routes(state.clone()))
        .nest("/v1", openai_v1_routes(state.clone()))
        .with_state(state.clone())
}

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
        .layer(TraceLayer::new_for_http())
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
        .layer(from_fn(auth_token_extraction_mw))
        .layer(from_fn(common_tracing_mw))
        .layer(TraceLayer::new_for_http())
}

#[derive(Serialize)]
pub(crate) struct StatusResponse {
    pub(crate) success: bool,
    pub(crate) message: Option<String>,
}

#[derive(Deserialize)]
struct Params {
    key: String,
    deployment: String,
}

pub(crate) async fn get_graph(
    State(state): State<Arc<LLMurState>>,
    Path(Params { key, deployment }): Path<Params>,
) -> Result<Json<Graph>, LLMurError> {
    let graph = state.data.get_graph(&key, &deployment, false, 10000, &state.application_secret).await;
    println!("{:?}", graph);
    Ok(Json(graph?))
}