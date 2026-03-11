use crate::data::utils::ConvertInto;
use crate::data::virtual_file::VirtualFileId;
use crate::data::virtual_key::VirtualKeyId;
use crate::data::{DataAccess, Database};
use crate::errors::{DataAccessError, DbRecordConversionError};
use crate::metrics::Metrics;
use crate::{
    default_access_fns, default_database_access_fns, impl_structured_id_utils,
    impl_with_id_parameter_for_struct,
};
use chrono::{DateTime, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, Postgres, QueryBuilder};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

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
    FromRow,
)]
#[sqlx(transparent)]
pub struct VirtualBatchId(pub Uuid);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum VirtualBatchEndpoint {
    #[serde(rename = "/v1/responses")]
    Responses,
    #[serde(rename = "/v1/chat/completions")]
    ChatCompletions,
    #[serde(rename = "/v1/embeddings")]
    Embeddings,
    #[serde(rename = "/v1/completions")]
    Completions,
}

impl FromStr for VirtualBatchEndpoint {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "/v1/responses" => Ok(Self::Responses),
            "/v1/chat/completions" => Ok(Self::ChatCompletions),
            "/v1/embeddings" => Ok(Self::Embeddings),
            "/v1/completions" => Ok(Self::Completions),
            _ => Err(format!("Invalid batch endpoint '{}'.", value)),
        }
    }
}

impl AsRef<str> for VirtualBatchEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            Self::Responses => "/v1/responses",
            Self::ChatCompletions => "/v1/chat/completions",
            Self::Embeddings => "/v1/embeddings",
            Self::Completions => "/v1/completions",
        }
    }
}

impl Display for VirtualBatchEndpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum VirtualBatchCompletionWindow {
    #[serde(rename = "24h")]
    Hours24,
}

impl FromStr for VirtualBatchCompletionWindow {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "24h" => Ok(Self::Hours24),
            _ => Err(format!("Invalid completion window '{}'.", value)),
        }
    }
}

impl AsRef<str> for VirtualBatchCompletionWindow {
    fn as_ref(&self) -> &str {
        match self {
            Self::Hours24 => "24h",
        }
    }
}

impl Display for VirtualBatchCompletionWindow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VirtualBatchStatus {
    Validating,
    Failed,
    InProgress,
    Finalizing,
    Completed,
    Expired,
    Cancelling,
    Cancelled,
}

impl FromStr for VirtualBatchStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "validating" => Ok(Self::Validating),
            "failed" => Ok(Self::Failed),
            "in_progress" => Ok(Self::InProgress),
            "finalizing" => Ok(Self::Finalizing),
            "completed" => Ok(Self::Completed),
            "expired" => Ok(Self::Expired),
            "cancelling" => Ok(Self::Cancelling),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(format!("Invalid batch status '{}'.", value)),
        }
    }
}

impl AsRef<str> for VirtualBatchStatus {
    fn as_ref(&self) -> &str {
        match self {
            Self::Validating => "validating",
            Self::Failed => "failed",
            Self::InProgress => "in_progress",
            Self::Finalizing => "finalizing",
            Self::Completed => "completed",
            Self::Expired => "expired",
            Self::Cancelling => "cancelling",
            Self::Cancelled => "cancelled",
        }
    }
}

impl Display for VirtualBatchStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchRequestCounts {
    pub total: i64,
    pub completed: i64,
    pub failed: i64,
}

#[derive(Clone, Debug)]
pub struct VirtualBatch {
    pub id: VirtualBatchId,
    pub virtual_key_id: VirtualKeyId,
    pub input_file_id: VirtualFileId,
    pub endpoint: VirtualBatchEndpoint,
    pub completion_window: VirtualBatchCompletionWindow,
    pub status: VirtualBatchStatus,
    pub request_counts: Option<BatchRequestCounts>,
    pub metadata: Option<HashMap<String, String>>,
    pub output_file_id: Option<VirtualFileId>,
    pub error_file_id: Option<VirtualFileId>,
    pub in_progress_at: Option<i64>,
    pub expires_at: Option<i64>,
    pub finalizing_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub failed_at: Option<i64>,
    pub expired_at: Option<i64>,
    pub cancelling_at: Option<i64>,
    pub cancelled_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl VirtualBatch {
    pub(crate) fn new(
        id: VirtualBatchId,
        virtual_key_id: VirtualKeyId,
        input_file_id: VirtualFileId,
        endpoint: VirtualBatchEndpoint,
        completion_window: VirtualBatchCompletionWindow,
        status: VirtualBatchStatus,
        request_counts: Option<BatchRequestCounts>,
        metadata: Option<HashMap<String, String>>,
        output_file_id: Option<VirtualFileId>,
        error_file_id: Option<VirtualFileId>,
        in_progress_at: Option<i64>,
        expires_at: Option<i64>,
        finalizing_at: Option<i64>,
        completed_at: Option<i64>,
        failed_at: Option<i64>,
        expired_at: Option<i64>,
        cancelling_at: Option<i64>,
        cancelled_at: Option<i64>,
        created_at: i64,
        updated_at: i64,
    ) -> Self {
        VirtualBatch {
            id,
            virtual_key_id,
            input_file_id,
            endpoint,
            completion_window,
            status,
            request_counts,
            metadata,
            output_file_id,
            error_file_id,
            in_progress_at,
            expires_at,
            finalizing_at,
            completed_at,
            failed_at,
            expired_at,
            cancelling_at,
            cancelled_at,
            created_at,
            updated_at,
        }
    }
}

impl_structured_id_utils!(VirtualBatchId);
impl_with_id_parameter_for_struct!(VirtualBatch, VirtualBatchId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {
    #[tracing::instrument(
        level = "trace",
        name = "get.virtual_batch",
        skip(self, id, metrics),
        fields(id = %id.0)
    )]
    pub async fn get_virtual_batch(
        &self,
        id: &VirtualBatchId,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<Option<VirtualBatch>, DataAccessError> {
        self.__get_virtual_batch(id, &None, metrics).await
    }

    #[tracing::instrument(
        level = "trace",
        name = "get.virtual_batches",
        skip(self, ids, metrics),
        fields(ids = ?ids.iter().map(|id| id.0).collect::<Vec<Uuid>>())
    )]
    pub async fn get_virtual_batches(
        &self,
        ids: &BTreeSet<VirtualBatchId>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<BTreeMap<VirtualBatchId, Option<VirtualBatch>>, DataAccessError> {
        self.__get_virtual_batches(ids, &None, metrics).await
    }

    #[tracing::instrument(
        level = "trace",
        name = "create.virtual_batch",
        skip(
            self,
            id,
            virtual_key_id,
            input_file_id,
            endpoint,
            completion_window,
            status,
            request_counts,
            metadata,
            output_file_id,
            error_file_id,
            in_progress_at,
            expires_at,
            finalizing_at,
            completed_at,
            failed_at,
            expired_at,
            cancelling_at,
            cancelled_at,
            metrics
        ),
        fields(id = %id.0, virtual_key_id = %virtual_key_id.0)
    )]
    pub async fn create_virtual_batch(
        &self,
        id: &VirtualBatchId,
        virtual_key_id: &VirtualKeyId,
        input_file_id: &VirtualFileId,
        endpoint: &VirtualBatchEndpoint,
        completion_window: &VirtualBatchCompletionWindow,
        status: &VirtualBatchStatus,
        request_counts: &Option<BatchRequestCounts>,
        metadata: &Option<HashMap<String, String>>,
        output_file_id: &Option<VirtualFileId>,
        error_file_id: &Option<VirtualFileId>,
        in_progress_at: &Option<i64>,
        expires_at: &Option<i64>,
        finalizing_at: &Option<i64>,
        completed_at: &Option<i64>,
        failed_at: &Option<i64>,
        expired_at: &Option<i64>,
        cancelling_at: &Option<i64>,
        cancelled_at: &Option<i64>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<VirtualBatch, DataAccessError> {
        self.__create_virtual_batch(
            id,
            virtual_key_id,
            input_file_id,
            endpoint,
            completion_window,
            status,
            request_counts,
            metadata,
            output_file_id,
            error_file_id,
            in_progress_at,
            expires_at,
            finalizing_at,
            completed_at,
            failed_at,
            expired_at,
            cancelling_at,
            cancelled_at,
            &None,
            metrics,
        )
        .await
    }

    #[tracing::instrument(
        level = "trace",
        name = "search.virtual_batches",
        skip(self, virtual_key_id, limit, order, after, metrics),
        fields(virtual_key_id = %virtual_key_id.0)
    )]
    pub async fn search_virtual_batches(
        &self,
        virtual_key_id: &VirtualKeyId,
        limit: &Option<i64>,
        order: &Option<ListOrder>,
        after: &Option<VirtualBatchId>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<Vec<VirtualBatch>, DataAccessError> {
        self.__search_virtual_batches(virtual_key_id, limit, order, after, &None, metrics)
            .await
    }

    #[tracing::instrument(
        level = "trace",
        name = "delete.virtual_batch",
        skip(self, id, metrics),
        fields(id = %id.0)
    )]
    pub async fn delete_virtual_batch(
        &self,
        id: &VirtualBatchId,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<u64, DataAccessError> {
        self.__delete_virtual_batch(id, metrics).await
    }

    #[tracing::instrument(
        level = "trace",
        name = "update.virtual_batch",
        skip(
            self,
            id,
            status,
            request_counts,
            output_file_id,
            error_file_id,
            in_progress_at,
            expires_at,
            finalizing_at,
            completed_at,
            failed_at,
            expired_at,
            cancelling_at,
            cancelled_at,
            metrics
        ),
        fields(id = %id.0)
    )]
    pub async fn update_virtual_batch(
        &self,
        id: &VirtualBatchId,
        status: &Option<VirtualBatchStatus>,
        request_counts: &Option<Option<BatchRequestCounts>>,
        output_file_id: &Option<Option<VirtualFileId>>,
        error_file_id: &Option<Option<VirtualFileId>>,
        in_progress_at: &Option<Option<i64>>,
        expires_at: &Option<Option<i64>>,
        finalizing_at: &Option<Option<i64>>,
        completed_at: &Option<Option<i64>>,
        failed_at: &Option<Option<i64>>,
        expired_at: &Option<Option<i64>>,
        cancelling_at: &Option<Option<i64>>,
        cancelled_at: &Option<Option<i64>>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<VirtualBatch, DataAccessError> {
        if status.is_none()
            && request_counts.is_none()
            && output_file_id.is_none()
            && error_file_id.is_none()
            && in_progress_at.is_none()
            && expires_at.is_none()
            && finalizing_at.is_none()
            && completed_at.is_none()
            && failed_at.is_none()
            && expired_at.is_none()
            && cancelling_at.is_none()
            && cancelled_at.is_none()
        {
            return self
                .get_virtual_batch(id, metrics)
                .await?
                .ok_or(DataAccessError::ResourceNotFound);
        }

        let updated = self
            .database
            .update_virtual_batch(
                id,
                status,
                request_counts,
                output_file_id,
                error_file_id,
                in_progress_at,
                expires_at,
                finalizing_at,
                completed_at,
                failed_at,
                expired_at,
                cancelling_at,
                cancelled_at,
                metrics,
            )
            .await?;

        if updated == 0 {
            return Err(DataAccessError::ResourceNotFound);
        }

        self.get_virtual_batch(id, metrics)
            .await?
            .ok_or(DataAccessError::ResourceNotFound)
    }
}

default_access_fns!(
    VirtualBatch,
    VirtualBatchId,
    virtual_batch,
    virtual_batches,
    create => {
        id: &VirtualBatchId,
        virtual_key_id: &VirtualKeyId,
        input_file_id: &VirtualFileId,
        endpoint: &VirtualBatchEndpoint,
        completion_window: &VirtualBatchCompletionWindow,
        status: &VirtualBatchStatus,
        request_counts: &Option<BatchRequestCounts>,
        metadata: &Option<HashMap<String, String>>,
        output_file_id: &Option<VirtualFileId>,
        error_file_id: &Option<VirtualFileId>,
        in_progress_at: &Option<i64>,
        expires_at: &Option<i64>,
        finalizing_at: &Option<i64>,
        completed_at: &Option<i64>,
        failed_at: &Option<i64>,
        expired_at: &Option<i64>,
        cancelling_at: &Option<i64>,
        cancelled_at: &Option<i64>
    },
    search => {
        virtual_key_id: &VirtualKeyId,
        limit: &Option<i64>,
        order: &Option<ListOrder>,
        after: &Option<VirtualBatchId>
    }
);
// endregion: --- Data Access

// region:    --- Database Access

default_database_access_fns!(
    DbVirtualBatchRecord,
    VirtualBatchId,
    virtual_batch,
    virtual_batches,
    insert => {
        id: &VirtualBatchId,
        virtual_key_id: &VirtualKeyId,
        input_file_id: &VirtualFileId,
        endpoint: &VirtualBatchEndpoint,
        completion_window: &VirtualBatchCompletionWindow,
        status: &VirtualBatchStatus,
        request_counts: &Option<BatchRequestCounts>,
        metadata: &Option<HashMap<String, String>>,
        output_file_id: &Option<VirtualFileId>,
        error_file_id: &Option<VirtualFileId>,
        in_progress_at: &Option<i64>,
        expires_at: &Option<i64>,
        finalizing_at: &Option<i64>,
        completed_at: &Option<i64>,
        failed_at: &Option<i64>,
        expired_at: &Option<i64>,
        cancelling_at: &Option<i64>,
        cancelled_at: &Option<i64>
    },
    search => {
        virtual_key_id: &VirtualKeyId,
        limit: &Option<i64>,
        order: &Option<ListOrder>,
        after: &Option<VirtualBatchId>
    }
);

impl Database {
    #[tracing::instrument(
        level = "trace",
        name = "db.update.virtual_batch",
        skip(
            self,
            id,
            status,
            request_counts,
            output_file_id,
            error_file_id,
            in_progress_at,
            expires_at,
            finalizing_at,
            completed_at,
            failed_at,
            expired_at,
            cancelling_at,
            cancelled_at,
            metrics
        ),
        fields(id = %id.0)
    )]
    pub(crate) async fn update_virtual_batch(
        &self,
        id: &VirtualBatchId,
        status: &Option<VirtualBatchStatus>,
        request_counts: &Option<Option<BatchRequestCounts>>,
        output_file_id: &Option<Option<VirtualFileId>>,
        error_file_id: &Option<Option<VirtualFileId>>,
        in_progress_at: &Option<Option<i64>>,
        expires_at: &Option<Option<i64>>,
        finalizing_at: &Option<Option<i64>>,
        completed_at: &Option<Option<i64>>,
        failed_at: &Option<Option<i64>>,
        expired_at: &Option<Option<i64>>,
        cancelling_at: &Option<Option<i64>>,
        cancelled_at: &Option<Option<i64>>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<u64, DataAccessError> {
        use crate::metrics::RegisterDatabaseRequest;

        let operation = "db.update.virtual_batch";
        let span = tracing::trace_span!("database_operation", operation = %operation);

        tracing::Instrument::instrument(
            async move {
                match self {
                    Database::Postgres { pool } => {
                        let start = std::time::Instant::now();
                        let Some(mut query) = pg_update(
                            id,
                            status,
                            request_counts,
                            output_file_id,
                            error_file_id,
                            in_progress_at,
                            expires_at,
                            finalizing_at,
                            completed_at,
                            failed_at,
                            expired_at,
                            cancelling_at,
                            cancelled_at,
                        ) else {
                            return Ok(0);
                        };
                        let sql = query.build_query_as::<(VirtualBatchId,)>();
                        let result = sql.fetch_optional(pool).await;

                        metrics.register_database_request(
                            operation,
                            start.elapsed().as_millis() as u64,
                            result.is_ok(),
                        );

                        Ok(result?.map(|_| 1).unwrap_or(0))
                    }
                }
            },
            span,
        )
        .await
    }
}

// region:      --- Postgres Queries
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ListOrder {
    Asc,
    Desc,
}

fn order_clause(order: &Option<ListOrder>) -> &'static str {
    match order {
        Some(ListOrder::Asc) => "ASC",
        Some(ListOrder::Desc) | None => "DESC",
    }
}

pub(crate) fn pg_update<'a>(
    id: &'a VirtualBatchId,
    status: &'a Option<VirtualBatchStatus>,
    request_counts: &'a Option<Option<BatchRequestCounts>>,
    output_file_id: &'a Option<Option<VirtualFileId>>,
    error_file_id: &'a Option<Option<VirtualFileId>>,
    in_progress_at: &'a Option<Option<i64>>,
    expires_at: &'a Option<Option<i64>>,
    finalizing_at: &'a Option<Option<i64>>,
    completed_at: &'a Option<Option<i64>>,
    failed_at: &'a Option<Option<i64>>,
    expired_at: &'a Option<Option<i64>>,
    cancelling_at: &'a Option<Option<i64>>,
    cancelled_at: &'a Option<Option<i64>>,
) -> Option<QueryBuilder<'a, Postgres>> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("UPDATE virtual_batches SET ");
    let mut has_updates = false;

    if let Some(value) = status {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        query.push("status = ");
        query.push_bind(value.as_ref());
    }

    if let Some(value) = request_counts {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        match value {
            Some(value) => {
                query.push("request_counts = ");
                query.push_bind(Json::from(value));
            }
            None => {
                query.push("request_counts = NULL");
            }
        }
    }

    if let Some(value) = output_file_id {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        match value {
            Some(value) => {
                query.push("output_file_id = ");
                query.push_bind(value);
            }
            None => {
                query.push("output_file_id = NULL");
            }
        }
    }

    if let Some(value) = error_file_id {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        match value {
            Some(value) => {
                query.push("error_file_id = ");
                query.push_bind(value);
            }
            None => {
                query.push("error_file_id = NULL");
            }
        }
    }

    if let Some(value) = in_progress_at {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        match value {
            Some(value) => {
                query.push("in_progress_at = ");
                query.push_bind(DateTime::from_timestamp(*value, 0));
            }
            None => {
                query.push("in_progress_at = NULL");
            }
        }
    }

    if let Some(value) = expires_at {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        match value {
            Some(value) => {
                query.push("expires_at = ");
                query.push_bind(DateTime::from_timestamp(*value, 0));
            }
            None => {
                query.push("expires_at = NULL");
            }
        }
    }

    if let Some(value) = finalizing_at {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        match value {
            Some(value) => {
                query.push("finalizing_at = ");
                query.push_bind(DateTime::from_timestamp(*value, 0));
            }
            None => {
                query.push("finalizing_at = NULL");
            }
        }
    }

    if let Some(value) = completed_at {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        match value {
            Some(value) => {
                query.push("completed_at = ");
                query.push_bind(DateTime::from_timestamp(*value, 0));
            }
            None => {
                query.push("completed_at = NULL");
            }
        }
    }

    if let Some(value) = failed_at {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        match value {
            Some(value) => {
                query.push("failed_at = ");
                query.push_bind(DateTime::from_timestamp(*value, 0));
            }
            None => {
                query.push("failed_at = NULL");
            }
        }
    }

    if let Some(value) = expired_at {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        match value {
            Some(value) => {
                query.push("expired_at = ");
                query.push_bind(DateTime::from_timestamp(*value, 0));
            }
            None => {
                query.push("expired_at = NULL");
            }
        }
    }

    if let Some(value) = cancelling_at {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        match value {
            Some(value) => {
                query.push("cancelling_at = ");
                query.push_bind(DateTime::from_timestamp(*value, 0));
            }
            None => {
                query.push("cancelling_at = NULL");
            }
        }
    }

    if let Some(value) = cancelled_at {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        match value {
            Some(value) => {
                query.push("cancelled_at = ");
                query.push_bind(DateTime::from_timestamp(*value, 0));
            }
            None => {
                query.push("cancelled_at = NULL");
            }
        }
    }

    if !has_updates {
        return None;
    }

    query.push(" WHERE id = ");
    query.push_bind(id);
    query.push(" RETURNING id");

    Some(query)
}

pub(crate) fn pg_search<'a>(
    virtual_key_id: &'a VirtualKeyId,
    limit: &'a Option<i64>,
    order: &'a Option<ListOrder>,
    after: &'a Option<VirtualBatchId>,
) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
            id,
            virtual_key_id,
            input_file_id,
            endpoint,
            completion_window,
            status,
            request_counts,
            metadata,
            output_file_id,
            error_file_id,
            in_progress_at,
            expires_at,
            finalizing_at,
            completed_at,
            failed_at,
            expired_at,
            cancelling_at,
            cancelled_at,
            created_at,
            updated_at
        FROM
            virtual_batches
        WHERE
            virtual_key_id = ",
    );

    query.push_bind(virtual_key_id);

    if let Some(after) = after {
        let order_op = match order {
            Some(ListOrder::Asc) => ">",
            Some(ListOrder::Desc) | None => "<",
        };
        query.push(" AND (created_at, id) ");
        query.push(order_op);
        query.push(" (SELECT created_at, id FROM virtual_batches WHERE id = ");
        query.push_bind(after);
        query.push(")");
    }

    query.push(" ORDER BY created_at ");
    query.push(order_clause(order));
    query.push(", id ");
    query.push(order_clause(order));

    let limit_value = limit.unwrap_or(20);
    query.push(" LIMIT ");
    query.push_bind(limit_value);

    query
}

pub(crate) fn pg_get(id: &'_ VirtualBatchId) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
            id,
            virtual_key_id,
            input_file_id,
            endpoint,
            completion_window,
            status,
            request_counts,
            metadata,
            output_file_id,
            error_file_id,
            in_progress_at,
            expires_at,
            finalizing_at,
            completed_at,
            failed_at,
            expired_at,
            cancelling_at,
            cancelled_at,
            created_at,
            updated_at
        FROM
            virtual_batches
        WHERE
            id = ",
    );
    query.push_bind(id);
    query
}

pub(crate) fn pg_getm(ids: &'_ Vec<VirtualBatchId>) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
            id,
            virtual_key_id,
            input_file_id,
            endpoint,
            completion_window,
            status,
            request_counts,
            metadata,
            output_file_id,
            error_file_id,
            in_progress_at,
            expires_at,
            finalizing_at,
            completed_at,
            failed_at,
            expired_at,
            cancelling_at,
            cancelled_at,
            created_at,
            updated_at
        FROM
            virtual_batches
        WHERE
            id IN (",
    );

    let mut separated = query.separated(", ");
    for id in ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");

    query
}

pub(crate) fn pg_delete(id: &'_ VirtualBatchId) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        DELETE FROM virtual_batches
        WHERE id = ",
    );
    query.push_bind(id);
    query
}

pub(crate) fn pg_insert<'a>(
    id: &'a VirtualBatchId,
    virtual_key_id: &'a VirtualKeyId,
    input_file_id: &'a VirtualFileId,
    endpoint: &'a VirtualBatchEndpoint,
    completion_window: &'a VirtualBatchCompletionWindow,
    status: &'a VirtualBatchStatus,
    request_counts: &'a Option<BatchRequestCounts>,
    metadata: &'a Option<HashMap<String, String>>,
    output_file_id: &'a Option<VirtualFileId>,
    error_file_id: &'a Option<VirtualFileId>,
    in_progress_at: &'a Option<i64>,
    expires_at: &'a Option<i64>,
    finalizing_at: &'a Option<i64>,
    completed_at: &'a Option<i64>,
    failed_at: &'a Option<i64>,
    expired_at: &'a Option<i64>,
    cancelling_at: &'a Option<i64>,
    cancelled_at: &'a Option<i64>,
) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        INSERT INTO virtual_batches
        (
            id,
            virtual_key_id,
            input_file_id,
            endpoint,
            completion_window,
            status,
            request_counts,
            metadata,
            output_file_id,
            error_file_id,
            in_progress_at,
            expires_at,
            finalizing_at,
            completed_at,
            failed_at,
            expired_at,
            cancelling_at,
            cancelled_at
        ) VALUES (",
    );

    query.push_bind(id);
    query.push(", ");
    query.push_bind(virtual_key_id);
    query.push(", ");
    query.push_bind(input_file_id);
    query.push(", ");
    query.push_bind(endpoint.as_ref());
    query.push(", ");
    query.push_bind(completion_window.as_ref());
    query.push(", ");
    query.push_bind(status.as_ref());
    query.push(", ");
    query.push_bind(request_counts.as_ref().map(Json::from));
    query.push(", ");
    query.push_bind(metadata.as_ref().map(Json::from));
    query.push(", ");
    query.push_bind(output_file_id);
    query.push(", ");
    query.push_bind(error_file_id);
    query.push(", ");
    query.push_bind(
        in_progress_at
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .map(|value| value.naive_utc()),
    );
    query.push(", ");
    query.push_bind(
        expires_at
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .map(|value| value.naive_utc()),
    );
    query.push(", ");
    query.push_bind(
        finalizing_at
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .map(|value| value.naive_utc()),
    );
    query.push(", ");
    query.push_bind(
        completed_at
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .map(|value| value.naive_utc()),
    );
    query.push(", ");
    query.push_bind(
        failed_at
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .map(|value| value.naive_utc()),
    );
    query.push(", ");
    query.push_bind(
        expired_at
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .map(|value| value.naive_utc()),
    );
    query.push(", ");
    query.push_bind(
        cancelling_at
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .map(|value| value.naive_utc()),
    );
    query.push(", ");
    query.push_bind(
        cancelled_at
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .map(|value| value.naive_utc()),
    );

    query.push(") RETURNING id");

    query
}
// endregion:   --- Postgres Queries
// endregion: --- Database Access

// region:    --- Database Model
#[derive(FromRow, Clone, Debug)]
pub(crate) struct DbVirtualBatchRecord {
    pub(crate) id: VirtualBatchId,
    pub(crate) virtual_key_id: VirtualKeyId,
    pub(crate) input_file_id: VirtualFileId,
    pub(crate) endpoint: String,
    pub(crate) completion_window: String,
    pub(crate) status: String,
    pub(crate) request_counts: Option<sqlx::types::Json<BatchRequestCounts>>,
    pub(crate) metadata: Option<sqlx::types::Json<HashMap<String, String>>>,
    pub(crate) output_file_id: Option<VirtualFileId>,
    pub(crate) error_file_id: Option<VirtualFileId>,
    pub(crate) in_progress_at: Option<NaiveDateTime>,
    pub(crate) expires_at: Option<NaiveDateTime>,
    pub(crate) finalizing_at: Option<NaiveDateTime>,
    pub(crate) completed_at: Option<NaiveDateTime>,
    pub(crate) failed_at: Option<NaiveDateTime>,
    pub(crate) expired_at: Option<NaiveDateTime>,
    pub(crate) cancelling_at: Option<NaiveDateTime>,
    pub(crate) cancelled_at: Option<NaiveDateTime>,
    pub(crate) created_at: NaiveDateTime,
    pub(crate) updated_at: NaiveDateTime,
}

impl ConvertInto<VirtualBatch> for DbVirtualBatchRecord {
    fn convert(
        self,
        _application_secret: &Option<Uuid>,
    ) -> Result<VirtualBatch, DbRecordConversionError> {
        Ok(VirtualBatch::new(
            self.id,
            self.virtual_key_id,
            self.input_file_id,
            self.endpoint
                .parse::<VirtualBatchEndpoint>()
                .map_err(DbRecordConversionError::InternalError)?,
            self.completion_window
                .parse::<VirtualBatchCompletionWindow>()
                .map_err(DbRecordConversionError::InternalError)?,
            self.status
                .parse::<VirtualBatchStatus>()
                .map_err(DbRecordConversionError::InternalError)?,
            self.request_counts.map(|value| value.0),
            self.metadata.map(|value| value.0),
            self.output_file_id,
            self.error_file_id,
            self.in_progress_at.map(|value| value.and_utc().timestamp()),
            self.expires_at.map(|value| value.and_utc().timestamp()),
            self.finalizing_at.map(|value| value.and_utc().timestamp()),
            self.completed_at.map(|value| value.and_utc().timestamp()),
            self.failed_at.map(|value| value.and_utc().timestamp()),
            self.expired_at.map(|value| value.and_utc().timestamp()),
            self.cancelling_at.map(|value| value.and_utc().timestamp()),
            self.cancelled_at.map(|value| value.and_utc().timestamp()),
            self.created_at.and_utc().timestamp(),
            self.updated_at.and_utc().timestamp(),
        ))
    }
}

impl_with_id_parameter_for_struct!(DbVirtualBatchRecord, VirtualBatchId);
// endregion: --- Database Model
