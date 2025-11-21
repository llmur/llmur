use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use crate::data::connection::ConnectionId;
use crate::data::connection_deployment::ConnectionDeploymentId;
use crate::data::deployment::DeploymentId;
use crate::data::errors::{CacheError, DataConversionError, DatabaseError};
use crate::data::project::ProjectId;
use crate::data::virtual_key::VirtualKeyId;
use crate::data::virtual_key_deployment::VirtualKeyDeploymentId;

#[derive(thiserror::Error, Debug)]
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

#[derive(thiserror::Error, Debug)]
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


#[derive(thiserror::Error, Debug)]
pub enum ProxyRequestError {
    #[error("Request successful but returned {0}")]
    ProxyReturnError(u16, serde_json::Value),
    #[error(transparent)]
    ReqwestSerdeError(reqwest::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    DataAccessError(#[from] DataAccessError),
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