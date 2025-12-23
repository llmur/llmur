use std::sync::Arc;
use chrono::{DateTime, Utc};
use crate::data::connection::ConnectionId;
use crate::data::deployment::DeploymentId;
use crate::data::project::ProjectId;
use crate::data::utils::ConvertInto;
use crate::data::virtual_key::VirtualKeyId;
use crate::data::{DataAccess, Database};
use crate::errors::{DataAccessError, DbRecordConversionError};
use crate::{default_access_fns, default_database_access_fns, impl_structured_id_utils, impl_with_id_parameter_for_struct};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use uuid::Uuid;
use crate::data::graph::{ConnectionNode, Graph};
use crate::metrics::Metrics;

// region:    --- Main Model
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
pub struct RequestLogId(pub Uuid);
#[derive(Clone, Debug, Serialize)]
pub struct RequestLog {
    pub id: RequestLogId,
    pub attempt_number: i16,

    pub virtual_key_id: VirtualKeyId,
    pub deployment_id: DeploymentId,
    pub connection_id: ConnectionId,
    pub project_id: ProjectId,

    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,

    pub cost: f64,

    pub http_status_code: i16,
    pub error: Option<String>,

    pub request_ts: i64,
    pub response_ts: i64,

    pub method: String,
    pub path: String,
    pub provider: String,
    pub deployment_name: String,
    pub project_name: String,
    pub virtual_key_alias: String,

    pub created_at: i64,
}

impl_structured_id_utils!(RequestLogId);
impl_with_id_parameter_for_struct!(RequestLog, RequestLogId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {

    #[tracing::instrument(
        level="trace",
        name = "get.request_log",
        skip(self, id, metrics),
        fields(
            id = %id.0
        )
    )]
    pub async fn get_request_log(&self, id: &RequestLogId, metrics: &Option<Arc<Metrics>>) -> Result<Option<RequestLog>, DataAccessError> {
        self.__get_request_log(id, &None, metrics).await
    }


    #[tracing::instrument(
        level="trace",
        name = "delete.project_invite_code",
        skip(self, id, metrics),
        fields(
            id = %id.0
        )
    )]
    pub async fn delete_request_log(&self, id: &RequestLogId, metrics: &Option<Arc<Metrics>>) -> Result<u64, DataAccessError> {
        self.__delete_request_log(id, metrics).await
    }
}

default_access_fns!(
        RequestLog,
        RequestLogId,
        request_log,
        request_logs,
        create => {
            id: &RequestLogId,

            virtual_key_id: &VirtualKeyId,
            project_id: &ProjectId,
            deployment_id: &DeploymentId,
            connection_id: &ConnectionId,

            input_tokens: i64,
            output_tokens: i64,

            cost: f64,

            http_status_code: i16,
            error: &Option<String>,

            request_ts: &chrono::DateTime<chrono::Utc>,
            response_ts: &chrono::DateTime<chrono::Utc>,

            method: &String,
            path: &String,
            provider: &String,
            deployment_name: &String,
            project_name: &String,
            virtual_key_alias: &String
        },
        search => {}
    );
// endregion: --- Data Access

// region:    --- Database Access
impl Database {
    pub async fn insert_request_logs(&self, request_logs: &Vec<Arc<RequestLogData>>, metrics: &Option<Arc<Metrics>>) -> Result<u64, DataAccessError> {
        match self {
            Database::Postgres { pool } => {
                let mut query = pg_insert_m(request_logs);
                let sql = query.build();
                let result = sql.execute(pool).await;

                Ok(result.map(|qr| qr.rows_affected())?)
            }
        }
    }
}

default_database_access_fns!(
    DbRequestLogRecord,
    RequestLogId,
    request_log,
    request_logs,
    insert => {
        id: &RequestLogId,

        virtual_key_id: &VirtualKeyId,
        project_id: &ProjectId,
        deployment_id: &DeploymentId,
        connection_id: &ConnectionId,

        input_tokens: i64,
        output_tokens: i64,

        cost: f64,

        http_status_code: i16,
        error: &Option<String>,

        request_ts: &chrono::DateTime<chrono::Utc>,
        response_ts: &chrono::DateTime<chrono::Utc>,

        method: &String,
        path: &String,
        provider: &String,
        deployment_name: &String,
        project_name: &String,
        virtual_key_alias: &String
    },
    search => { }
);
// region:      --- Postgres Queries

#[allow(unused)]
pub(crate) fn pg_search() -> QueryBuilder<'static, Postgres> {
    unimplemented!()
}

#[allow(unused)]
pub(crate) fn pg_get(id: &RequestLogId) -> QueryBuilder<Postgres> {
    unimplemented!()
}

#[allow(unused)]
pub(crate) fn pg_getm(ids: &Vec<RequestLogId>) -> QueryBuilder<Postgres> {
    unimplemented!()
}

#[allow(unused)]
pub(crate) fn pg_delete(id: &RequestLogId) -> QueryBuilder<Postgres> {
    unimplemented!()
}

pub(crate) fn pg_insert_m(
    request_logs: &Vec<Arc<RequestLogData>>,
) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        INSERT INTO request_logs
        (
            id,
            attempt_number,
            virtual_key_id,
            project_id,
            deployment_id,
            connection_id,

            input_tokens,
            output_tokens,

            cost,

            http_status_code,
            error,

            request_ts,
            response_ts,

            method,
            path,
            provider,
            deployment_name,
            project_name,
            virtual_key_alias
        )
        ");

    query.push_values(request_logs, |mut b, log| {
        b.push_bind(log.id)
            .push_bind(log.attempt_number)
            .push_bind(log.graph.virtual_key.data.id)
            .push_bind(log.graph.project.data.id)
            .push_bind(log.graph.deployment.data.id)
            .push_bind(log.selected_connection_node.data.id)
            .push_bind(log.input_tokens.unwrap_or(0))
            .push_bind(log.output_tokens.unwrap_or(0))
            .push_bind(log.cost.unwrap_or(0.0))
            .push_bind(log.http_status_code)
            .push_bind(&log.error)
            .push_bind(log.request_ts)
            .push_bind(log.response_ts)
            .push_bind(&log.method)
            .push_bind(&log.path)
            .push_bind(log.selected_connection_node.data.connection_info.get_provider_friendly_name())
            .push_bind(&log.graph.deployment.data.name)
            .push_bind(&log.graph.project.data.name)
            .push_bind(&log.graph.virtual_key.data.alias);
    });

    println!("### Built batch insert query for {} request logs", request_logs.len());
    query
}

#[allow(unused)]
pub(crate) fn pg_insert<'a>(
    id: &'a RequestLogId,
    virtual_key_id: &'a VirtualKeyId,
    project_id: &'a ProjectId,
    deployment_id: &'a DeploymentId,
    connection_id: &'a ConnectionId,

    input_tokens: i64,
    output_tokens: i64,

    cost: f64,

    http_status_code: i16,
    error: &'a Option<String>,

    request_ts: &'a chrono::DateTime<chrono::Utc>,
    response_ts: &'a chrono::DateTime<chrono::Utc>,

    method: &'a str,
    path: &'a str,
    provider: &'a str,
    deployment_name: &'a str,
    project_name: &'a str,
    virtual_key_alias: &'a str,

) -> QueryBuilder<'a, Postgres> {
    unimplemented!();
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        INSERT INTO request_logs
        (
            id,
            virtual_key_id,
            project_id,
            deployment_id,
            connection_id,

            input_tokens,
            output_tokens,

            cost,

            http_status_code,
            error,

            request_ts,
            response_ts,

            method,
            path,
            provider,
            deployment_name,
            project_name,
            virtual_key_alias,
        )
        VALUES (");
    // Push id
    query.push_bind(id);
    query.push(", ");
    // Push virtual_key_id
    query.push_bind(virtual_key_id);
    query.push(", ");
    // Push project_id
    query.push_bind(project_id);
    query.push(", ");
    // Push deployment_id
    query.push_bind(deployment_id);
    query.push(", ");
    // Push connection_id
    query.push_bind(connection_id);
    query.push(", ");
    // Push input_tokens
    query.push_bind(input_tokens);
    query.push(", ");
    // Push output_tokens
    query.push_bind(output_tokens);
    query.push(", ");
    // Push cost
    query.push_bind(cost);
    query.push(", ");
    // Push http_status_code
    query.push_bind(http_status_code);
    query.push(", ");
    // Push error
    query.push_bind(error);
    query.push(", ");
    // Push request_ts
    query.push_bind(request_ts);
    query.push(", ");
    // Push response_ts
    query.push_bind(response_ts);
    query.push(", ");
    // Push method
    query.push_bind(method);
    query.push(", ");
    // Push path
    query.push_bind(path);
    query.push(", ");
    // Push provider
    query.push_bind(provider);
    query.push(", ");
    // Push deployment_name
    query.push_bind(deployment_name);
    query.push(", ");
    // Push project_name
    query.push_bind(project_name);
    query.push(", ");
    // Push virtual_key_alias
    query.push_bind(virtual_key_alias);

    // Push the rest of the query
    query.push(") RETURNING id");
    // Return builder
    query
}
// endregion:   --- Postgres Queries
// endregion: --- Database Access

// region:    --- Database Model
#[derive(FromRow, Clone, Debug)]
pub(crate) struct DbRequestLogRecord {
    pub id: RequestLogId,
    pub attempt_number: i16,

    pub virtual_key_id: VirtualKeyId,
    pub deployment_id: DeploymentId,
    pub connection_id: ConnectionId,
    pub project_id: ProjectId,

    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,

    pub cost: f64,

    pub http_status_code: i16,
    pub error: Option<String>,

    pub request_ts: chrono::DateTime<chrono::Utc>,
    pub response_ts: chrono::DateTime<chrono::Utc>,

    pub method: String,
    pub path: String,
    pub provider: String,
    pub deployment_name: String,
    pub project_name: String,
    pub virtual_key_alias: String,

    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ConvertInto<RequestLog> for DbRequestLogRecord {
    fn convert(self, _application_secret: &Option<Uuid>) -> Result<RequestLog, DbRecordConversionError> {
        Ok(RequestLog {
            id: self.id,
            attempt_number: self.attempt_number,
            virtual_key_id: self.virtual_key_id,
            deployment_id: self.deployment_id,
            connection_id: self.connection_id,
            project_id: self.project_id,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            total_tokens: self.total_tokens,
            cost: self.cost,
            http_status_code: self.http_status_code,
            error: self.error,
            request_ts: self.request_ts.timestamp(),
            response_ts: self.response_ts.timestamp(),
            method: self.method,
            path: self.path,
            provider: self.provider,
            deployment_name: self.deployment_name,
            project_name: self.project_name,
            virtual_key_alias: self.virtual_key_alias,
            created_at: self.created_at.timestamp()
        })
    }
}

impl_with_id_parameter_for_struct!(DbRequestLogRecord, RequestLogId);
// endregion: --- Database Model

// region:    --- Helpers
// Used to simplify the payload of batch request method
#[derive(Clone, Debug)]
pub(crate) struct RequestLogData {
    pub(crate) id: RequestLogId,
    pub(crate) attempt_number: i16,

    pub(crate) graph: Graph,
    pub(crate) selected_connection_node: ConnectionNode,

    pub(crate) input_tokens: Option<i64>,
    pub(crate) output_tokens: Option<i64>,

    pub(crate) cost: Option<f64>,

    pub(crate) http_status_code: i16,
    pub(crate) error: Option<String>,

    pub(crate) request_ts: DateTime<Utc>,
    pub(crate) response_ts: DateTime<Utc>,

    pub(crate) method: String,
    pub(crate) path: String
}
// endregion: --- Helpers