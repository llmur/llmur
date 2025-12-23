use crate::data::connection::{AzureOpenAiApiVersion, Connection, ConnectionId, ConnectionInfo};
use crate::data::limits::{BudgetLimits, RequestLimits, TokenLimits};
use crate::errors::{AuthorizationError, DataAccessError, LLMurError};
use crate::routes::middleware::user_context::{
    AuthorizationManager, UserContextExtractionResult,
};
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
        .route("/", post(create_connection))
        .route("/{id}", get(get_connection))
        .route("/{id}", delete(delete_connection))
        .with_state(state.clone())
}

#[tracing::instrument(name = "handler.create.connection", skip(state, ctx, payload))]
pub(crate) async fn create_connection(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Json(payload): Json<CreateConnectionPayload>,
) -> Result<Json<GetConnectionResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    if !user_context.has_admin_access() {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let result = match &payload {
        CreateConnectionPayload::AzureOpenAi {
            deployment_name,
            api_endpoint,
            api_key,
            api_version,
            budget_limits,
            request_limits,
            token_limits,
        } => {
            state
                .data
                .create_azure_openai_connection(
                    deployment_name,
                    api_endpoint,
                    api_key,
                    api_version,
                    budget_limits,
                    request_limits,
                    token_limits,
                    &state.application_secret,
                    &state.metrics,
                )
                .await?
        }
        CreateConnectionPayload::OpenAi {
            model,
            api_endpoint,
            api_key,
            budget_limits,
            request_limits,
            token_limits,
        } => {
            state
                .data
                .create_openai_v1_connection(
                    model,
                    api_endpoint,
                    api_key,
                    budget_limits,
                    request_limits,
                    token_limits,
                    &state.application_secret,
                    &state.metrics,
                )
                .await?
        }
    };

    Ok(Json(result.into()))
}

#[tracing::instrument(
    name = "handler.get.connection",
    skip(state, ctx, id),
    fields(
        id = %id.0,
    )
)]
pub(crate) async fn get_connection(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<ConnectionId>,
) -> Result<Json<GetConnectionResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    if !user_context.has_admin_access() {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let connection = state
        .data
        .get_connection(&id, &state.application_secret, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    Ok(Json(connection.into()))
}

#[tracing::instrument(
    name = "handler.delete.connection",
    skip(state, ctx, id),
    fields(
        id = %id.0,
    )
)]
pub(crate) async fn delete_connection(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<ConnectionId>,
) -> Result<Json<StatusResponse>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    if !user_context.has_admin_access() {
        return Err(AuthorizationError::AccessDenied)?;
    }

    let connection = state
        .data
        .get_connection(&id, &state.application_secret, &state.metrics)
        .await?
        .ok_or(DataAccessError::ResourceNotFound)?;

    let result = state
        .data
        .delete_connection(&connection.id, &state.metrics)
        .await?;

    Ok(Json(StatusResponse {
        success: result != 0,
        message: None,
    }))
}

// endregion: --- Routes

// region:    --- Data Models
#[derive(Deserialize)]
#[serde(tag = "provider")]
pub(crate) enum CreateConnectionPayload {
    #[serde(rename = "azure/openai", alias = "azure/openai")]
    AzureOpenAi {
        deployment_name: String,
        api_endpoint: String,
        api_key: String,
        api_version: AzureOpenAiApiVersion,

        budget_limits: Option<BudgetLimits>,
        request_limits: Option<RequestLimits>,
        token_limits: Option<TokenLimits>,
    },
    #[serde(rename = "openai/v1", alias = "openai/v1")]
    OpenAi {
        model: String,
        api_endpoint: String,
        api_key: String,

        budget_limits: Option<BudgetLimits>,
        request_limits: Option<RequestLimits>,
        token_limits: Option<TokenLimits>,
    },
}

#[derive(Clone, Serialize)]
#[serde(tag = "provider")]
pub enum GetConnectionResult {
    #[serde(rename = "azure/openai", alias = "azure/openai")]
    AzureOpenAi {
        id: ConnectionId,
        api_key: String,
        deployment_name: String,
        api_endpoint: String,
        api_version: AzureOpenAiApiVersion,
    },
    #[serde(rename = "openai/v1", alias = "openai/v1")]
    OpenAiV1 {
        id: ConnectionId,
        api_key: String,
        model: String,
        api_endpoint: String,
    },
}

#[derive(Serialize)]
pub(crate) struct ListConnectionsResult {
    pub(crate) connections: Vec<GetConnectionResult>,
    pub(crate) total: usize,
}

impl_from_vec_result!(GetConnectionResult, ListConnectionsResult, connections);

impl From<Connection> for GetConnectionResult {
    fn from(connection: Connection) -> Self {
        match connection.connection_info {
            ConnectionInfo::AzureOpenAiApiKey {
                api_key,
                api_endpoint,
                api_version,
                deployment_name,
            } => GetConnectionResult::AzureOpenAi {
                id: connection.id,
                api_key,
                deployment_name,
                api_endpoint,
                api_version,
            },
            ConnectionInfo::OpenAiApiKey {
                api_key,
                api_endpoint,
                model,
            } => GetConnectionResult::OpenAiV1 {
                id: connection.id,
                api_key,
                model,
                api_endpoint,
            },
        }
    }
}
// endregion: --- Data Models
