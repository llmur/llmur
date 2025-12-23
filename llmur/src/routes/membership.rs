use crate::data::membership::{Membership, MembershipId};
use crate::data::project::{ProjectId, ProjectRole};
use crate::data::user::UserId;
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
        .route("/", post(create_membership))
        .route("/{id}", get(get_membership))
        .route("/{id}", delete(delete_membership))
        .with_state(state.clone())
}

#[tracing::instrument(
    name = "handler.create.membership",
    skip(state, ctx, payload)
)]
pub(crate) async fn create_membership(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Json(payload): Json<CreateMembershipPayload>,
) -> Result<Json<GetMembershipResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;
    match user_context {
        UserContext::MasterUser => {
            let membership = state.data.create_membership(
                &payload.user_id,
                &payload.project_id,
                &payload.role.unwrap_or(ProjectRole::Guest),
                &state.metrics,
            ).await?;

            Ok(Json(membership.into()))
        }
        UserContext::WebAppUser { user, .. } => {
            Err(AuthorizationError::AccessDenied)?
        }
    }
}

#[tracing::instrument(
    name = "handler.get.membership",
    skip(state, ctx, id),
    fields(
        id = %id.0,
    )
)]
pub(crate) async fn get_membership(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<MembershipId>,
) -> Result<Json<GetMembershipResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    match user_context {
        UserContext::MasterUser => {
            let membership = state.data.get_membership(&id, &state.metrics).await?.ok_or(DataAccessError::ResourceNotFound)?;
            Ok(Json(membership.into()))
        }
        UserContext::WebAppUser { user, .. } => {
            // TODO
            Err(AuthorizationError::AccessDenied)?
        }
    }
}

#[tracing::instrument(
    name = "handler.delete.membership",
    skip(state, ctx, id),
    fields(
        id = %id.0,
    )
)]
pub(crate) async fn delete_membership(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<MembershipId>,
) -> Result<Json<StatusResponse>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let membership = state.data.get_membership(&id, &state.metrics).await?.ok_or(DataAccessError::ResourceNotFound)?;

    match user_context {
        UserContext::MasterUser => {
            let result = state.data.delete_membership(&membership.id, &state.metrics).await?;
            Ok(Json(StatusResponse {
                success: result != 0,
                message: None,
            }))
        }
        UserContext::WebAppUser { user, .. } => {
            // TODO
            Err(AuthorizationError::AccessDenied)?
        }
    }
}

// endregion: --- Routes

// region:    --- Data Models
#[derive(Deserialize)]
pub(crate) struct CreateMembershipPayload {
    pub(crate) user_id: UserId,
    pub(crate) project_id: ProjectId,
    pub(crate) role: Option<ProjectRole>
}

#[derive(Serialize)]
pub(crate) struct GetMembershipResult {
    pub(crate) id: MembershipId,
    pub(crate) user_id: UserId,
    pub(crate) project_id: ProjectId,
    pub(crate) role: ProjectRole
}

#[derive(Serialize)]
pub(crate) struct ListMembershipsResult {
    pub(crate) memberships: Vec<GetMembershipResult>,
    pub(crate) total: usize
}

impl_from_vec_result!(GetMembershipResult, ListMembershipsResult, memberships);

impl From<Membership> for GetMembershipResult {
    fn from(value: Membership) -> Self {
        GetMembershipResult {
            id: value.id,
            user_id: value.user_id,
            project_id: value.project_id,
            role: value.role,
        }
    }
}
// endregion: --- Data Models