use std::collections::BTreeMap;
use crate::data::session_token::SessionToken;
use crate::data::user::{ApplicationRole, User};
use crate::errors::{AuthenticationError, AuthorizationError, LLMurError};
use crate::LLMurState;
use axum::extract::{Request, State};
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::data::membership::{Membership, MembershipId};
use crate::data::project::{ProjectId, ProjectRole};

#[derive(Clone, Debug)]
pub enum UserContext {
    MasterUser,
    WebAppUser {
        session_token: SessionToken,
        user: User
    },
}

pub type UserContextExtractionResult = Result<UserContext, Arc<AuthenticationError>>;

impl UserContext {
    pub fn resolve_from_key_header(state: Arc<LLMurState>, header: &HeaderValue) -> UserContextExtractionResult {
        let key_str = header.to_str().map_err(|_| AuthenticationError::InvalidAuthBearer)?;

        if state.master_keys.contains(key_str) {
            Ok(UserContext::MasterUser)
        } else { Err(Arc::new(AuthenticationError::InvalidAuthBearer)) }
    }

    pub async fn resolve_from_session_header(state: Arc<LLMurState>, header: &HeaderValue) -> UserContextExtractionResult {
        let session_token_str = header.to_str().map_err(|_| AuthenticationError::InvalidAuthBearer)?;
        let session_token_id = SessionToken::generate_id(session_token_str, &state.application_secret).into();

        let token = state.data
            .get_session_token(&session_token_id, &state.metrics)
            .await
            .map_err(|_| AuthenticationError::UnableToFetchSessionToken)?
            .ok_or(AuthenticationError::InvalidSessionToken)?;

        // TODO: Check if session was revoked or expired

        let user = state.data
            .get_user(&token.user_id, &state.metrics)
            .await
            .map_err(|_| AuthenticationError::UnableToFetchTokenUser)?
            .ok_or(AuthenticationError::TokenUserNotFound)?;

        Ok(
            UserContext::WebAppUser {
                session_token: token,
                user
            }
        )
    }

    pub fn unauthenticated() -> UserContextExtractionResult {
        Err(Arc::new(AuthenticationError::Unauthenticated))
    }
}

impl UserContext {
    // region:    --- Authorization Helpers

    /// Check if user has admin access to a project
    pub(crate) async fn has_project_admin_access(
        &self,
        state: Arc<LLMurState>,
        project_id: &ProjectId,
    ) -> Result<bool, LLMurError> {
        match self {
            UserContext::MasterUser => Ok(true),
            UserContext::WebAppUser { user, .. } => {
                if user.role == ApplicationRole::Admin {
                    return Ok(true);
                }

                let memberships: BTreeMap<MembershipId, Membership> = state
                    .data
                    .get_memberships(&user.memberships, &state.metrics)
                    .await?
                    .into_iter()
                    .filter_map(|(k, v)| v.map(|val| (k, val)))
                    .collect();

                Ok(memberships
                    .values()
                    .find(|&v| v.project_id == *project_id)
                    .map(|membership| membership.role == ProjectRole::Admin)
                    .unwrap_or(false))
            }
        }
    }

    /// Check if user has developer or admin access to a project
    pub(crate) async fn has_project_developer_access(
        &self,
        state: Arc<LLMurState>,
        project_id: &ProjectId,
    ) -> Result<bool, LLMurError> {
        match self {
            UserContext::MasterUser => Ok(true),
            UserContext::WebAppUser { user, .. } => {
                if user.role == ApplicationRole::Admin {
                    return Ok(true);
                }

                let memberships: BTreeMap<MembershipId, Membership> = state
                    .data
                    .get_memberships(&user.memberships, &state.metrics)
                    .await?
                    .into_iter()
                    .filter_map(|(k, v)| v.map(|val| (k, val)))
                    .collect();

                Ok(memberships
                    .values()
                    .find(|&v| v.project_id == *project_id)
                    .map(|membership| {
                        membership.role == ProjectRole::Admin
                            || membership.role == ProjectRole::Developer
                    })
                    .unwrap_or(false))
            }
        }
    }
    // endregion: --- Authorization Helpers
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
                    UserContext::WebAppUser { .. } => { Err(AuthorizationError::AccessDenied)? }
                }
            }
            Err(_) => { Err(AuthorizationError::AccessDenied)? }
        }
    }

    fn require_authenticated_user(self) -> Result<UserContext, LLMurError> {
        match self {
            Ok(c) => {Ok(c)}
            Err(e/*: Arc<AuthenticationError>*/) => { todo!("How can I do this") /*impl From<AuthenticationError> for LLMurError*/}
        }
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

