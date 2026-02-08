use crate::data::connection::ConnectionId;
use crate::data::deployment::{Deployment, DeploymentAccess, DeploymentId};
use crate::data::limits::{BudgetLimits, RequestLimits, TokenLimits};
use crate::errors::{AuthorizationError, DataAccessError, LLMurError};
use crate::routes::StatusResponse;
use crate::routes::middleware::user_context::{AuthorizationManager, UserContextExtractionResult};
use crate::routes::utils::{NullableField, normalize_limits};
use crate::{LLMurState, impl_from_vec_result};
use axum::extract::{Path, State};
use axum::routing::{delete, get, patch, post};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// region:    --- Routes
pub(crate) fn routes(state: Arc<LLMurState>) -> Router<Arc<LLMurState>> {
    Router::new()
        .route("/", post(create_deployment))
        .route("/{id}", get(get_deployment))
        .route("/{id}", patch(update_deployment))
        .route("/{id}", delete(delete_deployment))
        .with_state(state.clone())
}

#[tracing::instrument(name = "handler.create.deployment", skip(state, ctx, payload))]
pub(crate) async fn create_deployment(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Json(payload): Json<CreateDeploymentPayload>,
) -> Result<Json<GetDeploymentResult>, LLMurError> {
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

    let result = state
        .data
        .create_deployment(
            &payload.name,
            &payload.access.unwrap_or(DeploymentAccess::Private),
            &payload.connection_id,
            &payload.budget_limits,
            &payload.request_limits,
            &payload.token_limits,
            &state.metrics,
        )
        .await?;

    Ok(Json(result.into()))
}

#[tracing::instrument(
    name = "handler.update.deployment",
    skip(state, ctx, id, payload),
    fields(id = %id.0)
)]
pub(crate) async fn update_deployment(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<DeploymentId>,
    Json(payload): Json<UpdateDeploymentPayload>,
) -> Result<Json<GetDeploymentResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    if !user_context.has_admin_access() {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let _deployment = state
        .data
        .get_deployment(&id, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    if let Some(connection_id) = &payload.connection_id {
        let _connection = state
            .data
            .get_connection(connection_id, &state.application_secret, &state.metrics)
            .await?
            .ok_or(DataAccessError::ResourceNotFound)?;
    }

    let UpdateDeploymentPayload {
        access,
        connection_id,
        budget_limits,
        request_limits,
        token_limits,
    } = payload;

    let budget_limits = normalize_limits(budget_limits);
    let request_limits = normalize_limits(request_limits);
    let token_limits = normalize_limits(token_limits);

    let result = state
        .data
        .update_deployment(
            &id,
            &access,
            &connection_id,
            &budget_limits,
            &request_limits,
            &token_limits,
            &state.metrics,
        )
        .await?;

    Ok(Json(result.into()))
}

#[tracing::instrument(
    name = "handler.get.deployment",
    skip(state, ctx, id),
    fields(
        id = %id.0,
    )
)]
pub(crate) async fn get_deployment(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<DeploymentId>,
) -> Result<Json<GetDeploymentResult>, LLMurError> {
    let _ = ctx.require_authenticated_user()?;

    let deployment = state
        .data
        .get_deployment(&id, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    Ok(Json(deployment.into()))
}

#[tracing::instrument(
    name = "handler.delete.deployment",
    skip(state, ctx, id),
    fields(
        id = %id.0,
    )
)]
pub(crate) async fn delete_deployment(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<DeploymentId>,
) -> Result<Json<StatusResponse>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    if !user_context.has_admin_access() {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let deployment = state
        .data
        .get_deployment(&id, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    let result = state
        .data
        .delete_deployment(&deployment.id, &state.metrics)
        .await?;

    Ok(Json(StatusResponse {
        success: result != 0,
        message: None,
    }))
}
// endregion: --- Routes

// region:    --- Data Models
#[derive(Deserialize)]
pub(crate) struct CreateDeploymentPayload {
    pub(crate) name: String,
    pub(crate) access: Option<DeploymentAccess>,
    pub(crate) connection_id: ConnectionId,

    pub(crate) budget_limits: Option<BudgetLimits>,
    pub(crate) request_limits: Option<RequestLimits>,
    pub(crate) token_limits: Option<TokenLimits>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateDeploymentPayload {
    pub(crate) access: Option<DeploymentAccess>,
    pub(crate) connection_id: Option<ConnectionId>,

    #[serde(default)]
    pub(crate) budget_limits: NullableField<BudgetLimits>,
    #[serde(default)]
    pub(crate) request_limits: NullableField<RequestLimits>,
    #[serde(default)]
    pub(crate) token_limits: NullableField<TokenLimits>,
}

#[derive(Serialize)]
pub(crate) struct GetDeploymentResult {
    pub(crate) id: DeploymentId,
    pub(crate) name: String,
    pub(crate) access: DeploymentAccess,
    pub(crate) connection_id: ConnectionId,
}

#[derive(Serialize)]
pub(crate) struct ListDeploymentsResult {
    pub(crate) deployments: Vec<GetDeploymentResult>,
    pub(crate) total: usize,
}

impl_from_vec_result!(GetDeploymentResult, ListDeploymentsResult, deployments);

impl From<Deployment> for GetDeploymentResult {
    fn from(value: Deployment) -> Self {
        GetDeploymentResult {
            id: value.id,
            name: value.name,
            access: value.access,
            connection_id: value.connection_id,
        }
    }
}
// endregion: --- Data Models
