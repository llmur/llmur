use crate::data::deployment::DeploymentId;
use crate::data::virtual_key::VirtualKeyId;
use crate::data::virtual_key_deployment::{VirtualKeyDeployment, VirtualKeyDeploymentId};
use crate::errors::{AuthorizationError, DataAccessError, LLMurError};
use crate::routes::middleware::user_context::{AuthorizationManager, UserContextExtractionResult};
use crate::routes::StatusResponse;
use crate::{impl_from_vec_result, LLMurState};
use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// region:    --- Routes

pub(crate) fn routes(state: Arc<LLMurState>) -> Router<Arc<LLMurState>> {
    Router::new()
        .route("/", post(create_virtual_key_deployment))
        .route("/", get(search_virtual_key_deployments))
        .route("/{id}", get(get_virtual_key_deployment))
        .route("/{id}", delete(delete_virtual_key_deployment))
        .with_state(state.clone())
}

#[tracing::instrument(
    name = "handler.create.virtual_key",
    skip(state, ctx, payload)
)]
pub(crate) async fn create_virtual_key_deployment(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Json(payload): Json<CreateVirtualKeyDeploymentPayload>,
) -> Result<Json<GetVirtualKeyDeploymentResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let virtual_key = state
        .data
        .get_virtual_key(&payload.virtual_key_id, &state.application_secret, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    if !user_context.has_project_admin_access(state.clone(), &virtual_key.project_id).await? {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let result = state
        .data
        .create_virtual_key_deployment(&payload.virtual_key_id, &payload.deployment_id, &state.metrics)
        .await?;

    Ok(Json(result.into()))
}

#[tracing::instrument(
    name = "handler.get.virtual_key",
    skip(state, ctx, id),
    fields(
        id = %id.0,
    )
)]
pub(crate) async fn get_virtual_key_deployment(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<VirtualKeyDeploymentId>,
) -> Result<Json<GetVirtualKeyDeploymentResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let vkd = state
        .data
        .get_virtual_key_deployment(&id, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    let virtual_key = state
        .data
        .get_virtual_key(&vkd.virtual_key_id, &state.application_secret, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    if !user_context.has_project_developer_access(state.clone(), &virtual_key.project_id).await? {
        return Err(AuthorizationError::AccessDenied)?;
    }

    Ok(Json(vkd.into()))
}

#[tracing::instrument(
    name = "handler.delete.virtual_key",
    skip(state, ctx, id),
    fields(
        id = %id.0,
    )
)]
pub(crate) async fn delete_virtual_key_deployment(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<VirtualKeyDeploymentId>,
) -> Result<Json<StatusResponse>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let vkd = state
        .data
        .get_virtual_key_deployment(&id, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    let virtual_key = state
        .data
        .get_virtual_key(&vkd.virtual_key_id, &state.application_secret, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    if !user_context.has_project_admin_access(state.clone(), &virtual_key.project_id).await? {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let result = state
        .data
        .delete_virtual_key_deployment(&vkd.id, &state.metrics)
        .await?;

    Ok(Json(StatusResponse {
        success: result != 0,
        message: None,
    }))
}


#[tracing::instrument(
    name = "handler.search.virtual_key",
    skip(state, ctx, params)
)]
pub(crate) async fn search_virtual_key_deployments(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Query(params): Query<Option<SearchVirtualKeyDeploymentQueryParams>>,
) -> Result<Json<ListVirtualKeyDeploymentsResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let virtual_key_id = params.as_ref().and_then(|p| p.virtual_key_id);
    let deployment_id = params.as_ref().and_then(|p| p.deployment_id);

    // If the virtual key is passed as a parameter we need to check if the user has access
    // to the project it belongs to
    if let Some(virtual_key_id) = virtual_key_id {
        let virtual_key = state
            .data
            .get_virtual_key(&virtual_key_id, &state.application_secret, &state.metrics)
            .await?
            .ok_or(DataAccessError::ResourceNotFound)?;

        if !user_context.has_project_developer_access(state.clone(), &virtual_key.project_id).await? {
            return Err(AuthorizationError::AccessDenied)?;
        }
    } else if !user_context.has_admin_access() {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let result = state
        .data
        .search_virtual_key_deployments(
            &virtual_key_id,
            &deployment_id,
            &state.metrics,
        ).await?
        .into_iter()
        .map(Into::<GetVirtualKeyDeploymentResult>::into)
        .collect::<Vec<GetVirtualKeyDeploymentResult>>()
        .into();

    Ok(Json(result))
}


// endregion: --- Routes

// region:    --- Data Models
#[derive(Deserialize)]
pub(crate) struct CreateVirtualKeyDeploymentPayload {
    pub(crate) virtual_key_id: VirtualKeyId,
    pub(crate) deployment_id: DeploymentId,
}

#[derive(Deserialize)]
pub(crate) struct SearchVirtualKeyDeploymentQueryParams {
    pub(crate) virtual_key_id: Option<VirtualKeyId>,
    pub(crate) deployment_id: Option<DeploymentId>
}

#[derive(Serialize)]
pub(crate) struct GetVirtualKeyDeploymentResult {
    pub(crate) id: VirtualKeyDeploymentId,
    pub(crate) virtual_key_id: VirtualKeyId,
    pub(crate) deployment_id: DeploymentId,
}

#[derive(Serialize)]
pub(crate) struct ListVirtualKeyDeploymentsResult {
    pub(crate) maps: Vec<GetVirtualKeyDeploymentResult>,
    pub(crate) total: usize,
}

impl_from_vec_result!(GetVirtualKeyDeploymentResult, ListVirtualKeyDeploymentsResult, maps);

impl From<VirtualKeyDeployment> for GetVirtualKeyDeploymentResult {
    fn from(value: VirtualKeyDeployment) -> Self {
        GetVirtualKeyDeploymentResult {
            id: value.id,
            virtual_key_id: value.virtual_key_id,
            deployment_id: value.deployment_id,
        }
    }
}
// endregion: --- Data Models