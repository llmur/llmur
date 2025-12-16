use std::string::FromUtf8Error;
use aes_gcm::aead;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use hex::FromHexError;
use redis::RedisError;
use serde_json::json;
use sqlx::migrate::MigrateError;
use tokio::task::JoinError;
use uuid::Uuid;
use crate::data::connection::ConnectionId;
use crate::data::connection_deployment::ConnectionDeploymentId;
use crate::data::deployment::DeploymentId;
use crate::data::password::SchemeDispatcher;
use crate::data::project::ProjectId;
use crate::data::virtual_key::VirtualKeyId;
use crate::data::virtual_key_deployment::VirtualKeyDeploymentId;



#[derive(thiserror::Error, Debug)]
pub enum GraphError {
    #[error(transparent)]
    GraphLoadError(#[from] GraphLoadError),
    #[error(transparent)]
    UsageExceededError(#[from] UsageExceededError),
    #[error("No connection available for deployment")]
    NoConnectionAvailable(MissingConnectionReason),
}

#[derive(Debug)]
pub enum MissingConnectionReason {
    NoUsageAvailable,
    DeploymentConnectionsNotSetup
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

    
    // TODO: These are used on the DB record creation methods. Should be replaced either by a single transaction or by returning the created record from the insert query instead of just the id
    #[error("Successfully created {1} resource but failed to retrieve it afterward. Resource id: {2}. Reason: {0}")]
    FailedToGetCreatedResource(Box<DataAccessError>, String, Uuid),
    #[error("Successfully created {0} resource but failed to retrieve it afterward. Resource id: {1}. Reason: Resource not found")]
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

#[derive(thiserror::Error, Debug)]
pub enum AuthenticationError {
    #[error("Invalid credentials.")]
    InvalidPassword,
    #[error("Invalid password scheme. Got: {0}")]
    AsyncError(#[from] AsyncError),
    #[error("Unable to parse the password scheme into its correct parts.")]
    PasswordSchemeParsingFailed,

    #[error(transparent)]
    HashError(#[from] HashError),
}

#[derive(thiserror::Error, Debug)]
pub enum AsyncError {
    #[error("Failed to join threads")]
    JoinError(#[from] JoinError),
}
#[derive(thiserror::Error, Debug)]
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



/*
#[derive(thiserror::Error, Debug, Clone)]
pub enum GraphConstructionError {
    /// A deployment referenced by a VirtualKeyDeployment was not found in the provided deployments
    #[error("Virtual Key not found while building graph: {0}")]
    VirtualKeyNotFound(VirtualKeyId),
    /// A deployment referenced by a VirtualKeyDeployment was not found in the provided deployments
    #[error("Deployment not found while building graph: {0}")]
    DeploymentNotFound(DeploymentId),
    /// A connection referenced by a ConnectionDeployment was not found in the provided connections
    #[error("Connection not found while building graph: {0}")]
    ConnectionNotFound(ConnectionId),

    #[error("Invalid association between Virtual Key and Deployment")]
    VirtualKeyDeploymentMismatch
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum DataAccessError {
    #[error(transparent)]
    DataConversionError(#[from] DataConversionError),
    #[error(transparent)]
    DatabaseError(#[from] DatabaseError),
    #[error(transparent)]
    CacheError(#[from] CacheError),
    #[error("CDatabaseInconsistencyError")] // TODO
    DatabaseInconsistencyError,
    #[error("InvalidEmail")] // TODO
    InvalidEmail,
    #[error("InvalidInviteCode")] // TODO
    InvalidInviteCode,
    #[error("FailedToCreateKey")] // TODO
    FailedToCreateKey,
    #[error("FailedToHashPassword")] // TODO
    FailedToHashPassword,
    #[error("NoConnectionsAvailable")] // TODO
    NoConnectionsAvailable,


    #[error("Virtual key not found in cache: {0}")]
    InMemoryVirtualKeyNotFound(VirtualKeyId),

    #[error("Project not found in cache: {0}")]
    InMemoryProjectNotFound(ProjectId),

    #[error("Virtual key deployment mapping not found: {0}")]
    InMemoryVirtualKeyDeploymentNotFound(VirtualKeyDeploymentId),

    #[error("Deployment not found: {0}")]
    InMemoryDeploymentNotFound(DeploymentId),

    #[error("Connection deployment mapping not found: {0}")]
    InMemoryConnectionDeploymentNotFound(ConnectionDeploymentId),

    #[error("Connection not found: {0}")]
    InMemoryConnectionNotFound(ConnectionId),

    #[error("Graph construction failed: {0}")]
    GraphConstructionFailed(#[from] GraphConstructionError),

    #[error("Graph Load failed: {0}")]
    GraphLoadError(#[from] GraphLoadError),

    #[error("Couldn't find deployment with name: {0}")]
    InMemoryDeploymentNotFoundByName(String),

    #[error("No association between Virtual Key and Deployment not found for deployment: {0}")]
    InMemoryVirtualKeyDeploymentNotFoundForDeployment(DeploymentId),

    #[error("Exceeded Budget: {0}")]
    BudgetExceeded(String),
    #[error("Exceeded RPM: {0}")]
    RequestUsageExceeded(String),
    #[error("Exceeded TPM: {0}")]
    TokenUsageExceeded(String),
}


#[derive(thiserror::Error, Debug)]
pub enum BuilderError {
    #[error("missing database configuration")]
    MissingDatabase,
    #[error("missing cache configuration")]
    MissingCache,
    #[error("database already set")]
    DatabaseAlreadySet,
    #[error("cache already set")]
    CacheAlreadySet,
    #[error("HTTP client already set")]
    HttpClientAlreadySet,
    #[error("inner db/cache error: {0}")]
    DatabaseBuilderError(#[from] DatabaseError),
    #[error("inner db/cache error: {0}")]
    CacheBuilderError(#[from] CacheError),
    #[error("http client build error: {0}")]
    Http(#[from] reqwest::Error),
}






#[derive(thiserror::Error, Debug)]
pub enum LLMurError {
    #[error("Upstream Unavailable")]
    UpstreamUnavailable,
    #[error("Missing Api Key error")]
    ProxyApiKeyNotFound,
    #[error("Missing Authorization error")]
    ProxyMissingAuthorizationHeader,
    #[error("Malformed Authorization error")]
    ProxyMalformedAuthorizationHeader,
    #[error(transparent)]
    ProxyRequestError(#[from] ProxyRequestError),
    #[error(transparent)]
    GraphLoadError(#[from] GraphLoadError),

    #[error(transparent)]
    AdminDataAccessError(#[from] DataAccessError),

    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    UserContextExtractionError(#[from] UserContextExtractionError),
    #[error(transparent)]
    AuthorizationHeaderExtractionError(#[from] AuthorizationHeaderExtractionError),

    #[error("Resource not found")]
    AdminResourceNotFound,
    #[error("Resource not found")]
    UserNotFound,
    #[error("Resource not found")]
    PasswordDoesNotMatch,
    #[error("Resource not found")]
    FailedToHashPassword,
    #[error("Resource not found")]
    ApiKeyNotFound,
    #[error("Resource not found")]
    InvalidApiKey,
    #[error("Not authorized")]
    NotAuthorized,

    #[error("Internal server error: {0}")]
    InternalServerError(String),
    #[error("Invalid Payload: {0}")]
    BadRequest(String),
}



#[derive(thiserror::Error, Debug, Clone)]
pub enum AuthorizationHeaderExtractionError {
    #[error("Error")]
    InvalidAuthorizationHeader ,
    #[error("Error")]
    AuthorizationHeaderNotProvided
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum GraphLoadError {
    //#[error("Error")]
    //AuthorizationHeaderExtractionError(#[from] AuthorizationHeaderExtractionError),
    #[error("Error")]
    InvalidVirtualKey, // Virtual key does not exist
    #[error("Error")]
    InvalidDeploymentName, // Deployment name does not exist
    #[error("Error")]
    InvalidVirtualKeyDeployment, // Deployment is not associated with the virtual key

    #[error("Error")]
    InternalServerError,
    #[error(transparent)]
    InconsistentGraphDataError(#[from] InconsistentGraphDataError), // Graph data is inconsistent - This can happen if a record is updated/deleted during the load process. Retrying the operation may resolve the issue.
}


#[derive(thiserror::Error, Debug, Clone)]
pub enum InconsistentGraphDataError {
    #[error("Error")]
    InvalidProject,
    #[error("Error")]
    InvalidConnectionDeployments,
    #[error("Error")]
    InvalidConnection,
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum UserContextExtractionError {
    #[error("Error")]
    AuthSetCookieError,
    #[error("Error")]
    AuthDeleteCookieError,
    #[error("Error")]
    AuthTokenNotInCookie,
    #[error("Error")]
    AuthTokenWrongFormat,
    #[error("Error")]
    AuthDataAccessError,
    #[error("Error")]
    AuthUserNotFound,
    #[error("Error")]
    AuthTokenValidationFailed,
    #[error("Error")]
    AuthCannotSetTokenCookie,
    #[error("Error")]
    AuthInvalidAuthBearer,
    #[error("Error")]
    FailedToGenerateToken,
    #[error("Error")]
    UnableToFetchSessionToken,
    #[error("Error")]
    SessionTokenNotFound,
    #[error("Error")]
    AuthenticationNotProvided,
}


#[derive(thiserror::Error, Debug, Clone)]
pub enum ProxyRequestError {
    #[error("Request successful but returned {0}")]
    ProxyReturnError(u16, serde_json::Value),
    #[error("Error: {0}")]
    ReqwestSerdeError(String),
    #[error("Error: {0}")]
    ReqwestError(String),
    #[error("Error: {0}")]
    SerdeError(String),
    #[error("Error: {0}")]
    DataAccessError(#[from] DataAccessError),
}

*/
// TODO: Improve error handling
impl IntoResponse for ProxyRequestError {
    fn into_response(self) -> Response {
        let status = match &self {
            ProxyRequestError::ProxyReturnError(code, _value) => StatusCode::from_u16(*code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            _ => StatusCode::INTERNAL_SERVER_ERROR,

        };

        let body = axum::Json(serde_json::json!({
            "error": format!("{:?}", self)
        }));

        let mut resp = (status, body).into_response();

        // Insert error into extensions for middleware access
        resp.extensions_mut().insert(self.clone());

        resp
    }
}

impl IntoResponse for LLMurError {
    fn into_response(self) -> Response {
        match self {
            LLMurError::NotAuthorized => {
                (StatusCode::UNAUTHORIZED, Json(json!({"error": self.to_string()}))).into_response()
            }
            _=> (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": self.to_string()}))).into_response()
        }

    }
    // Conversion logic to a suitable response type, e.g., JSON error message or HTTP status
}