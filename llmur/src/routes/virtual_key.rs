use std::sync::Arc;
use axum::{Extension, Json, Router};
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use serde::{Deserialize, Serialize};
use crate::errors::{DataAccessError, LLMurError};
use crate::{impl_from_vec_result, LLMurState};
use crate::data::project::ProjectId;
use crate::data::virtual_key::{VirtualKey, VirtualKeyId};
use crate::routes::middleware::user_context_load_mw::{AuthorizationManager, UserContext, UserContextExtractionResult};
use crate::routes::StatusResponse;

// region:    --- Routes
pub(crate) fn routes(state: Arc<LLMurState>) -> Router<Arc<LLMurState>> {
    Router::new()
        .route("/", post(create_key))
        //.route("/", get(search_keys))
        .route("/{id}", get(get_key))
        .route("/{id}", delete(delete_key))
        .with_state(state.clone())
}

pub(crate) async fn create_key(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Json(payload): Json<CreateVirtualKeyPayload>,
) -> Result<Json<GetVirtualKeyResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;
    match user_context {
        UserContext::MasterUser => {

            let key = state.data.create_virtual_key(
                32,
                &payload.alias,
                &payload.description,
                false,
                &payload.project_id,
                &state.application_secret).await?;


            Ok(Json(key.into()))
        }
        UserContext::WebAppUser { user, .. } => {
            Err(LLMurError::NotAuthorized)
        }
    }
}

pub(crate) async fn get_key(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<VirtualKeyId>,
) -> Result<Json<GetVirtualKeyResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let key = state.data.get_virtual_key(&id, &state.application_secret).await?.ok_or(LLMurError::AdminResourceNotFound)?;

    match user_context {
        UserContext::MasterUser => {
            Ok(Json(key.into()))
        }
        UserContext::WebAppUser { user, .. } => {
            Err(LLMurError::NotAuthorized)
        }
    }
}

pub(crate) async fn delete_key(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<VirtualKeyId>,
) -> Result<Json<StatusResponse>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let key = state.data.get_virtual_key(&id, &state.application_secret).await?.ok_or(LLMurError::AdminResourceNotFound)?; // TODO

    match user_context {
        UserContext::MasterUser => {
            let result = state.data.delete_virtual_key(&key.id).await?;
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
pub(crate) struct CreateVirtualKeyPayload {
    pub(crate) project_id: ProjectId,
    pub(crate) alias: Option<String>,
    pub(crate) description: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct GetVirtualKeyResult {
    pub(crate) id: VirtualKeyId,
    pub(crate) key: String,
    pub(crate) alias: String,
    pub(crate) blocked: bool,
    pub(crate) project_id: ProjectId
}

#[derive(Serialize)]
pub(crate) struct ListVirtualKeysResult {
    pub(crate) keys: Vec<GetVirtualKeyResult>,
    pub(crate) total: usize
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