use crate::data::deployment::DeploymentId;
use crate::data::utils::ConvertInto;
use crate::data::virtual_batch::{BatchRequestCounts, VirtualBatchId, VirtualBatchStatus};
use crate::data::{DataAccess, Database};
use crate::errors::{DataAccessError, DbRecordConversionError};
use crate::metrics::Metrics;
use crate::{
    default_access_fns, default_database_access_fns, impl_structured_id_utils,
    impl_with_id_parameter_for_struct,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, Postgres, QueryBuilder};
use std::collections::{BTreeMap, BTreeSet};
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
pub struct ProviderBatchId(pub Uuid);

#[derive(Clone, Debug)]
pub struct ProviderBatch {
    pub id: ProviderBatchId,
    pub batch_id: VirtualBatchId,
    pub provider: String,
    pub deployment_id: DeploymentId,
    pub provider_batch_id: String,
    pub status: VirtualBatchStatus,
    pub request_counts: Option<BatchRequestCounts>,
    pub output_file_id: Option<String>,
    pub error_file_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ProviderBatch {
    pub(crate) fn new(
        id: ProviderBatchId,
        batch_id: VirtualBatchId,
        provider: String,
        deployment_id: DeploymentId,
        provider_batch_id: String,
        status: VirtualBatchStatus,
        request_counts: Option<BatchRequestCounts>,
        output_file_id: Option<String>,
        error_file_id: Option<String>,
        created_at: i64,
        updated_at: i64,
    ) -> Self {
        ProviderBatch {
            id,
            batch_id,
            provider,
            deployment_id,
            provider_batch_id,
            status,
            request_counts,
            output_file_id,
            error_file_id,
            created_at,
            updated_at,
        }
    }
}

impl_structured_id_utils!(ProviderBatchId);
impl_with_id_parameter_for_struct!(ProviderBatch, ProviderBatchId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {
    #[tracing::instrument(
        level = "trace",
        name = "get.provider_batch",
        skip(self, id, metrics),
        fields(id = %id.0)
    )]
    pub async fn get_provider_batch(
        &self,
        id: &ProviderBatchId,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<Option<ProviderBatch>, DataAccessError> {
        self.__get_provider_batch(id, &None, metrics).await
    }

    #[tracing::instrument(
        level = "trace",
        name = "get.provider_batches",
        skip(self, ids, metrics),
        fields(ids = ?ids.iter().map(|id| id.0).collect::<Vec<Uuid>>())
    )]
    pub async fn get_provider_batches(
        &self,
        ids: &BTreeSet<ProviderBatchId>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<BTreeMap<ProviderBatchId, Option<ProviderBatch>>, DataAccessError> {
        self.__get_provider_batches(ids, &None, metrics).await
    }

    #[tracing::instrument(
        level = "trace",
        name = "create.provider_batch",
        skip(
            self,
            id,
            batch_id,
            provider,
            deployment_id,
            provider_batch_id,
            status,
            request_counts,
            output_file_id,
            error_file_id,
            metrics
        ),
        fields(id = %id.0, batch_id = %batch_id.0)
    )]
    pub async fn create_provider_batch(
        &self,
        id: &ProviderBatchId,
        batch_id: &VirtualBatchId,
        provider: &str,
        deployment_id: &DeploymentId,
        provider_batch_id: &str,
        status: &VirtualBatchStatus,
        request_counts: &Option<BatchRequestCounts>,
        output_file_id: &Option<String>,
        error_file_id: &Option<String>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<ProviderBatch, DataAccessError> {
        self.__create_provider_batch(
            id,
            batch_id,
            provider,
            deployment_id,
            provider_batch_id,
            status,
            request_counts,
            output_file_id,
            error_file_id,
            &None,
            metrics,
        )
        .await
    }

    #[tracing::instrument(
        level = "trace",
        name = "search.provider_batches",
        skip(self, batch_id, provider, metrics),
        fields(batch_id = %batch_id.0)
    )]
    pub async fn search_provider_batches(
        &self,
        batch_id: &VirtualBatchId,
        provider: &Option<String>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<Vec<ProviderBatch>, DataAccessError> {
        self.__search_provider_batches(batch_id, provider, &None, metrics)
            .await
    }

    #[tracing::instrument(
        level = "trace",
        name = "delete.provider_batch",
        skip(self, id, metrics),
        fields(id = %id.0)
    )]
    pub async fn delete_provider_batch(
        &self,
        id: &ProviderBatchId,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<u64, DataAccessError> {
        self.__delete_provider_batch(id, metrics).await
    }

    #[tracing::instrument(
        level = "trace",
        name = "update.provider_batch",
        skip(
            self,
            id,
            provider_batch_id,
            status,
            request_counts,
            output_file_id,
            error_file_id,
            metrics
        ),
        fields(id = %id.0)
    )]
    pub async fn update_provider_batch(
        &self,
        id: &ProviderBatchId,
        provider_batch_id: &Option<Option<String>>,
        status: &Option<VirtualBatchStatus>,
        request_counts: &Option<Option<BatchRequestCounts>>,
        output_file_id: &Option<Option<String>>,
        error_file_id: &Option<Option<String>>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<ProviderBatch, DataAccessError> {
        if provider_batch_id.is_none()
            && status.is_none()
            && request_counts.is_none()
            && output_file_id.is_none()
            && error_file_id.is_none()
        {
            return self
                .get_provider_batch(id, metrics)
                .await?
                .ok_or(DataAccessError::ResourceNotFound);
        }

        let updated = self
            .database
            .update_provider_batch(
                id,
                provider_batch_id,
                status,
                request_counts,
                output_file_id,
                error_file_id,
                metrics,
            )
            .await?;

        if updated == 0 {
            return Err(DataAccessError::ResourceNotFound);
        }

        self.get_provider_batch(id, metrics)
            .await?
            .ok_or(DataAccessError::ResourceNotFound)
    }
}

default_access_fns!(
    ProviderBatch,
    ProviderBatchId,
    provider_batch,
    provider_batches,
    create => {
        id: &ProviderBatchId,
        batch_id: &VirtualBatchId,
        provider: &str,
        deployment_id: &DeploymentId,
        provider_batch_id: &str,
        status: &VirtualBatchStatus,
        request_counts: &Option<BatchRequestCounts>,
        output_file_id: &Option<String>,
        error_file_id: &Option<String>
    },
    search => {
        batch_id: &VirtualBatchId,
        provider: &Option<String>
    }
);
// endregion: --- Data Access

// region:    --- Database Access
default_database_access_fns!(
    DbProviderBatchRecord,
    ProviderBatchId,
    provider_batch,
    provider_batches,
    insert => {
        id: &ProviderBatchId,
        batch_id: &VirtualBatchId,
        provider: &str,
        deployment_id: &DeploymentId,
        provider_batch_id: &str,
        status: &VirtualBatchStatus,
        request_counts: &Option<BatchRequestCounts>,
        output_file_id: &Option<String>,
        error_file_id: &Option<String>
    },
    search => {
        batch_id: &VirtualBatchId,
        provider: &Option<String>
    }
);

impl Database {
    #[tracing::instrument(
        level = "trace",
        name = "db.update.provider_batch",
        skip(
            self,
            id,
            provider_batch_id,
            status,
            request_counts,
            output_file_id,
            error_file_id,
            metrics
        ),
        fields(id = %id.0)
    )]
    pub(crate) async fn update_provider_batch(
        &self,
        id: &ProviderBatchId,
        provider_batch_id: &Option<Option<String>>,
        status: &Option<VirtualBatchStatus>,
        request_counts: &Option<Option<BatchRequestCounts>>,
        output_file_id: &Option<Option<String>>,
        error_file_id: &Option<Option<String>>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<u64, DataAccessError> {
        use crate::metrics::RegisterDatabaseRequest;

        let operation = "db.update.provider_batch";
        let span = tracing::trace_span!("database_operation", operation = %operation);

        tracing::Instrument::instrument(
            async move {
                match self {
                    Database::Postgres { pool } => {
                        let start = std::time::Instant::now();
                        let Some(mut query) = pg_update(
                            id,
                            provider_batch_id,
                            status,
                            request_counts,
                            output_file_id,
                            error_file_id,
                        ) else {
                            return Ok(0);
                        };
                        let sql = query.build_query_as::<(ProviderBatchId,)>();
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
pub(crate) fn pg_update<'a>(
    id: &'a ProviderBatchId,
    provider_batch_id: &'a Option<Option<String>>,
    status: &'a Option<VirtualBatchStatus>,
    request_counts: &'a Option<Option<BatchRequestCounts>>,
    output_file_id: &'a Option<Option<String>>,
    error_file_id: &'a Option<Option<String>>,
) -> Option<QueryBuilder<'a, Postgres>> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("UPDATE provider_batches SET ");
    let mut has_updates = false;

    if let Some(value) = provider_batch_id {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        match value {
            Some(value) => {
                query.push("provider_batch_id = ");
                query.push_bind(value);
            }
            None => {
                query.push("provider_batch_id = NULL");
            }
        }
    }

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

    if !has_updates {
        return None;
    }

    query.push(" WHERE id = ");
    query.push_bind(id);
    query.push(" RETURNING id");

    Some(query)
}

pub(crate) fn pg_search<'a>(
    batch_id: &'a VirtualBatchId,
    provider: &'a Option<String>,
) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
            id,
            batch_id,
            provider,
            deployment_id,
            provider_batch_id,
            status,
            request_counts,
            output_file_id,
            error_file_id,
            created_at,
            updated_at
        FROM
            provider_batches
        WHERE
            batch_id = ",
    );

    query.push_bind(batch_id);

    if let Some(provider) = provider {
        query.push(" AND provider = ");
        query.push_bind(provider);
    }

    query.push(" ORDER BY created_at ASC, id ASC");
    query
}

pub(crate) fn pg_get(id: &'_ ProviderBatchId) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
            id,
            batch_id,
            provider,
            deployment_id,
            provider_batch_id,
            status,
            request_counts,
            output_file_id,
            error_file_id,
            created_at,
            updated_at
        FROM
            provider_batches
        WHERE
            id = ",
    );
    query.push_bind(id);
    query
}

pub(crate) fn pg_getm(ids: &'_ Vec<ProviderBatchId>) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
            id,
            batch_id,
            provider,
            deployment_id,
            provider_batch_id,
            status,
            request_counts,
            output_file_id,
            error_file_id,
            created_at,
            updated_at
        FROM
            provider_batches
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

pub(crate) fn pg_delete(id: &'_ ProviderBatchId) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        DELETE FROM provider_batches
        WHERE id = ",
    );
    query.push_bind(id);
    query
}

pub(crate) fn pg_insert<'a>(
    id: &'a ProviderBatchId,
    batch_id: &'a VirtualBatchId,
    provider: &'a str,
    deployment_id: &'a DeploymentId,
    provider_batch_id: &'a str,
    status: &'a VirtualBatchStatus,
    request_counts: &'a Option<BatchRequestCounts>,
    output_file_id: &'a Option<String>,
    error_file_id: &'a Option<String>,
) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        INSERT INTO provider_batches
        (
            id,
            batch_id,
            provider,
            deployment_id,
            provider_batch_id,
            status,
            request_counts,
            output_file_id,
            error_file_id
        ) VALUES (",
    );

    query.push_bind(id);
    query.push(", ");
    query.push_bind(batch_id);
    query.push(", ");
    query.push_bind(provider);
    query.push(", ");
    query.push_bind(deployment_id);
    query.push(", ");
    query.push_bind(provider_batch_id);
    query.push(", ");
    query.push_bind(status.as_ref());
    query.push(", ");
    query.push_bind(request_counts.as_ref().map(Json::from));
    query.push(", ");
    query.push_bind(output_file_id);
    query.push(", ");
    query.push_bind(error_file_id);

    query.push(") RETURNING id");

    query
}
// endregion:   --- Postgres Queries
// endregion: --- Database Access

// region:    --- Database Model
#[derive(FromRow, Clone, Debug)]
pub(crate) struct DbProviderBatchRecord {
    pub(crate) id: ProviderBatchId,
    pub(crate) batch_id: VirtualBatchId,
    pub(crate) provider: String,
    pub(crate) deployment_id: DeploymentId,
    pub(crate) provider_batch_id: String,
    pub(crate) status: String,
    pub(crate) request_counts: Option<sqlx::types::Json<BatchRequestCounts>>,
    pub(crate) output_file_id: Option<String>,
    pub(crate) error_file_id: Option<String>,
    pub(crate) created_at: NaiveDateTime,
    pub(crate) updated_at: NaiveDateTime,
}

impl ConvertInto<ProviderBatch> for DbProviderBatchRecord {
    fn convert(
        self,
        _application_secret: &Option<Uuid>,
    ) -> Result<ProviderBatch, DbRecordConversionError> {
        Ok(ProviderBatch::new(
            self.id,
            self.batch_id,
            self.provider,
            self.deployment_id,
            self.provider_batch_id,
            self.status
                .parse::<VirtualBatchStatus>()
                .map_err(DbRecordConversionError::InternalError)?,
            self.request_counts.map(|value| value.0),
            self.output_file_id,
            self.error_file_id,
            self.created_at.and_utc().timestamp(),
            self.updated_at.and_utc().timestamp(),
        ))
    }
}

impl_with_id_parameter_for_struct!(DbProviderBatchRecord, ProviderBatchId);
// endregion: --- Database Model
