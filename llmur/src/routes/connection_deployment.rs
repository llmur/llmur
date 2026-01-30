use crate::data::connection::ConnectionId;
use crate::data::connection_deployment::{ConnectionDeployment, ConnectionDeploymentId};
use crate::data::deployment::DeploymentId;
use crate::errors::{AuthorizationError, DataAccessError, LLMurError};
use crate::routes::StatusResponse;
use crate::routes::middleware::user_context::{AuthorizationManager, UserContextExtractionResult};
use crate::{LLMurState, impl_from_vec_result};
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// region:    --- Routes
pub(crate) fn routes(state: Arc<LLMurState>) -> Router<Arc<LLMurState>> {
    Router::new()
        .route("/", post(create_connection_deployment))
        .route("/{id}", get(get_connection_deployment))
        .route("/{id}", delete(delete_connection_deployment))
        .with_state(state.clone())
}

#[tracing::instrument(
    name = "handler.create.connection_deployment",
    skip(state, ctx, payload)
)]
pub(crate) async fn create_connection_deployment(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Json(payload): Json<CreateConnectionDeploymentPayload>,
) -> Result<Json<GetConnectionDeploymentResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    if !user_context.has_admin_access() {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let _connection = state
        .data
        .get_connection(
            &payload.connection_id,
            &state.application_secret,
            &state.metrics,
        )
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    let _deployment = state
        .data
        .get_deployment(&payload.deployment_id, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    let result = state
        .data
        .create_connection_deployment(
            &payload.connection_id,
            &payload.deployment_id,
            payload.weight.unwrap_or(1),
            &state.metrics,
        )
        .await?;
    Ok(Json(result.into()))
}

#[tracing::instrument(
    name = "handler.get.connection_deployment",
    skip(state, ctx, id),
    fields(
        id = %id.0,
    )
)]
pub(crate) async fn get_connection_deployment(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<ConnectionDeploymentId>,
) -> Result<Json<GetConnectionDeploymentResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    if !user_context.has_admin_access() {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let cd = state
        .data
        .get_connection_deployment(&id, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    Ok(Json(cd.into()))
}

#[tracing::instrument(
    name = "handler.delete.connection_deployment",
    skip(state, ctx, id),
    fields(
        id = %id.0,
    )
)]
pub(crate) async fn delete_connection_deployment(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<ConnectionDeploymentId>,
) -> Result<Json<StatusResponse>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    if !user_context.has_admin_access() {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let cd = state
        .data
        .get_connection_deployment(&id, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?; // TODO

    let result = state
        .data
        .delete_connection_deployment(&cd.id, &state.metrics)
        .await?;
    Ok(Json(StatusResponse {
        success: result != 0,
        message: None,
    }))
}
// endregion: --- Routes

// region:    --- Data Models
#[derive(Deserialize)]
pub(crate) struct CreateConnectionDeploymentPayload {
    pub(crate) connection_id: ConnectionId,
    pub(crate) deployment_id: DeploymentId,
    pub(crate) weight: Option<i16>,
}

#[derive(Serialize)]
pub(crate) struct GetConnectionDeploymentResult {
    pub(crate) id: ConnectionDeploymentId,
    pub(crate) connection_id: ConnectionId,
    pub(crate) deployment_id: DeploymentId,
}

#[derive(Serialize)]
pub(crate) struct ListConnectionDeploymentsResult {
    pub(crate) maps: Vec<GetConnectionDeploymentResult>,
    pub(crate) total: usize,
}

impl_from_vec_result!(
    GetConnectionDeploymentResult,
    ListConnectionDeploymentsResult,
    maps
);

impl From<ConnectionDeployment> for GetConnectionDeploymentResult {
    fn from(value: ConnectionDeployment) -> Self {
        GetConnectionDeploymentResult {
            id: value.id,
            connection_id: value.connection_id,
            deployment_id: value.deployment_id,
        }
    }
}
// endregion: --- Data Models
