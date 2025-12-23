use crate::data::limits::{BudgetLimits, RequestLimits, TokenLimits};
use crate::data::project::{Project, ProjectId};
use crate::errors::{AuthorizationError, DataAccessError, LLMurError};
use crate::routes::middleware::user_context::{AuthorizationManager, UserContextExtractionResult};
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
        // Project routes
        .route("/", post(create_project))
        .route("/{id}", get(get_project))
        .route("/{id}", delete(delete_project))
        .with_state(state.clone())
}

#[tracing::instrument(
    name = "handler.create.project",
    skip(state, ctx, payload)
)]
pub(crate) async fn create_project(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Json(payload): Json<CreateProjectPayload>,
) -> Result<Json<GetProjectResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    if !user_context.has_admin_access() {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let project = state
        .data
        .create_project(&payload.name, &user_context.get_user_id(), &payload.budget_limits, &payload.request_limits, &payload.token_limits, &state.metrics)
        .await?;

    Ok(Json(project.into()))
}

#[tracing::instrument(
    name = "handler.get.project",
    skip(state, ctx, id),
    fields(
        id = %id.0,
    )
)]
pub(crate) async fn get_project(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<ProjectId>,
) -> Result<Json<GetProjectResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    if !user_context.has_project_member_access(state.clone(), &id).await? {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let project = state
        .data
        .get_project(&id, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    Ok(Json(project.into()))
}

#[tracing::instrument(
    name = "handler.delete.project",
    skip(state, ctx, id),
    fields(
        id = %id.0,
    )
)]
pub(crate) async fn delete_project(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<ProjectId>,
) -> Result<Json<StatusResponse>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let project = state
        .data
        .get_project(&id, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    if !user_context.has_project_admin_access(state.clone(), &id).await? {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let result = state.data.delete_project(&project.id, &state.metrics).await?;
    Ok(Json(StatusResponse {
        success: result != 0,
        message: None,
    }))
}


// endregion: --- Routes

// region:    --- Data Models
#[derive(Deserialize)]
pub(crate) struct CreateProjectPayload {
    pub(crate) name: String,
    
    pub(crate) budget_limits: Option<BudgetLimits>,
    pub(crate) request_limits: Option<RequestLimits>,
    pub(crate) token_limits: Option<TokenLimits>,
}

#[derive(Serialize)]
pub(crate) struct GetProjectResult {
    pub(crate) id: ProjectId,
    pub(crate) name: String,
}

#[derive(Serialize)]
pub(crate) struct ListProjectsResult {
    pub(crate) projects: Vec<GetProjectResult>,
    pub(crate) total: usize
}

impl_from_vec_result!(GetProjectResult, ListProjectsResult, projects);

impl From<Project> for GetProjectResult {
    fn from(value: Project) -> Self {
        GetProjectResult {
            id: value.id,
            name: value.name,
        }
    }
}
// endregion: --- Data Models