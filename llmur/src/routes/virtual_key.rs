use crate::data::limits::{BudgetLimits, RequestLimits, TokenLimits};
use crate::data::membership::{Membership, MembershipId};
use crate::data::project::{ProjectId, ProjectRole};
use crate::data::user::ApplicationRole;
use crate::data::virtual_key::{VirtualKey, VirtualKeyId};
use crate::errors::{AuthorizationError, DataAccessError, LLMurError};
use crate::routes::StatusResponse;
use crate::routes::middleware::user_context::{
    AuthorizationManager, UserContext, UserContextExtractionResult,
};
use crate::{LLMurState, impl_from_vec_result};
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

// region:    --- Routes
pub(crate) fn routes(state: Arc<LLMurState>) -> Router<Arc<LLMurState>> {
    Router::new()
        .route("/", post(create_key))
        //.route("/", get(search_keys))
        .route("/{id}", get(get_key))
        .route("/{id}", delete(delete_key))
        .with_state(state.clone())
}

#[tracing::instrument(name = "handler.create.virtual_key", skip(state, ctx, payload))]
pub(crate) async fn create_key(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Json(payload): Json<CreateVirtualKeyPayload>,
) -> Result<Json<GetVirtualKeyResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;
    match user_context {
        UserContext::MasterUser => {
            let key = state
                .data
                .create_virtual_key(
                    32,
                    &payload.alias,
                    &payload.description,
                    false,
                    &payload.project_id,
                    &payload.budget_limits,
                    &payload.request_limits,
                    &payload.token_limits,
                    &state.application_secret,
                    &state.metrics,
                )
                .await?;

            Ok(Json(key.into()))
        }
        UserContext::WebAppUser { user, .. } => {
            // Allow if the user is a service admin
            if user.role == ApplicationRole::Admin || {
                // Or if the user is an admin of the project
                let memberships: BTreeMap<MembershipId, Membership> = state
                    .data
                    .get_memberships(&user.memberships, &state.metrics)
                    .await?
                    .into_iter()
                    .filter_map(|(k, v)| v.map(|val| (k, val)))
                    .collect();

                memberships
                    .values()
                    .find(|&v| v.project_id == payload.project_id)
                    .map(|membership| membership.role == ProjectRole::Admin)
                    .unwrap_or(false)
            } {
                let key = state
                    .data
                    .create_virtual_key(
                        32,
                        &payload.alias,
                        &payload.description,
                        false,
                        &payload.project_id,
                        &payload.budget_limits,
                        &payload.request_limits,
                        &payload.token_limits,
                        &state.application_secret,
                        &state.metrics,
                    )
                    .await?;

                Ok(Json(key.into()))
            } else {
                Err(AuthorizationError::AccessDenied)?
            }
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
pub(crate) async fn get_key(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<VirtualKeyId>,
) -> Result<Json<GetVirtualKeyResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let key = state
        .data
        .get_virtual_key(&id, &state.application_secret, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    match user_context {
        UserContext::MasterUser => Ok(Json(key.into())),
        UserContext::WebAppUser { user, .. } => {
            // Allow if the user is a service admin
            if user.role == ApplicationRole::Admin || {
                // Or if the user is an admin or developer of the project that the key belongs to
                let memberships: BTreeMap<MembershipId, Membership> = state
                    .data
                    .get_memberships(&user.memberships, &state.metrics)
                    .await?
                    .into_iter()
                    .filter_map(|(k, v)| v.map(|val| (k, val)))
                    .collect();

                memberships
                    .values()
                    .find(|&v| v.project_id == key.project_id)
                    .map(|membership| {
                        membership.role == ProjectRole::Admin
                            || membership.role == ProjectRole::Developer
                    })
                    .unwrap_or(false)
            } {
                Ok(Json(key.into()))
            } else {
                Err(AuthorizationError::AccessDenied)?
            }
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
pub(crate) async fn delete_key(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<VirtualKeyId>,
) -> Result<Json<StatusResponse>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let key = state
        .data
        .get_virtual_key(&id, &state.application_secret, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    match user_context {
        UserContext::MasterUser => {
            let result = state
                .data
                .delete_virtual_key(&key.id, &state.metrics)
                .await?;
            Ok(Json(StatusResponse {
                success: result != 0,
                message: None,
            }))
        }
        UserContext::WebAppUser { user, .. } => {
            // Allow if the user is a service admin
            if user.role == ApplicationRole::Admin || {
                // Or if the user is an admin of the project that the key belongs to
                let memberships: BTreeMap<MembershipId, Membership> = state
                    .data
                    .get_memberships(&user.memberships, &state.metrics)
                    .await?
                    .into_iter()
                    .filter_map(|(k, v)| v.map(|val| (k, val)))
                    .collect();

                memberships
                    .values()
                    .find(|&v| v.project_id == key.project_id)
                    .map(|membership| membership.role == ProjectRole::Admin)
                    .unwrap_or(false)
            } {
                let result = state
                    .data
                    .delete_virtual_key(&key.id, &state.metrics)
                    .await?;
                Ok(Json(StatusResponse {
                    success: result != 0,
                    message: None,
                }))
            } else {
                Err(AuthorizationError::AccessDenied)?
            }
        }
    }
}

// endregion: --- Routes

// region:    --- Data Models
#[derive(Deserialize)]
pub(crate) struct CreateVirtualKeyPayload {
    pub(crate) project_id: ProjectId,
    pub(crate) alias: Option<String>,
    pub(crate) description: Option<String>,

    pub(crate) budget_limits: Option<BudgetLimits>,
    pub(crate) request_limits: Option<RequestLimits>,
    pub(crate) token_limits: Option<TokenLimits>,
}

#[derive(Serialize)]
pub(crate) struct GetVirtualKeyResult {
    pub(crate) id: VirtualKeyId,
    pub(crate) key: String,
    pub(crate) alias: String,
    pub(crate) blocked: bool,
    pub(crate) project_id: ProjectId,
}

#[derive(Serialize)]
pub(crate) struct ListVirtualKeysResult {
    pub(crate) keys: Vec<GetVirtualKeyResult>,
    pub(crate) total: usize,
}

impl_from_vec_result!(GetVirtualKeyResult, ListVirtualKeysResult, keys);

impl From<VirtualKey> for GetVirtualKeyResult {
    fn from(value: VirtualKey) -> Self {
        GetVirtualKeyResult {
            id: value.id,
            key: value.key,
            alias: value.alias,
            blocked: value.blocked,
            project_id: value.project_id,
        }
    }
}
// endregion: --- Data Models
