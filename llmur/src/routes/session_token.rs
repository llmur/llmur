use crate::data::password::{SchemeStatus, validate_password};
use crate::data::session_token::{SessionToken, SessionTokenId};
use crate::data::user::UserId;
use crate::errors::{AuthenticationError, LLMurError};
use crate::{LLMurState, impl_from_vec_result};
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// region:    --- Routes

pub(crate) fn routes(state: Arc<LLMurState>) -> Router<Arc<LLMurState>> {
    Router::new()
        // Project routes
        .route("/", post(create_session_token))
        .with_state(state.clone())
}

#[tracing::instrument(name = "handler.create.session_token", skip(state, payload))]
pub(crate) async fn create_session_token(
    State(state): State<Arc<LLMurState>>,
    Json(payload): Json<CreateSessionTokenPayload>,
) -> Result<Json<GetSessionTokenResult>, LLMurError> {
    let CreateSessionTokenPayload {
        email,
        password: password_clear,
    } = payload;

    let user = state
        .data
        .get_user_with_email(&email, &state.metrics)
        .await?
        .ok_or(AuthenticationError::UserEmailNotFound)?;

    let status = validate_password(
        &password_clear,
        &user.hashed_password,
        &user.salt,
        &state.application_secret,
    )
    .await?;

    if let SchemeStatus::Outdated = status {
        // TODO: update password
    }

    let token_str = SessionToken::generate_random_token();
    let token_id = SessionToken::generate_id(&token_str, &state.application_secret).into();

    let token = state
        .data
        .create_session_token(&token_id, &user.id, &state.metrics)
        .await?;

    Ok(Json(GetSessionTokenResult {
        token: token_str,
        info: SessionTokenInfoData {
            id: token.id,
            revoked: token.revoked,
            user_id: user.id,
        },
    }))
}
// endregion: --- Routes

// region:    --- Data Models

#[derive(Deserialize)]
pub(crate) struct CreateSessionTokenPayload {
    pub(crate) email: String,
    pub(crate) password: String,
}

#[derive(Serialize)]
pub(crate) struct GetSessionTokenResult {
    pub(crate) token: String,
    pub(crate) info: SessionTokenInfoData,
}

#[derive(Serialize)]
pub(crate) struct SessionTokenInfoData {
    pub(crate) id: SessionTokenId,
    pub(crate) revoked: bool,
    pub(crate) user_id: UserId,
}

#[derive(Serialize)]
pub(crate) struct ListSessionTokensResult {
    pub(crate) tokens: Vec<GetSessionTokenResult>,
    pub(crate) total: usize,
}

impl_from_vec_result!(GetSessionTokenResult, ListSessionTokensResult, tokens);
// endregion: --- Data Models
