use std::string::FromUtf8Error;
use std::sync::Arc;
use aes_gcm::aead;
use axum::extract::rejection::JsonRejection;
use axum::Json;
use axum::response::{IntoResponse, Response};
use hex::FromHexError;
use redis::RedisError;
use sqlx::migrate::MigrateError;
use tracing::info;
use uuid::Uuid;


#[derive(thiserror::Error, Debug)]
pub enum LLMurError {
    #[error(transparent)]
    AuthenticationError(#[from] AuthenticationError),
    #[error(transparent)]
    AuthorizationError(#[from] AuthorizationError),
    #[error(transparent)]
    DataAccessError(#[from] DataAccessError),
    #[error(transparent)]
    GraphError(#[from] GraphError),
    #[error(transparent)]
    ProxyError(#[from] ProxyError),
}

#[derive(thiserror::Error, Debug)]
pub enum ProxyError {
    #[error("Invalid request")]
    InvalidRequest(#[from] JsonRejection),
    #[error("Got error from provider: ({0})")]
    ProxyReturnError(reqwest::StatusCode, ProxyErrorMessage),
    #[error("Got error from provider: ({0}) {1}")]
    ProxyReqwestError(reqwest::StatusCode, reqwest::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Internal error: {0}. You should not be seeing this error. Please report a bug.")]
    InternalError(String),
}

#[derive(Debug)]
pub enum ProxyErrorMessage {
    Text(String),
    Json(serde_json::Value),
}


#[derive(thiserror::Error, Debug)]
pub enum GraphError {
    #[error(transparent)]
    GraphLoadError(#[from] GraphLoadError),
    #[error("No connection available for deployment")]
    NoConnectionAvailable(MissingConnectionReason),
    #[error(transparent)]
    UsageExceededError(#[from] UsageExceededError),
}

#[derive(Debug)]
pub enum MissingConnectionReason {
    NoUsageAvailable,
    DeploymentConnectionsNotSetup,
}

#[derive(thiserror::Error, Debug)]
pub enum GraphLoadError {
    #[error(transparent)]
    DataAccessError(#[from] DataAccessError),
    #[error("Invalid Virtual Key")]
    InvalidVirtualKey,
    #[error("Invalid Deployment Name")]
    InvalidDeploymentName,
    #[error(transparent)]
    InconsistentGraphDataError(#[from] InconsistentGraphDataError),
    #[error("Virtual key does not have access to deployment")]
    InvalidVirtualKeyDeployment,
}

#[derive(thiserror::Error, Debug)]
pub enum InconsistentGraphDataError {
    #[error("Invalid Project")]
    InvalidProject,
    #[error("Invalid Connection - Deployment association")]
    InvalidConnectionDeployments,
    #[error("Connection")]
    InvalidConnection,
}

#[derive(thiserror::Error, Debug)]
pub enum DataAccessError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    DbRecordConversionError(#[from] DbRecordConversionError),
    #[error(transparent)]
    EncryptionError(#[from] EncryptionError),
    #[error(transparent)]
    InvalidTimeFormatError(#[from] InvalidTimeFormatError),
    #[error(transparent)]
    HashError(#[from] HashError),
    #[error(transparent)]
    CacheAccessError(#[from] CacheAccessError),
    #[error("Resource not found")]
    ResourceNotFound,

    // TODO: These are used on the DB record creation methods. Should be replaced either by a single transaction or by returning the created record from the insert query instead of just the id
    #[error("Successfully created {1} resource but failed to retrieve it afterward. Resource id: {2}. Reason: {0}"
    )]
    FailedToGetCreatedResource(Box<DataAccessError>, String, Uuid),
    #[error("Successfully created {0} resource but failed to retrieve it afterward. Resource id: {1}. Reason: Resource not found"
    )]
    CreatedResourceNotFound(String, Uuid),
}

#[derive(thiserror::Error, Debug)]
pub enum SetupError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    RedisError(#[from] RedisError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    MigrationError(#[from] MigrateError),
    #[error("Database not configured")]
    MissingDatabase,
    #[error("Trying to set database twice")]
    DatabaseAlreadySet,
    #[error("Trying to set external cache twice")]
    ExternalCacheAlreadySet,
    #[error("Trying to set HTTP client twice")]
    HttpClientAlreadySet,
}

#[derive(thiserror::Error, Debug)]
pub enum DbRecordConversionError {
    #[error(transparent)]
    EncryptionError(#[from] EncryptionError),
    #[error(transparent)]
    DecryptionError(#[from] DecryptionError),
    #[error("Internal error: {0}. You should not be seeing this error. Please report a bug.")]
    InternalError(String),
    #[error("Internal error: {0}. You should not be seeing this error. Please report a bug.")]
    InvalidStatusCode(#[from] axum::http::status::InvalidStatusCode),
}

#[derive(thiserror::Error, Debug)]
pub enum CacheAccessError {
    #[error(transparent)]
    RedisError(#[from] RedisError)
}

#[derive(thiserror::Error, Debug)]
pub enum InvalidTimeFormatError {
    #[error("Invalid time format. Got '{0}'. '{1}' is not a valid value.")]
    TimeValueNotAValidNumber(String, String),
    #[error("Invalid time format. Got '{0}'. '{1}' is not a valid period.")]
    InvalidTimePeriod(String, String),
    #[error("Invalid time format. Got '{0}'.")]
    InvalidTimeFormat(String),
    #[error("Timestamp {0} out of range.")]
    TimestampOutOfRange(i64),
}

#[derive(thiserror::Error, Debug)]
pub enum EncryptionError {
    #[error(transparent)]
    AeadError(#[from] aead::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum DecryptionError {
    #[error(transparent)]
    AeadError(#[from] aead::Error),
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum AuthenticationError {
    #[error("Unauthorized.")]
    Unauthenticated,

    #[error("Invalid credentials.")]
    UserEmailNotFound,
    #[error("Invalid credentials.")]
    InvalidPassword,
    #[error("Invalid password scheme. Got: {0}")]
    AsyncError(#[from] AsyncError),
    #[error("Unable to parse the password scheme into its correct parts.")]
    PasswordSchemeParsingFailed,

    #[error("Auth Header was not provided.")]
    AuthHeaderNotProvided,
    #[error("Auth Bearer has invalid format.")]
    InvalidAuthBearer,
    #[error("Failed to retrieve session token")]
    UnableToFetchSessionToken,
    #[error("Session token not valid.")]
    InvalidSessionToken,

    #[error("Failed to retrieve user.")]
    UnableToFetchTokenUser,
    #[error("User not found.")]
    TokenUserNotFound,

    #[error(transparent)]
    HashError(#[from] HashError),

    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(thiserror::Error, Debug)]
pub enum AuthorizationError {
    #[error("Access denied.")]
    AccessDenied,
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum AsyncError {
    #[error("Failed to join threads. Reason: {0}")]
    JoinError(String),
}
#[derive(thiserror::Error, Debug, Clone)]
pub enum HashError {
    #[error(transparent)]
    AsyncError(#[from] AsyncError),
    #[error("Invalid password scheme. Got: {0}")]
    SchemeNotFound(String),
}

#[derive(thiserror::Error, Debug)]
pub enum UsageExceededError {
    #[error("Maximum budget limit exceeded. Used: {used}. Available: {limit} per month")]
    MonthBudgetOverLimit { used: f64, limit: f64 },
    #[error("Maximum budget limit exceeded. Used: {used}. Available: {limit} per hour")]
    HourBudgetOverLimit { used: f64, limit: f64 },
    #[error("Maximum budget limit exceeded. Used: {used}. Available: {limit} per day")]
    DayBudgetOverLimit { used: f64, limit: f64 },
    #[error("Maximum budget limit exceeded. Used: {used}. Available: {limit} per minute")]
    MinuteBudgetOverLimit { used: f64, limit: f64 },

    #[error("Maximum requests limit exceeded. Used: {used}. Available: {limit} per month")]
    MonthRequestsOverLimit { used: i64, limit: i64 },
    #[error("Maximum requests limit exceeded. Used: {used}. Available: {limit} per hour")]
    HourRequestsOverLimit { used: i64, limit: i64 },
    #[error("Maximum requests limit exceeded. Used: {used}. Available: {limit} per day")]
    DayRequestsOverLimit { used: i64, limit: i64 },
    #[error("Maximum requests limit exceeded. Used: {used}. Available: {limit} per minute")]
    MinuteRequestsOverLimit { used: i64, limit: i64 },

    #[error("Maximum tokens limit exceeded. Used: {used}. Available: {limit} per month")]
    MonthTokensOverLimit { used: i64, limit: i64 },
    #[error("Maximum tokens limit exceeded. Used: {used}. Available: {limit} per hour")]
    HourTokensOverLimit { used: i64, limit: i64 },
    #[error("Maximum tokens limit exceeded. Used: {used}. Available: {limit} per day")]
    DayTokensOverLimit { used: i64, limit: i64 },
    #[error("Maximum tokens limit exceeded. Used: {used}. Available: {limit} per minute")]
    MinuteTokensOverLimit { used: i64, limit: i64 },
}

impl IntoResponse for LLMurError {
    fn into_response(self) -> Response {
        info!("Responding with error: {:?}", self);
        match self {
            LLMurError::AuthenticationError(e) => match e {
                AuthenticationError::AsyncError(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "AuthenticationError").into_response(),
                AuthenticationError::PasswordSchemeParsingFailed => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "PasswordSchemeParsingFailed").into_response(),
                AuthenticationError::UnableToFetchSessionToken => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "UnableToFetchSessionToken").into_response(),
                AuthenticationError::UnableToFetchTokenUser => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "UnableToFetchTokenUser").into_response(),
                AuthenticationError::HashError(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "HashError").into_response(),
                AuthenticationError::InternalError(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
                _ => (axum::http::StatusCode::UNAUTHORIZED, "Not allowed").into_response()
            }
            LLMurError::AuthorizationError(e) => match e {
                AuthorizationError::AccessDenied => (axum::http::StatusCode::FORBIDDEN, "Access denied").into_response(),
            },
            LLMurError::DataAccessError(e) => e.into_response(),
            LLMurError::GraphError(e) => match e {
                GraphError::GraphLoadError(ge) => match ge {
                    GraphLoadError::DataAccessError(e) => e.into_response(),
                    GraphLoadError::InvalidVirtualKey => (axum::http::StatusCode::UNAUTHORIZED, "Not allowed").into_response(),
                    GraphLoadError::InvalidDeploymentName => (axum::http::StatusCode::NOT_FOUND, "Model not found").into_response(),
                    GraphLoadError::InconsistentGraphDataError(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "InconsistentGraphDataError").into_response(),
                    GraphLoadError::InvalidVirtualKeyDeployment => (axum::http::StatusCode::FORBIDDEN, "Not allowed").into_response(),
                }
                GraphError::NoConnectionAvailable(_) => (axum::http::StatusCode::SERVICE_UNAVAILABLE, "No connection available").into_response(),
                GraphError::UsageExceededError(_) => (axum::http::StatusCode::TOO_MANY_REQUESTS, "Too many requests").into_response(),
            },
            LLMurError::ProxyError(e) => e.into_response(),
        }
    }
}

impl From<Arc<AuthenticationError>> for LLMurError {
    fn from(err: Arc<AuthenticationError>) -> Self {
        LLMurError::AuthenticationError(Arc::unwrap_or_clone(err))
    }
}

impl IntoResponse for &ProxyError {
    fn into_response(self) -> Response {
        let resp = match self {
            ProxyError::InvalidRequest(e) => (axum::http::StatusCode::BAD_REQUEST, format!("Invalid payload. Reason: {}", e)).into_response(),
            ProxyError::ProxyReturnError(s, v) => match v {
                ProxyErrorMessage::Text(v) => (*s, v.to_string()).into_response(),
                ProxyErrorMessage::Json(v) => (*s, Json(v)).into_response()
            },
            ProxyError::ProxyReqwestError(s, e) => (*s, e.to_string()).into_response(),
            ProxyError::SerdeJsonError(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            ProxyError::ReqwestError(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            ProxyError::InternalError(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        };

        resp
    }
}

impl IntoResponse for DataAccessError {
    fn into_response(self) -> Response {
        match self {
            DataAccessError::SqlxError(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "DataAccessError").into_response(),
            DataAccessError::DbRecordConversionError(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "DataAccessError").into_response(),
            DataAccessError::EncryptionError(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "DataAccessError").into_response(),
            DataAccessError::InvalidTimeFormatError(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            DataAccessError::HashError(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "DataAccessError").into_response(),
            DataAccessError::CacheAccessError(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "DataAccessError").into_response(),
            DataAccessError::ResourceNotFound => (axum::http::StatusCode::NOT_FOUND, "Not Found").into_response(),
            DataAccessError::FailedToGetCreatedResource(_, _, _) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "FailedToGetCreatedResource").into_response(),
            DataAccessError::CreatedResourceNotFound(_, _) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "CreatedResourceNotFound").into_response(),
        }
    }
}
