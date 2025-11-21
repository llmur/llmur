use crate::data::session_token::SessionToken;
use crate::data::user::User;
use crate::errors::{LLMurError, UserContextExtractionError};
use crate::LLMurState;
use axum::extract::{Request, State};
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum UserContext {
    MasterUser,
    WebAppUser {
        session_token: SessionToken,
        user: User
    },
}

pub type UserContextExtractionResult = Result<UserContext, UserContextExtractionError>;

impl UserContext {
    pub fn resolve_from_key_header(state: Arc<LLMurState>, header: &HeaderValue) -> UserContextExtractionResult {
        let key_str = header.to_str().map_err(|_| UserContextExtractionError::AuthInvalidAuthBearer)?;

        if state.master_keys.contains(key_str) {
            Ok(UserContext::MasterUser)
        } else { Err(UserContextExtractionError::AuthInvalidAuthBearer) }
    }

    pub async fn resolve_from_session_header(state: Arc<LLMurState>, header: &HeaderValue) -> UserContextExtractionResult {
        let session_token_str = header.to_str().map_err(|_| UserContextExtractionError::AuthInvalidAuthBearer)?;
        let session_token_id = SessionToken::generate_id(session_token_str, &state.application_secret).into();

        let token = state.data
            .get_session_token(&session_token_id)
            .await
            .map_err(|_| UserContextExtractionError::UnableToFetchSessionToken)?
            .ok_or(UserContextExtractionError::SessionTokenNotFound)?;

        // TODO: Check if session was revoked or expired

        let user = state.data
            .get_user(&token.user_id)
            .await
            .map_err(|_| UserContextExtractionError::AuthUserNotFound)?
            .ok_or(UserContextExtractionError::AuthUserNotFound)?;

        Ok(
            UserContext::WebAppUser {
                session_token: token,
                user
            }
        )
    }

    pub fn unauthenticated() -> UserContextExtractionResult {
        Err(UserContextExtractionError::AuthenticationNotProvided)
    }
}

pub(crate) trait AuthorizationManager {
    fn require_master_user(self) -> Result<UserContext, LLMurError>;
    fn require_authenticated_user(self) -> Result<UserContext, LLMurError>;
}

impl AuthorizationManager for UserContextExtractionResult {
    fn require_master_user(self) -> Result<UserContext, LLMurError> {
        match self {
            Ok(ctx) => {
                match ctx {
                    UserContext::MasterUser => { Ok(ctx) }
                    UserContext::WebAppUser { .. } => { Err(LLMurError::NotAuthorized) }
                }
            }
            Err(_) => { Err(LLMurError::NotAuthorized) }
        }
    }

    fn require_authenticated_user(self) -> Result<UserContext, LLMurError> {
        self.map_err(|_| LLMurError::NotAuthorized)
    }
}

pub(crate) fn user_context_load_mw(
    State(state): State<Arc<LLMurState>>,
    mut request: Request,
    next: Next,
) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> {
    Box::pin(async move {
        let headers = request.headers();

        // Handle the resolution logic with proper async/await flow
        let res = if let Some(header) = headers.get("X-LLMur-Key") {
            // API key takes precedence - this is sync, so we can call it directly
            UserContext::resolve_from_key_header(state.clone(), header)
        } else if let Some(header) = headers.get("X-LLMur-Session") {
            // Session token resolution - this is async
            UserContext::resolve_from_session_header(state.clone(), header).await
        } else {
            // No authentication provided
            UserContext::unauthenticated()
        };

        request.extensions_mut().insert(res);
        next.run(request).await
    })
}