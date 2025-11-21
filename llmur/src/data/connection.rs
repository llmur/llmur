use crate::data::errors::DataConversionError;
use crate::data::limits::{BudgetLimits, RequestLimits, TokenLimits};
use crate::data::utils::{decrypt, encrypt, ConvertInto};
use crate::data::DataAccess;
use crate::errors::DataAccessError;
use crate::{default_access_fns, default_database_access_fns, impl_structured_id_utils, impl_with_id_parameter_for_struct};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, Postgres, QueryBuilder};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

// region:    --- Main Model
#[derive(Debug, Clone, sqlx::Type, PartialEq, Serialize, Deserialize)]
#[sqlx(type_name = "azure_openai_api_version", rename_all = "lowercase")]
pub enum AzureOpenAiApiVersion {
    #[serde(rename = "2024-02-01")]
    #[sqlx(rename = "2024-02-01")]
    V2024_02_01,
    #[sqlx(rename = "2024-06-01")]
    #[serde(rename = "2024-06-01")]
    V2024_06_01,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    sqlx::Type,
    Serialize,
    Deserialize,
    FromRow
)]
#[sqlx(transparent)]
pub struct ConnectionId(pub Uuid);

#[derive(Clone, Serialize, Debug)]
pub enum ConnectionInfo {
    AzureOpenAiApiKey {
        api_key: String,
        api_endpoint: String,
        api_version: AzureOpenAiApiVersion,
        deployment_name: String
    },
    OpenAiApiKey {
        api_key: String,
        api_endpoint: String,
        model: String
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct Connection {
    pub id: ConnectionId,
    pub connection_info: ConnectionInfo,
    pub budget_limits: Option<BudgetLimits>,
    pub request_limits: Option<RequestLimits>,
    pub token_limits: Option<TokenLimits>,
}

impl Connection {
    pub(crate) fn new(id: ConnectionId, connection_info: ConnectionInfo, budget_limits: Option<BudgetLimits>, request_limits: Option<RequestLimits>, token_limits: Option<TokenLimits>) -> Self {
        Connection {
            id,
            connection_info,
            budget_limits,
            request_limits,
            token_limits
        }
    }
}

impl ConnectionInfo {
    fn concat(&self) -> String {
        match self {
            ConnectionInfo::AzureOpenAiApiKey {
                api_key, api_endpoint, api_version, deployment_name
            } => {
                format!("AzureOpenAiApiKey:{:?}:{:?}:{:?}:{:?}", api_key, api_endpoint, api_version, deployment_name)
            }
            ConnectionInfo::OpenAiApiKey {
                api_key, api_endpoint, model
            } => {
                format!("OpenAiApiKey:{:?}:{:?}:{:?}", api_key, api_endpoint, model)
            }
        }
    }
}

impl std::fmt::Display for AzureOpenAiApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureOpenAiApiVersion::V2024_02_01 => write!(f, "2024-02-01"),
            AzureOpenAiApiVersion::V2024_06_01 => write!(f, "2024-06-01"),
        }
    }
}

impl_structured_id_utils!(ConnectionId);
impl_with_id_parameter_for_struct!(Connection, ConnectionId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {
    pub async fn get_connection(&self, id: &ConnectionId, application_secret: &Uuid) -> Result<Option<Connection>, DataAccessError> {
        self.__get_connection(id, &Some(application_secret.clone())).await
    }

    pub async fn get_connections(&self, ids: &BTreeSet<ConnectionId>, application_secret: &Uuid) -> Result<BTreeMap<ConnectionId, Option<Connection>>, DataAccessError> {
        self.__get_connections(ids, &Some(application_secret.clone())).await
    }

    pub async fn create_azure_openai_connection(&self, deployment_name: &str, api_endpoint: &str, api_key: &str, api_version: &AzureOpenAiApiVersion, application_secret: &Uuid) -> Result<Connection, DataAccessError> {
        let salt = Uuid::now_v7();
        let encrypted_api_key = encrypt(api_key, &salt, application_secret).map_err(|_| DataAccessError::FailedToCreateKey)?;

        let connection_info = DbConnectionInfoColumn::AzureOpenAiApiKey {
            encrypted_api_key,
            api_endpoint: api_endpoint.to_string(),
            api_version: api_version.clone(),
            deployment_name: deployment_name.to_string(),
            salt,
        };

        self.__create_connection(
            &connection_info,
            &Some(application_secret.clone())
        ).await
    }

    pub async fn create_openai_v1_connection(&self, model: &str, api_endpoint: &str, api_key: &str, application_secret: &Uuid) -> Result<Connection, DataAccessError> {
        let salt = Uuid::now_v7();
        let encrypted_api_key = encrypt(api_key, &salt, application_secret).map_err(|_| DataAccessError::FailedToCreateKey)?;

        let connection_info = DbConnectionInfoColumn::OpenAiApiKey {
            encrypted_api_key,
            api_endpoint: api_endpoint.to_string(),
            model: model.to_string(),
            salt,
        };

        self.__create_connection(
            &connection_info,
            &Some(application_secret.clone())
        ).await
    }

    pub async fn delete_connection(&self, id: &ConnectionId) -> Result<u64, DataAccessError> {
        self.__delete_connection(id).await
    }
}

default_access_fns!(
        Connection,
        ConnectionId,
        connection,
        connections,
        create => {
            connection_info: &DbConnectionInfoColumn
        },
        search => {}
    );
// endregion: --- Data Access

// region:    --- Database Access
default_database_access_fns!(
    DbConnectionRecord,
    ConnectionId,
    connection,
    connections,
    insert => {
        connection_info: &DbConnectionInfoColumn
    },
    search => { }
);
// region:      --- Postgres Queries
pub(crate) fn pg_search() -> QueryBuilder<'static, Postgres> {
    todo!()
}
pub(crate) fn pg_insert(connection_info: &DbConnectionInfoColumn) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("INSERT INTO connections
        (
            id,
            connection_info
        )
        VALUES (gen_random_uuid(), ");

    // Push connection_info
    query.push_bind(Json::from(connection_info));
    query.push(") RETURNING id");
    // Return builder
    query
}
pub(crate) fn pg_get(id: &ConnectionId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            id, connection_info, budget_limits, request_limits, token_limits
            FROM connections
        WHERE id="
    );
    // Push id
    query.push_bind(id);

    // Return builder
    query
}
pub(crate) fn pg_getm(ids: &Vec<ConnectionId>) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            id, connection_info, budget_limits, request_limits, token_limits
            FROM connections
        WHERE id IN ( "
    );
    // Push ids
    let mut separated = query.separated(", ");
    for id in ids.iter() {
        separated.push_bind(id);
    }
    separated.push_unseparated(") ");

    query
}
pub(crate) fn pg_delete(id: &ConnectionId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        DELETE FROM connections
        WHERE id="
    );
    // Push id
    query.push_bind(id);
    // Return query
    query
}
// endregion:   --- Postgres Queries
// endregion: --- Database Access

// region:    --- Database Model
#[derive(FromRow, Clone, Debug)]
pub(crate) struct DbConnectionRecord {
    pub(crate) id: ConnectionId,
    pub(crate) connection_info: sqlx::types::Json<DbConnectionInfoColumn>,
    pub(crate) budget_limits: Option<sqlx::types::Json<BudgetLimits>>,
    pub(crate) request_limits: Option<sqlx::types::Json<RequestLimits>>,
    pub(crate) token_limits: Option<sqlx::types::Json<TokenLimits>>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "provider")]
pub(crate) enum DbConnectionInfoColumn {
    #[serde(rename = "azure/openai", alias = "azure/openai")]
    AzureOpenAiApiKey {
        encrypted_api_key: String,
        api_endpoint: String,
        api_version: AzureOpenAiApiVersion,
        deployment_name: String,
        salt: Uuid,
    },
    #[serde(rename = "openai/v1", alias = "openai/v1")]
    OpenAiApiKey {
        encrypted_api_key: String,
        api_endpoint: String,
        model: String,
        salt: Uuid,
    },
}

impl ConvertInto<ConnectionInfo> for DbConnectionInfoColumn {
    fn convert(self, application_secret: &Option<Uuid>) -> Result<ConnectionInfo, DataConversionError> {
        let application_secret = application_secret.ok_or(DataConversionError::DefaultError {cause: "Ups".to_string()})?; // TODO return Internal Server Error
        match self {
            DbConnectionInfoColumn::AzureOpenAiApiKey {
                encrypted_api_key, api_endpoint, api_version, deployment_name, salt
            } => {
                Ok(ConnectionInfo::AzureOpenAiApiKey {
                    api_key: decrypt(&encrypted_api_key, &salt, &application_secret).map_err(|_| DataConversionError::DefaultError {cause: "Ups".to_string()})?,
                    api_endpoint,
                    api_version,
                    deployment_name,
                })
            }
            DbConnectionInfoColumn::OpenAiApiKey {
                encrypted_api_key, api_endpoint, model, salt
            } => {
                Ok(ConnectionInfo::OpenAiApiKey {
                    api_key: decrypt(&encrypted_api_key, &salt, &application_secret).map_err(|_| DataConversionError::DefaultError {cause: "Ups".to_string()})?,
                    api_endpoint,
                    model,
                })
            }
        }
    }
}

impl ConvertInto<Connection> for DbConnectionRecord {
    fn convert(self, application_secret: &Option<Uuid>) -> Result<Connection, DataConversionError> {
        let connection_info = self.connection_info.0.convert(application_secret).map_err(|_| DataConversionError::DefaultError {cause: "Ups".to_string()})?;
        Ok(
            Connection::new(self.id, connection_info, None, None, None)
        )
    }
}

impl_with_id_parameter_for_struct!(DbConnectionRecord, ConnectionId);
// endregion: --- Database Model
