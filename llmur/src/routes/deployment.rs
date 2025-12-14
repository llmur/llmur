use crate::data::deployment::{Deployment, DeploymentAccess, DeploymentId};
use crate::errors::LLMurError;
use crate::routes::middleware::user_context::{AuthorizationManager, UserContext, UserContextExtractionResult};
use crate::routes::StatusResponse;
use crate::{impl_from_vec_result, LLMurState};
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::data::limits::{BudgetLimits, RequestLimits, TokenLimits};
use crate::data::load_balancer::LoadBalancingStrategy;

// region:    --- Routes
pub(crate) fn routes(state: Arc<LLMurState>) -> Router<Arc<LLMurState>> {
    Router::new()
        .route("/", post(create_deployment))
        .route("/{id}", get(get_deployment))
        .route("/{id}", delete(delete_deployment))
        .with_state(state.clone())
}

#[tracing::instrument(
    name = "handler.create.deployment",
    skip(state, ctx, payload)
)]
pub(crate) async fn create_deployment(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Json(payload): Json<CreateDeploymentPayload>,
) -> Result<Json<GetDeploymentResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;
    match user_context {
        UserContext::MasterUser => {
            let result = state.data.create_deployment(&payload.name, &payload.access.unwrap_or(DeploymentAccess::Private), &payload.strategy.unwrap_or(LoadBalancingStrategy::RoundRobin), &payload.budget_limits, &payload.request_limits, &payload.token_limits).await;
            println!("{:?}", result);
            let deployment = result?;
            Ok(Json(deployment.into()))
        }
        UserContext::WebAppUser { user, .. } => {
            return Err(LLMurError::NotAuthorized)
        }
    }
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
    let user_context = ctx.require_authenticated_user()?;

    let deployment = state.data.get_deployment(&id).await?.ok_or(LLMurError::AdminResourceNotFound)?;

    match user_context {
        UserContext::MasterUser => {
            Ok(Json(deployment.into()))
        }
        UserContext::WebAppUser { user, .. } => {
            Err(LLMurError::NotAuthorized)
        }
    }
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

    let deployment = state.data.get_deployment(&id).await?.ok_or(LLMurError::AdminResourceNotFound)?; // TODO

    match user_context {
        UserContext::MasterUser => {
            let result = state.data.delete_deployment(&deployment.id).await?;
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
pub(crate) struct CreateDeploymentPayload {
    pub(crate) name: String,
    pub(crate) access: Option<DeploymentAccess>,
    pub(crate) strategy: Option<LoadBalancingStrategy>,

    pub(crate) budget_limits: Option<BudgetLimits>,
    pub(crate) request_limits: Option<RequestLimits>,
    pub(crate) token_limits: Option<TokenLimits>,
}

#[derive(Serialize)]
pub(crate) struct GetDeploymentResult {
    pub(crate) id: DeploymentId,
    pub(crate) name: String,
    pub(crate) access: DeploymentAccess
}

#[derive(Serialize)]
pub(crate) struct ListDeploymentsResult {
    pub(crate) deployments: Vec<GetDeploymentResult>,
    pub(crate) total: usize
}

impl_from_vec_result!(GetDeploymentResult, ListDeploymentsResult, deployments);

impl From<Deployment> for GetDeploymentResult {
    fn from(value: Deployment) -> Self {
        GetDeploymentResult {
            id: value.id,
            name: value.name,
            access: value.access,
        }
    }
}
// endregion: --- Data Models