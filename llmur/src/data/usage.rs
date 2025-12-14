use crate::data::connection::{Connection, ConnectionId};
use crate::data::deployment::DeploymentId;
use crate::data::errors::{CacheError, DatabaseError};
use crate::data::graph::local_store::GraphData;
use crate::data::graph::usage_stats::{
    ConnectionUsageStats, DeploymentUsageStats, MetricsUsageStats, PeriodStats, ProjectUsageStats,
    StatValue, VirtualKeyUsageStats,
};
use crate::data::project::ProjectId;
use crate::data::request_log::{RequestLogData, pg_insert_m};
use crate::data::virtual_key::VirtualKeyId;
use crate::data::{Cache, DataAccess, Database, ExternalCache, connection};
use crate::errors::DataAccessError;
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use redis::Pipeline;
use redis::aio::MultiplexedConnection;
use sqlx::{Execute, FromRow, Postgres, QueryBuilder};
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::Instrument;

impl DataAccess {
    #[tracing::instrument(
        level="trace",
        name = "get.virtual_key.usage",
        skip(self, virtual_key_id, cached_stats_map, now_utc),
        fields(id = %virtual_key_id.0)
    )]
    pub(crate) async fn load_virtual_key_usage_and_set_cache(
        &self,
        virtual_key_id: &VirtualKeyId,
        cached_stats_map: &BTreeMap<String, Option<String>>,
        now_utc: &DateTime<Utc>,
    ) -> Result<VirtualKeyUsageStats, DataAccessError> {
        let virtual_key_cached_stats =
            VirtualKeyUsageStats::extract_from_map(virtual_key_id, now_utc, cached_stats_map);

        // If all the stats are available return the cached stats
        if !virtual_key_cached_stats.has_value_missing() {
            return Ok(virtual_key_cached_stats);
        }

        // If at least one of the usage stats is not cached we fetch it from our source of truth: The DB
        let record: DbUsageStatsRecord = self
            .database
            .load_virtual_key_usage(virtual_key_id, now_utc)
            .await?;

        let stats = VirtualKeyUsageStats::from_db_record(virtual_key_id, now_utc, record);
        match self.cache.set_usage_stats(&stats.0).await {
            Ok(_) => {
                println!(
                    "Successfully updated cached usage stats for {}",
                    virtual_key_id
                );
            }
            Err(e) => {
                println!(
                    "Failed to update usage stats for {}: {:?}",
                    virtual_key_id, e
                );
            }
        }

        Ok(stats)
    }

    #[tracing::instrument(
        level="trace",
        name = "get.connection.usage",
        skip(self, connection_id, cached_stats_map, now_utc),
        fields(id = %connection_id.0)
    )]
    pub(crate) async fn load_connection_usage_and_set_cache(
        &self,
        connection_id: &ConnectionId,
        cached_stats_map: &BTreeMap<String, Option<String>>,
        now_utc: &DateTime<Utc>,
    ) -> Result<ConnectionUsageStats, DataAccessError> {
        let connection_cached_stats =
            ConnectionUsageStats::extract_from_map(connection_id, now_utc, cached_stats_map);

        // If all the stats are available return the cached stats
        if !connection_cached_stats.has_value_missing() {
            return Ok(connection_cached_stats);
        }

        // If at least one of the usage stats is not cached we fetch it from our source of truth: The DB
        let record: DbUsageStatsRecord = self
            .database
            .load_connection_usage(connection_id, now_utc)
            .await?;

        let stats = ConnectionUsageStats::from_db_record(connection_id, now_utc, record);
        match self.cache.set_usage_stats(&stats.0).await {
            Ok(_) => {
                println!(
                    "Successfully updated cached usage stats for {}",
                    connection_id
                );
            }
            Err(e) => {
                println!(
                    "Failed to update usage stats for {}: {:?}",
                    connection_id, e
                );
            }
        }

        Ok(stats)
    }

    #[tracing::instrument(
        level="trace",
        name = "get.project.usage",
        skip(self, project_id, cached_stats_map, now_utc),
        fields(id = %project_id.0)
    )]
    pub(crate) async fn load_project_usage_and_set_cache(
        &self,
        project_id: &ProjectId,
        cached_stats_map: &BTreeMap<String, Option<String>>,
        now_utc: &DateTime<Utc>,
    ) -> Result<ProjectUsageStats, DataAccessError> {
        let project_cached_stats =
            ProjectUsageStats::extract_from_map(project_id, now_utc, cached_stats_map);

        // If all the stats are available return the cached stats
        if !project_cached_stats.has_value_missing() {
            return Ok(project_cached_stats);
        }

        // If at least one of the usage stats is not cached we fetch it from our source of truth: The DB
        let record: DbUsageStatsRecord = self
            .database
            .load_project_usage(project_id, now_utc)
            .await?;

        let stats = ProjectUsageStats::from_db_record(project_id, now_utc, record);
        match self.cache.set_usage_stats(&stats.0).await {
            Ok(_) => {
                println!("Successfully updated cached usage stats for {}", project_id);
            }
            Err(e) => {
                println!("Failed to update usage stats for {}: {:?}", project_id, e);
            }
        }

        Ok(stats)
    }

    #[tracing::instrument(
        level="trace",
        name = "get.deployment.usage",
        skip(self, deployment_id, cached_stats_map, now_utc),
        fields(id = %deployment_id.0)
    )]
    pub(crate) async fn load_deployment_usage_and_set_cache(
        &self,
        deployment_id: &DeploymentId,
        cached_stats_map: &BTreeMap<String, Option<String>>,
        now_utc: &DateTime<Utc>,
    ) -> Result<DeploymentUsageStats, DataAccessError> {
        let deployment_cached_stats =
            DeploymentUsageStats::extract_from_map(deployment_id, now_utc, cached_stats_map);

        // If all the stats are available return the cached stats
        if !deployment_cached_stats.has_value_missing() {
            return Ok(deployment_cached_stats);
        }

        // If at least one of the usage stats is not cached we fetch it from our source of truth: The DB
        let record: DbUsageStatsRecord = self
            .database
            .load_deployment_usage(deployment_id, now_utc)
            .await?;

        let stats = DeploymentUsageStats::from_db_record(deployment_id, now_utc, record);
        match self.cache.set_usage_stats(&stats.0).await {
            Ok(_) => {
                println!(
                    "Successfully updated cached usage stats for {}",
                    deployment_id
                );
            }
            Err(e) => {
                println!(
                    "Failed to update usage stats for {}: {:?}",
                    deployment_id, e
                );
            }
        }

        Ok(stats)
    }
}

impl Database {
    #[tracing::instrument(
        level="trace",
        name = "db.get.project.usage",
        skip(self, project_id, now_utc),
        fields(id = %project_id.0)
    )]
    pub(crate) async fn load_project_usage(
        &self,
        project_id: &ProjectId,
        now_utc: &DateTime<Utc>,
    ) -> Result<DbUsageStatsRecord, DataAccessError> {
        match self {
            Database::Postgres { pool } => {
                let mut query = pg_get_project_usage(project_id, now_utc);
                let sql = query.build_query_as::<DbUsageStatsRecord>();
                // TODO: Handle errors properly
                let result = sql
                    .fetch_one(pool)
                    .await
                    .map_err(|e| DatabaseError::SqlxError(e.to_string()))?;

                Ok(result)
            }
        }
    }

    #[tracing::instrument(
        level="trace",
        name = "db.get.virtual_key.usage",
        skip(self, virtual_key_id, now_utc),
        fields(id = %virtual_key_id.0)
    )]
    pub(crate) async fn load_virtual_key_usage(
        &self,
        virtual_key_id: &VirtualKeyId,
        now_utc: &DateTime<Utc>,
    ) -> Result<DbUsageStatsRecord, DataAccessError> {
        match self {
            Database::Postgres { pool } => {
                let mut query = pg_get_virtual_key_usage(virtual_key_id, now_utc);
                let sql = query.build_query_as::<DbUsageStatsRecord>();
                // TODO: Handle errors properly
                let result = sql
                    .fetch_one(pool)
                    .await
                    .map_err(|e| DatabaseError::SqlxError(e.to_string()))?;

                Ok(result)
            }
        }
    }

    #[tracing::instrument(
        level="trace",
        name = "db.get.deployment.usage",
        skip(self, deployment_id, now_utc),
        fields(id = %deployment_id.0)
    )]
    pub(crate) async fn load_deployment_usage(
        &self,
        deployment_id: &DeploymentId,
        now_utc: &DateTime<Utc>,
    ) -> Result<DbUsageStatsRecord, DataAccessError> {
        match self {
            Database::Postgres { pool } => {
                let mut query = pg_get_deployment_usage(deployment_id, now_utc);
                let sql = query.build_query_as::<DbUsageStatsRecord>();
                // TODO: Handle errors properly
                let result = sql
                    .fetch_one(pool)
                    .await
                    .map_err(|e| DatabaseError::SqlxError(e.to_string()))?;

                Ok(result)
            }
        }
    }
    #[tracing::instrument(
        level="trace",
        name = "db.get.connection.usage",
        skip(self, connection_id, now_utc),
        fields(id = %connection_id.0)
    )]
    pub(crate) async fn load_connection_usage(
        &self,
        connection_id: &ConnectionId,
        now_utc: &DateTime<Utc>,
    ) -> Result<DbUsageStatsRecord, DataAccessError> {
        match self {
            Database::Postgres { pool } => {
                let mut query = pg_get_connection_usage(connection_id, now_utc);
                let sql = query.build_query_as::<DbUsageStatsRecord>();
                // TODO: Handle errors properly
                let result = sql
                    .fetch_one(pool)
                    .await
                    .map_err(|e| DatabaseError::SqlxError(e.to_string()))?;

                Ok(result)
            }
        }
    }
}

impl Cache {
    #[tracing::instrument(        
        level="trace",
        name = "cache.set.usage_stats", 
        skip(self, stats)
    )]
    pub(crate) async fn set_usage_stats(
        &self,
        stats: &MetricsUsageStats,
    ) -> Result<(), CacheError> {
        let stats_map = stats.clone().into_usage_stat_map();

        if let Some(external) = &self.external {
            match external {
                ExternalCache::Redis { connection } => {
                    let span = tracing::trace_span!("redis.set.usage_stats");
                    async move {
                        let mut pipe: Pipeline = redis::pipe();
                        let mut conn: MultiplexedConnection = connection.clone();

                        for (key, value) in stats_map {
                            match value {
                                StatValue::Int(v) => {
                                    pipe.cmd("INCRBY")
                                        .arg(&key)
                                        .arg(v)
                                        .cmd("EXPIRE")
                                        .arg(&key)
                                        .arg(60);
                                }
                                StatValue::Float(v) => {
                                    pipe.cmd("INCRBYFLOAT")
                                        .arg(&key)
                                        .arg(v)
                                        .cmd("EXPIRE")
                                        .arg(&key)
                                        .arg(60);
                                }
                                StatValue::NotSet => {
                                    // Should always be set even if the value is zero. If this is executed it means something is not properly setup. Log a warning
                                    println!("Warn: Stat value is not set");
                                }
                            }
                        }

                        pipe.exec_async(&mut conn).await.map_err(|e| {
                            CacheError::RedisExecutionError {
                                cause: e.to_string(),
                            }
                        })
                    }
                    .instrument(span)
                    .await?;
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        level="trace",
        name = "cache.increment.usage_stats", 
        skip(self, records)
    )]
    pub(crate) async fn increment_usage_data(
        &self,
        records: Vec<Arc<RequestLogData>>,
    ) -> Result<(), CacheError> {
        let (requests_map, budget_map, tokens_map) = convert_records_to_cache_maps(records);

        if let Some(external) = &self.external {
            match external {
                ExternalCache::Redis { connection } => {
                    let span = tracing::trace_span!("redis.increment.usage_stats");
                    async move {
                        let mut pipe: Pipeline = redis::pipe();
                        let mut conn: MultiplexedConnection = connection.clone();

                        for (key, value) in requests_map {
                            pipe.cmd("INCRBY").arg(key).arg(value);
                        }
                        for (key, value) in budget_map {
                            pipe.cmd("INCRBYFLOAT").arg(key).arg(value);
                        }
                        for (key, value) in tokens_map {
                            pipe.cmd("INCRBY").arg(key).arg(value);
                        }

                        pipe.exec_async(&mut conn).await.map_err(|e| {
                            CacheError::RedisExecutionError {
                                cause: e.to_string(),
                            }
                        })
                    }
                    .instrument(span)
                    .await?;
                }
            }
        }

        Ok(())
    }
}

fn convert_records_to_cache_maps(
    records: Vec<Arc<RequestLogData>>,
) -> (
    BTreeMap<String, i64>,
    BTreeMap<String, f64>,
    BTreeMap<String, i64>,
) {
    records.into_iter().fold(
        (BTreeMap::new(), BTreeMap::new(), BTreeMap::new()),
        |(mut req_acc, mut bud_acc, mut tok_acc), record| {
            let requests = 1; // TODO: If the request fails, it will still count as a request. Need to decide how to handle this.
            let cost = record.cost.unwrap_or(0.0);
            let tokens = record.input_tokens.unwrap_or(0) + record.output_tokens.unwrap_or(0);

            // ----- requests -----
            req_acc.extend(
                ConnectionUsageStats::generate_request_keys_with_values(
                    &record.selected_connection_node.data.id,
                    &record.request_ts,
                    requests,
                )
                .into_iter()
                .chain(VirtualKeyUsageStats::generate_request_keys_with_values(
                    &record.graph.virtual_key.data.id,
                    &record.request_ts,
                    requests,
                ))
                .chain(DeploymentUsageStats::generate_request_keys_with_values(
                    &record.graph.deployment.data.id,
                    &record.request_ts,
                    requests,
                ))
                .chain(ProjectUsageStats::generate_request_keys_with_values(
                    &record.graph.project.data.id,
                    &record.request_ts,
                    requests,
                )),
            );

            // ----- budget / cost -----
            bud_acc.extend(
                ConnectionUsageStats::generate_budget_keys_with_values(
                    &record.selected_connection_node.data.id,
                    &record.request_ts,
                    cost,
                )
                .into_iter()
                .chain(VirtualKeyUsageStats::generate_budget_keys_with_values(
                    &record.graph.virtual_key.data.id,
                    &record.request_ts,
                    cost,
                ))
                .chain(DeploymentUsageStats::generate_budget_keys_with_values(
                    &record.graph.deployment.data.id,
                    &record.request_ts,
                    cost,
                ))
                .chain(ProjectUsageStats::generate_budget_keys_with_values(
                    &record.graph.project.data.id,
                    &record.request_ts,
                    cost,
                )),
            );

            // ----- tokens -----
            tok_acc.extend(
                ConnectionUsageStats::generate_token_keys_with_values(
                    &record.selected_connection_node.data.id,
                    &record.request_ts,
                    tokens,
                )
                .into_iter()
                .chain(VirtualKeyUsageStats::generate_token_keys_with_values(
                    &record.graph.virtual_key.data.id,
                    &record.request_ts,
                    tokens,
                ))
                .chain(DeploymentUsageStats::generate_token_keys_with_values(
                    &record.graph.deployment.data.id,
                    &record.request_ts,
                    tokens,
                ))
                .chain(ProjectUsageStats::generate_token_keys_with_values(
                    &record.graph.project.data.id,
                    &record.request_ts,
                    tokens,
                )),
            );

            (req_acc, bud_acc, tok_acc)
        },
    )
}

#[derive(FromRow, Clone, Debug)]
pub struct DbUsageStatsRecord {
    pub current_minute_cost: f64,
    pub current_minute_requests: i64,
    pub current_minute_tokens: i64,

    pub current_hour_cost: f64,
    pub current_hour_requests: i64,
    pub current_hour_tokens: i64,

    pub current_day_cost: f64,
    pub current_day_requests: i64,
    pub current_day_tokens: i64,

    pub current_month_cost: f64,
    pub current_month_requests: i64,
    pub current_month_tokens: i64,
}

pub(crate) fn pg_get_connection_usage<'a>(
    connection_id: &'a ConnectionId,
    ts: &'a DateTime<Utc>,
) -> QueryBuilder<'a, Postgres> {
    let current_minute = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), ts.day(), ts.hour(), ts.minute(), 0)
        .unwrap();
    let current_hour = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), ts.day(), ts.hour(), 0, 0)
        .unwrap();
    let current_day = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), ts.day(), 0, 0, 0)
        .unwrap();
    let current_month = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), 1, 0, 0, 0)
        .unwrap();

    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN request_ts >= "#,
    );

    query.push_bind(current_minute);
    query.push(" THEN cost ELSE 0 END), 0) as current_minute_cost,");
    query.push("COALESCE(COUNT(CASE WHEN request_ts >= ");
    query.push_bind(current_minute);
    query.push(" THEN 1 END), 0) as current_minute_requests,");
    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_minute);
    query.push(" THEN input_tokens + output_tokens ELSE 0 END), 0) as current_minute_tokens,");

    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_hour);
    query.push(" THEN cost ELSE 0 END), 0) as current_hour_cost,");
    query.push("COALESCE(COUNT(CASE WHEN request_ts >= ");
    query.push_bind(current_hour);
    query.push(" THEN 1 END), 0) as current_hour_requests,");
    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_hour);
    query.push(" THEN input_tokens + output_tokens ELSE 0 END), 0) as current_hour_tokens,");

    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_day);
    query.push(" THEN cost ELSE 0 END), 0) as current_day_cost,");
    query.push("COALESCE(COUNT(CASE WHEN request_ts >= ");
    query.push_bind(current_day);
    query.push(" THEN 1 END), 0) as current_day_requests,");
    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_day);
    query.push(" THEN input_tokens + output_tokens ELSE 0 END), 0) as current_day_tokens,");

    query.push("COALESCE(SUM(cost), 0) as current_month_cost,");
    query.push("COALESCE(COUNT(*), 0) as current_month_requests,");
    query.push("COALESCE(SUM(input_tokens + output_tokens), 0) as current_month_tokens ");

    query.push("FROM request_logs WHERE connection_id = ");
    query.push_bind(connection_id);
    query.push(" AND request_ts >= ");
    query.push_bind(current_month);

    query
}

pub(crate) fn pg_get_virtual_key_usage<'a>(
    virtual_key_id: &'a VirtualKeyId,
    ts: &'a DateTime<Utc>,
) -> QueryBuilder<'a, Postgres> {
    let current_minute = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), ts.day(), ts.hour(), ts.minute(), 0)
        .unwrap();
    let current_hour = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), ts.day(), ts.hour(), 0, 0)
        .unwrap();
    let current_day = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), ts.day(), 0, 0, 0)
        .unwrap();
    let current_month = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), 1, 0, 0, 0)
        .unwrap();

    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN request_ts >= "#,
    );

    query.push_bind(current_minute);
    query.push(" THEN cost ELSE 0 END), 0) as current_minute_cost,");
    query.push("COALESCE(COUNT(CASE WHEN request_ts >= ");
    query.push_bind(current_minute);
    query.push(" THEN 1 END), 0) as current_minute_requests,");
    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_minute);
    query.push(" THEN input_tokens + output_tokens ELSE 0 END), 0) as current_minute_tokens,");

    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_hour);
    query.push(" THEN cost ELSE 0 END), 0) as current_hour_cost,");
    query.push("COALESCE(COUNT(CASE WHEN request_ts >= ");
    query.push_bind(current_hour);
    query.push(" THEN 1 END), 0) as current_hour_requests,");
    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_hour);
    query.push(" THEN input_tokens + output_tokens ELSE 0 END), 0) as current_hour_tokens,");

    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_day);
    query.push(" THEN cost ELSE 0 END), 0) as current_day_cost,");
    query.push("COALESCE(COUNT(CASE WHEN request_ts >= ");
    query.push_bind(current_day);
    query.push(" THEN 1 END), 0) as current_day_requests,");
    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_day);
    query.push(" THEN input_tokens + output_tokens ELSE 0 END), 0) as current_day_tokens,");

    query.push("COALESCE(SUM(cost), 0) as current_month_cost,");
    query.push("COALESCE(COUNT(*), 0) as current_month_requests,");
    query.push("COALESCE(SUM(input_tokens + output_tokens), 0) as current_month_tokens ");

    query.push("FROM request_logs WHERE virtual_key_id = ");
    query.push_bind(virtual_key_id);
    query.push(" AND request_ts >= ");
    query.push_bind(current_month);

    query
}

pub(crate) fn pg_get_project_usage<'a>(
    project_id: &'a ProjectId,
    ts: &'a DateTime<Utc>,
) -> QueryBuilder<'a, Postgres> {
    let current_minute = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), ts.day(), ts.hour(), ts.minute(), 0)
        .unwrap();
    let current_hour = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), ts.day(), ts.hour(), 0, 0)
        .unwrap();
    let current_day = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), ts.day(), 0, 0, 0)
        .unwrap();
    let current_month = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), 1, 0, 0, 0)
        .unwrap();

    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN request_ts >= "#,
    );

    query.push_bind(current_minute);
    query.push(" THEN cost ELSE 0 END), 0) as current_minute_cost,");
    query.push("COALESCE(COUNT(CASE WHEN request_ts >= ");
    query.push_bind(current_minute);
    query.push(" THEN 1 END), 0) as current_minute_requests,");
    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_minute);
    query.push(" THEN input_tokens + output_tokens ELSE 0 END), 0) as current_minute_tokens,");

    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_hour);
    query.push(" THEN cost ELSE 0 END), 0) as current_hour_cost,");
    query.push("COALESCE(COUNT(CASE WHEN request_ts >= ");
    query.push_bind(current_hour);
    query.push(" THEN 1 END), 0) as current_hour_requests,");
    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_hour);
    query.push(" THEN input_tokens + output_tokens ELSE 0 END), 0) as current_hour_tokens,");

    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_day);
    query.push(" THEN cost ELSE 0 END), 0) as current_day_cost,");
    query.push("COALESCE(COUNT(CASE WHEN request_ts >= ");
    query.push_bind(current_day);
    query.push(" THEN 1 END), 0) as current_day_requests,");
    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_day);
    query.push(" THEN input_tokens + output_tokens ELSE 0 END), 0) as current_day_tokens,");

    query.push("COALESCE(SUM(cost), 0) as current_month_cost,");
    query.push("COALESCE(COUNT(*), 0) as current_month_requests,");
    query.push("COALESCE(SUM(input_tokens + output_tokens), 0) as current_month_tokens ");

    query.push("FROM request_logs WHERE project_id = ");
    query.push_bind(project_id);
    query.push(" AND request_ts >= ");
    query.push_bind(current_month);

    query
}

pub(crate) fn pg_get_deployment_usage<'a>(
    deployment_id: &'a DeploymentId,
    ts: &'a DateTime<Utc>,
) -> QueryBuilder<'a, Postgres> {
    let current_minute = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), ts.day(), ts.hour(), ts.minute(), 0)
        .unwrap();
    let current_hour = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), ts.day(), ts.hour(), 0, 0)
        .unwrap();
    let current_day = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), ts.day(), 0, 0, 0)
        .unwrap();
    let current_month = Utc
        .with_ymd_and_hms(ts.year(), ts.month(), 1, 0, 0, 0)
        .unwrap();

    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN request_ts >= "#,
    );

    query.push_bind(current_minute);
    query.push(" THEN cost ELSE 0 END), 0) as current_minute_cost,");
    query.push("COALESCE(COUNT(CASE WHEN request_ts >= ");
    query.push_bind(current_minute);
    query.push(" THEN 1 END), 0) as current_minute_requests,");
    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_minute);
    query.push(" THEN input_tokens + output_tokens ELSE 0 END), 0) as current_minute_tokens,");

    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_hour);
    query.push(" THEN cost ELSE 0 END), 0) as current_hour_cost,");
    query.push("COALESCE(COUNT(CASE WHEN request_ts >= ");
    query.push_bind(current_hour);
    query.push(" THEN 1 END), 0) as current_hour_requests,");
    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_hour);
    query.push(" THEN input_tokens + output_tokens ELSE 0 END), 0) as current_hour_tokens,");

    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_day);
    query.push(" THEN cost ELSE 0 END), 0) as current_day_cost,");
    query.push("COALESCE(COUNT(CASE WHEN request_ts >= ");
    query.push_bind(current_day);
    query.push(" THEN 1 END), 0) as current_day_requests,");
    query.push("COALESCE(SUM(CASE WHEN request_ts >= ");
    query.push_bind(current_day);
    query.push(" THEN input_tokens + output_tokens ELSE 0 END), 0) as current_day_tokens,");

    query.push("COALESCE(SUM(cost), 0) as current_month_cost,");
    query.push("COALESCE(COUNT(*), 0) as current_month_requests,");
    query.push("COALESCE(SUM(input_tokens + output_tokens), 0) as current_month_tokens ");

    query.push("FROM request_logs WHERE deployment_id = ");
    query.push_bind(deployment_id);
    query.push(" AND request_ts >= ");
    query.push_bind(current_month);

    query
}
