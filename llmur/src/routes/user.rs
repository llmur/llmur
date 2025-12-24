use crate::data::user::{ApplicationRole, User, UserId};
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
        .route("/me", get(get_current_user))
        .route("/", post(create_user))
        .route("/{id}", get(get_user))
        .route("/{id}", delete(delete_user))
        //.route("/", get(list_users))
        //.route("/:id/memberships", get(get_user_memberships))
        //.route("/:id", put(update_user))

        .with_state(state.clone())
}

#[tracing::instrument(
    name = "handler.create.user",
    skip(state, ctx, payload)
)]
pub(crate) async fn create_user(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<Json<GetUserResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    if !user_context.is_master_user() {
        return Err(AuthorizationError::AccessDenied)?
    }

    let user = state.data.create_user(
        &payload.email,
        &payload.name,
        &payload.password,
        payload.email_verified.unwrap_or(false),
        payload.blocked.unwrap_or(false),
        &payload.role.unwrap_or(ApplicationRole::Member),
        &state.application_secret,
        &state.metrics
    ).await?;

    Ok(Json(user.into()))
}

#[tracing::instrument(
    name = "handler.create.user",
    skip(state, ctx, id),
    fields(
        id = %id.0,
    )
)]
pub(crate) async fn get_user(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<UserId>
) -> Result<Json<GetUserResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    match user_context {
        UserContext::MasterUser => {
            let user = state.data.get_user(&id, &state.metrics).await?.ok_or(DataAccessError::ResourceNotFound)?;
            Ok(Json(user.into()))
        }
        UserContext::WebAppUser { user, .. } => {
            if user.id == id || user.role == ApplicationRole::Admin {
                Ok(Json(user.into()))
            }
            else { Err(AuthorizationError::AccessDenied)? }
        }
    }
}

#[tracing::instrument(
    name = "handler.get.user",
    skip(ctx)
)]
pub(crate) async fn get_current_user(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(_): State<Arc<LLMurState>>
) -> Result<Json<GetUserResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    match user_context {
        UserContext::MasterUser => {
            Err(AuthorizationError::AccessDenied)?
        }
        UserContext::WebAppUser { user, .. } => {
            Ok(Json(user.into()))
        }
    }
}

#[tracing::instrument(
    name = "handler.delete.user",
    skip(state, ctx, id),
    fields(
        id = %id.0,
    )
)]
pub(crate) async fn delete_user(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<UserId>,
) -> Result<Json<StatusResponse>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let result = match user_context {
        UserContext::MasterUser => {
            state.data.delete_user(&id, &state.metrics).await?
        }
        UserContext::WebAppUser { user, .. } => {
            if id == user.id { state.data.delete_user(&id, &state.metrics).await? }
            else { Err(AuthorizationError::AccessDenied)? }
        }
    };

    if result == 0 {
        Err(DataAccessError::ResourceNotFound)?
    }
    else {
        Ok(Json(StatusResponse { success: true, message: Some(format!("User {} deleted successfully", &id)) }))
    }
}
// endregion: --- Routes

// region:    --- Data Models
#[derive(Deserialize, Debug)]
pub(crate) struct CreateUserPayload {
    pub(crate) email: String,
    pub(crate) password: String,
    pub(crate) name: Option<String>,
    pub(crate) blocked: Option<bool>,
    pub(crate) email_verified: Option<bool>,
    pub(crate) role: Option<ApplicationRole>
}

#[derive(Serialize)]
pub(crate) struct GetUserResult {
    pub(crate) id: UserId,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) blocked: bool,
    pub(crate) role: ApplicationRole
}

#[derive(Serialize)]
pub(crate) struct ListUsersResult {
    pub(crate) users: Vec<GetUserResult>,
    pub(crate) total: usize
}

impl_from_vec_result!(GetUserResult, ListUsersResult, users);

impl From<User> for GetUserResult {
    fn from(value: User) -> Self {
        GetUserResult {
            id: value.id,
            name: value.name,
            email: value.email,
            blocked: value.blocked,
            role: value.role,
        }
    }
}
// endregion: --- Data Models