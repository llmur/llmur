use crate::data::deployment::DeploymentId;
use crate::data::virtual_key::VirtualKeyId;
use crate::data::virtual_key_deployment::{VirtualKeyDeployment, VirtualKeyDeploymentId};
use crate::errors::{AuthorizationError, DataAccessError, LLMurError};
use crate::routes::middleware::user_context::{AuthorizationManager, UserContext, UserContextExtractionResult};
use crate::routes::StatusResponse;
use crate::{impl_from_vec_result, LLMurState};
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// region:    --- Routes

pub(crate) fn routes(state: Arc<LLMurState>) -> Router<Arc<LLMurState>> {
    Router::new()
        .route("/", post(create_virtual_key_deployment))
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
    match user_context {
        UserContext::MasterUser => {
            let result = state.data.create_virtual_key_deployment(&payload.virtual_key_id, &payload.deployment_id, &state.metrics).await?;
            Ok(Json(result.into()))
        }
        UserContext::WebAppUser { user, .. } => {
            Err(AuthorizationError::AccessDenied)?
        }
    }
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

    let vkd = state.data.get_virtual_key_deployment(&id, &state.metrics).await?.ok_or(DataAccessError::ResourceNotFound)?;

    match user_context {
        UserContext::MasterUser => {
            Ok(Json(vkd.into()))
        }
        UserContext::WebAppUser { user, .. } => {
            Err(AuthorizationError::AccessDenied)?
        }
    }
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

    let vkd = state.data.get_virtual_key_deployment(&id, &state.metrics).await?.ok_or(DataAccessError::ResourceNotFound)?; // TODO

    match user_context {
        UserContext::MasterUser => {
            let result = state.data.delete_virtual_key_deployment(&vkd.id, &state.metrics).await?;
            Ok(Json(StatusResponse {
                success: result != 0,
                message: None,
            }))
        }
        UserContext::WebAppUser { user, .. } => {
            Err(AuthorizationError::AccessDenied)?
        }
    }
}

// endregion: --- Routes

// region:    --- Data Models
#[derive(Deserialize)]
pub(crate) struct CreateVirtualKeyDeploymentPayload {
    pub(crate) virtual_key_id: VirtualKeyId,
    pub(crate) deployment_id: DeploymentId
}

#[derive(Serialize)]
pub(crate) struct GetVirtualKeyDeploymentResult {
    pub(crate) id: VirtualKeyDeploymentId,
    pub(crate) virtual_key_id: VirtualKeyId,
    pub(crate) deployment_id: DeploymentId
}

#[derive(Serialize)]
pub(crate) struct ListVirtualKeyDeploymentsResult {
    pub(crate) maps: Vec<GetVirtualKeyDeploymentResult>,
    pub(crate) total: usize
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