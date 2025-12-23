use crate::data::project::{ProjectId, ProjectRole};
use crate::data::project_invite_code::{ProjectInviteCode, ProjectInviteCodeId};
use crate::data::utils::current_timestamp_s;
use crate::errors::{AuthorizationError, DataAccessError, LLMurError};
use crate::routes::StatusResponse;
use crate::routes::middleware::user_context::{
    AuthorizationManager, UserContext, UserContextExtractionResult,
};
use crate::{LLMurState, impl_from_vec_result};
use axum::extract::{Path, State};
use axum::routing::{delete, post};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// region:    --- Routes
pub(crate) fn routes(state: Arc<LLMurState>) -> Router<Arc<LLMurState>> {
    Router::new()
        .route("/", post(create_project_invite_code))
        .route("/{id}", delete(delete_project_invite_code))
        .with_state(state.clone())
}

#[tracing::instrument(name = "handler.create.project_invite_code", skip(state, ctx, payload))]
pub(crate) async fn create_project_invite_code(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Json(payload): Json<CreateProjectInviteCodePayload>,
) -> Result<Json<GetProjectInviteCodeResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    if !user_context
        .has_project_admin_access(state.clone(), &payload.project_id)
        .await?
    {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let invite = state
        .data
        .create_invite_code(
            &payload.project_id,
            &payload.role.unwrap_or(ProjectRole::Guest),
            &payload.validity,
            &payload.code_length,
            &state.metrics,
        )
        .await?;

    Ok(Json(invite.into()))
}

#[tracing::instrument(
    name = "handler.delete.project_invite_code",
    skip(state, ctx, id),
    fields(
        id = %id.0,
    )
)]
pub(crate) async fn delete_project_invite_code(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<ProjectInviteCodeId>,
) -> Result<Json<StatusResponse>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let project_invite_code = state
        .data
        .get_invite_code(&id, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    if !user_context.has_project_admin_access(state.clone(), &project_invite_code.project_id).await? {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let result = state.data.delete_invite_code(&id, &state.metrics).await?;

    Ok(Json(StatusResponse {
        success: result != 0,
        message: None,
    }))
}
// endregion: --- Routes

// region:    --- Data Models
#[derive(Deserialize)]
pub(crate) struct CreateProjectInviteCodePayload {
    pub(crate) project_id: ProjectId,
    pub(crate) validity: Option<String>,
    pub(crate) role: Option<ProjectRole>,
    pub(crate) code_length: Option<usize>,
}

#[derive(Serialize)]
pub(crate) struct GetProjectInviteCodeResult {
    pub(crate) id: ProjectInviteCodeId,
    pub(crate) project_id: ProjectId,
    pub(crate) code: String,
    pub(crate) valid: bool,
    pub(crate) valid_until: Option<i64>,
    pub(crate) role: ProjectRole,
}

#[derive(Serialize)]
pub(crate) struct ListProjectInviteCodesResult {
    pub(crate) codes: Vec<GetProjectInviteCodeResult>,
    pub(crate) total: usize,
}

impl_from_vec_result!(
    GetProjectInviteCodeResult,
    ListProjectInviteCodesResult,
    codes
);

impl From<ProjectInviteCode> for GetProjectInviteCodeResult {
    fn from(value: ProjectInviteCode) -> Self {
        GetProjectInviteCodeResult {
            id: value.id,
            project_id: value.project_id,
            code: value.code,
            valid: value
                .valid_until
                .map(|vu| vu < current_timestamp_s())
                .unwrap_or(true),
            valid_until: value.valid_until,
            role: value.assign_role,
        }
    }
}
// endregion: --- Data Models
