use crate::data::connection::ConnectionId;
use crate::data::connection_deployment::{ConnectionDeployment, ConnectionDeploymentId};
use crate::data::deployment::DeploymentId;
use crate::errors::LLMurError;
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
        .route("/", post(create_connection_deployment))
        .route("/{id}", get(get_connection_deployment))
        .route("/{id}", delete(delete_connection_deployment))
        .with_state(state.clone())
}

pub(crate) async fn create_connection_deployment(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Json(payload): Json<CreateConnectionDeploymentPayload>,
) -> Result<Json<GetConnectionDeploymentResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;
    match user_context {
        UserContext::MasterUser => {
            println!("Creating Connection - Deployment");
            let result = state.data.create_connection_deployment(&payload.connection_id, &payload.deployment_id, payload.weight.unwrap_or(1)).await;
            println!("{:?}", result);
            let cd = result?;
            Ok(Json(cd.into()))
        }
        UserContext::WebAppUser { user, .. } => {
            Err(LLMurError::NotAuthorized)
        }
    }
}

pub(crate) async fn get_connection_deployment(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<ConnectionDeploymentId>,
) -> Result<Json<GetConnectionDeploymentResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let cd = state.data.get_connection_deployment(&id).await?.ok_or(LLMurError::AdminResourceNotFound)?;

    match user_context {
        UserContext::MasterUser => {
            Ok(Json(cd.into()))
        }
        UserContext::WebAppUser { user, .. } => {
            Err(LLMurError::NotAuthorized)
        }
    }
}

pub(crate) async fn delete_connection_deployment(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<ConnectionDeploymentId>,
) -> Result<Json<StatusResponse>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let cd = state.data.get_connection_deployment(&id).await?.ok_or(LLMurError::AdminResourceNotFound)?; // TODO

    match user_context {
        UserContext::MasterUser => {
            let result = state.data.delete_connection_deployment(&cd.id).await?;
            Ok(Json(StatusResponse {
                success: result != 0,
                message: None,
            }))
        }
        UserContext::WebAppUser { user, .. } => {
            Err(LLMurError::NotAuthorized)
        }
    }
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
    pub(crate) deployment_id: DeploymentId
}

#[derive(Serialize)]
pub(crate) struct ListConnectionDeploymentsResult {
    pub(crate) maps: Vec<GetConnectionDeploymentResult>,
    pub(crate) total: usize
}

impl_from_vec_result!(GetConnectionDeploymentResult, ListConnectionDeploymentsResult, maps);

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