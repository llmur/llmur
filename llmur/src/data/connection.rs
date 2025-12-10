use crate::data::DataAccess;
use crate::data::errors::DataConversionError;
use crate::data::limits::{BudgetLimits, RequestLimits, TokenLimits};
use crate::data::utils::{ConvertInto, decrypt, encrypt};
use crate::errors::DataAccessError;
use crate::{
    default_access_fns, default_database_access_fns, impl_structured_id_utils,
    impl_with_id_parameter_for_struct,
};
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
    FromRow,
)]
#[sqlx(transparent)]
pub struct ConnectionId(pub Uuid);

#[derive(Clone, Serialize, Debug)]
pub enum ConnectionInfo {
    AzureOpenAiApiKey {
        api_key: String,
        api_endpoint: String,
        api_version: AzureOpenAiApiVersion,
        deployment_name: String,
    },
    OpenAiApiKey {
        api_key: String,
        api_endpoint: String,
        model: String,
    },
}

impl ConnectionInfo {
    pub(crate) fn get_provider_friendly_name(&self) -> &'static str {
        match self {
            ConnectionInfo::AzureOpenAiApiKey { .. } => "azure/openai",
            ConnectionInfo::OpenAiApiKey { .. } => "openai/v1",
        }
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct Connection {
    pub id: ConnectionId,
    pub connection_info: ConnectionInfo,
    pub budget_limits: BudgetLimits,
    pub request_limits: RequestLimits,
    pub token_limits: TokenLimits,
}

impl Connection {
    pub(crate) fn new(
        id: ConnectionId,
        connection_info: ConnectionInfo,
        budget_limits: BudgetLimits,
        request_limits: RequestLimits,
        token_limits: TokenLimits,
    ) -> Self {
        Connection {
            id,
            connection_info,
            budget_limits,
            request_limits,
            token_limits,
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
    pub async fn get_connection(
        &self,
        id: &ConnectionId,
        application_secret: &Uuid,
    ) -> Result<Option<Connection>, DataAccessError> {
        self.__get_connection(id, &Some(application_secret.clone()))
            .await
    }

    pub async fn get_connections(
        &self,
        ids: &BTreeSet<ConnectionId>,
        application_secret: &Uuid,
    ) -> Result<BTreeMap<ConnectionId, Option<Connection>>, DataAccessError> {
        self.__get_connections(ids, &Some(application_secret.clone()))
            .await
    }

    pub async fn create_azure_openai_connection(
        &self,deployment_name: &str,
        api_endpoint: &str,
        api_key: &str,
        api_version: &AzureOpenAiApiVersion,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>,
        application_secret: &Uuid,
    ) -> Result<Connection, DataAccessError> {
        let salt = Uuid::now_v7();
        let encrypted_api_key = encrypt(api_key, &salt, application_secret)
            .map_err(|_| DataAccessError::FailedToCreateKey)?;

        let connection_info = DbConnectionInfoColumn::AzureOpenAiApiKey {
            encrypted_api_key,
            api_endpoint: api_endpoint.to_string(),
            api_version: api_version.clone(),
            deployment_name: deployment_name.to_string(),
            salt,
        };

        self.__create_connection(&connection_info, budget_limits, request_limits, token_limits, &Some(application_secret.clone()))
            .await
    }

    pub async fn create_openai_v1_connection(
        &self,
        model: &str,
        api_endpoint: &str,
        api_key: &str,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>,
        application_secret: &Uuid,
    ) -> Result<Connection, DataAccessError> {
        let salt = Uuid::now_v7();
        let encrypted_api_key = encrypt(api_key, &salt, application_secret)
            .map_err(|_| DataAccessError::FailedToCreateKey)?;

        let connection_info = DbConnectionInfoColumn::OpenAiApiKey {
            encrypted_api_key,
            api_endpoint: api_endpoint.to_string(),
            model: model.to_string(),
            salt,
        };

        self.__create_connection(
            &connection_info,
            budget_limits,
            request_limits,
            token_limits,
            &Some(application_secret.clone()),
        )
        .await
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
        connection_info: &DbConnectionInfoColumn,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>
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
        connection_info: &DbConnectionInfoColumn,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>
    },
    search => { }
);
// region:      --- Postgres Queries
pub(crate) fn pg_search() -> QueryBuilder<'static, Postgres> {
    todo!()
}
pub(crate) fn pg_insert<'a>(
    connection_info: &'a DbConnectionInfoColumn,
    budget_limits: &'a Option<BudgetLimits>,
    request_limits: &'a Option<RequestLimits>,
    token_limits: &'a Option<TokenLimits>,
) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'a, Postgres> = QueryBuilder::new(
        "INSERT INTO connections
        (
            id,
            connection_info");

    if budget_limits.is_some() {
        query.push(", budget_limits");
    }
    if request_limits.is_some() {
        query.push(", request_limits");
    }
    if token_limits.is_some() {
        query.push(", token_limits");
    }
    query.push(")
        VALUES (gen_random_uuid(), ",
    );

    // Push connection_info
    query.push_bind(Json::from(connection_info));

    if let Some(limits) = budget_limits {
        query.push(", ");
        query.push_bind(Json::from(limits));
    }

    if let Some(limits) = request_limits {
        query.push(", ");
        query.push_bind(Json::from(limits));
    }

    if let Some(limits) = token_limits {
        query.push(", ");
        query.push_bind(Json::from(limits));
    }

    query.push(") RETURNING id");
    // Return builder
    query
}
pub(crate) fn pg_get(id: &ConnectionId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
            id, connection_info, budget_limits, request_limits, token_limits
            FROM connections
        WHERE id=",
    );
    // Push id
    query.push_bind(id);

    // Return builder
    query
}
pub(crate) fn pg_getm(ids: &Vec<ConnectionId>) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
            id, connection_info, budget_limits, request_limits, token_limits
            FROM connections
        WHERE id IN ( ",
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
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        DELETE FROM connections
        WHERE id=",
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
    pub(crate) token_limits: Option<sqlx::types::Json<TokenLimits>>,
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
    fn convert(
        self,
        application_secret: &Option<Uuid>,
    ) -> Result<ConnectionInfo, DataConversionError> {
        let application_secret = application_secret.ok_or(DataConversionError::DefaultError {
            cause: "Ups".to_string(),
        })?; // TODO return Internal Server Error
        match self {
            DbConnectionInfoColumn::AzureOpenAiApiKey {
                encrypted_api_key,
                api_endpoint,
                api_version,
                deployment_name,
                salt,
            } => Ok(ConnectionInfo::AzureOpenAiApiKey {
                api_key: decrypt(&encrypted_api_key, &salt, &application_secret).map_err(|_| {
                    DataConversionError::DefaultError {
                        cause: "Ups".to_string(),
                    }
                })?,
                api_endpoint,
                api_version,
                deployment_name,
            }),
            DbConnectionInfoColumn::OpenAiApiKey {
                encrypted_api_key,
                api_endpoint,
                model,
                salt,
            } => Ok(ConnectionInfo::OpenAiApiKey {
                api_key: decrypt(&encrypted_api_key, &salt, &application_secret).map_err(|_| {
                    DataConversionError::DefaultError {
                        cause: "Ups".to_string(),
                    }
                })?,
                api_endpoint,
                model,
            }),
        }
    }
}

impl ConvertInto<Connection> for DbConnectionRecord {
    fn convert(self, application_secret: &Option<Uuid>) -> Result<Connection, DataConversionError> {
        let connection_info = self
            .connection_info
            .0
            .convert(application_secret)
            .map_err(|_| DataConversionError::DefaultError {
                cause: "Ups".to_string(),
            })?;
        Ok(Connection::new(
            self.id,
            connection_info,
            self.budget_limits.map(|l| l.0).unwrap_or_default(),
            self.request_limits.map(|l| l.0).unwrap_or_default(),
            self.token_limits.map(|l| l.0).unwrap_or_default(),
        ))
    }
}

impl_with_id_parameter_for_struct!(DbConnectionRecord, ConnectionId);
// endregion: --- Database Model
