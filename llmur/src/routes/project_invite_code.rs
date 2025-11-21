use crate::data::project::{ProjectId, ProjectRole};
use crate::data::project_invite_code::{ProjectInviteCode, ProjectInviteCodeId};
use crate::data::utils::current_timestamp_s;
use crate::errors::LLMurError;
use crate::routes::middlewares::user_context_load_mw::{AuthorizationManager, UserContext, UserContextExtractionResult};
use crate::routes::StatusResponse;
use crate::{impl_from_vec_result, LLMurState};
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

pub(crate) async fn create_project_invite_code(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Json(payload): Json<CreateProjectInviteCodePayload>,
) -> Result<Json<GetProjectInviteCodeResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    match user_context {
        UserContext::MasterUser => {
            let invite = state.data.create_invite_code(
                &payload.project_id,
                &payload.role.unwrap_or(ProjectRole::Guest),
                &payload.validity,
                &payload.code_length,
            ).await?;

            Ok(Json(invite.into()))
        }
        UserContext::WebAppUser { user, .. } => {
            /*
            let memberships = state.data.projects()
                .get_project_memberships(&payload.project_id)
                .await?
                .ok_or(LLMurError::AdminResourceNotFound)?;

            if let Some(membership) = memberships.iter().find(|m| m.user_id == user.id) {
                if membership.role == ProjectRole::Admin {
                    let result = state.data.invites().create_invite(
                        &payload.project_id,
                        &payload.role.unwrap_or(ProjectRole::Guest),
                        &payload.validity,
                        &payload.code_length,
                    ).await?;
                    return Ok(Json(StatusResponse {
                        success: true,
                        message: Some(format!("Invite code {} created successfully", result.id)),
                    }));
                }
            }

             */

            Err(LLMurError::NotAuthorized)
        }
    }
}

pub(crate) async fn delete_project_invite_code(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<ProjectInviteCodeId>,
) -> Result<Json<StatusResponse>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;


    match user_context {
        UserContext::MasterUser => {
            let result = state.data.delete_invite_code(&id).await?;

            Ok(Json(StatusResponse {
                success: result != 0,
                message: None,
            }))
        }
        UserContext::WebAppUser { user, .. } => {
            /*
            let maybe_invite = state.data.invites().get_invite_with_id(&id).await?;

            if let Some(invite) = maybe_invite {
                let memberships = state.data.projects()
                    .get_project_memberships(&invite.project_id)
                    .await?
                    .ok_or(LLMurError::AdminResourceNotFound)?;

                if let Some(membership) = memberships.iter().find(|m| m.user_id == user.id) {
                    if membership.role == ProjectRole::Admin {
                        let result = state.data.invites().delete_invite(&id).await?;
                        return Ok(Json(StatusResponse {
                            success: result != 0,
                            message: None,
                        }));
                    }
                }
            }

             */

            Err(LLMurError::NotAuthorized)
        }
    }
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
    pub(crate) total: usize
}

impl_from_vec_result!(GetProjectInviteCodeResult, ListProjectInviteCodesResult, codes);

impl From<ProjectInviteCode> for GetProjectInviteCodeResult {
    fn from(value: ProjectInviteCode) -> Self {
        GetProjectInviteCodeResult {
            id: value.id,
            project_id: value.project_id,
            code: value.code,
            valid: value.valid_until.map(|vu| vu  < current_timestamp_s()).unwrap_or(true),
            valid_until: value.valid_until,
            role: value.assign_role,
        }
    }
}
// endregion: --- Data Models