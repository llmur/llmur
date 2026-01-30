use crate::data::DataAccess;
use crate::data::connection::Connection;
use crate::data::connection_deployment::{ConnectionDeployment, ConnectionDeploymentId};
use crate::data::deployment::Deployment;
use crate::data::graph::local_store::{GraphData, GraphDataId};
use crate::data::graph::usage_stats::{
    ConnectionUsageStats, DeploymentUsageStats, ProjectUsageStats, VirtualKeyUsageStats,
};
use crate::data::project::Project;
use crate::data::virtual_key::VirtualKey;
use crate::data::virtual_key_deployment::VirtualKeyDeploymentId;
use crate::errors::{
    DataAccessError, GraphLoadError, InconsistentGraphDataError, UsageExceededError,
};
use crate::metrics::Metrics;
use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use log::error;
use serde::Serialize;
use std::sync::Arc;
use tracing::{Instrument, instrument, trace_span};
use uuid::Uuid;

pub(crate) mod external_cache;
pub(crate) mod local_store;
pub(crate) mod usage_stats;

// region:    --- Main Model
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Graph {
    pub(crate) virtual_key: VirtualKeyNode,
    pub(crate) deployment: DeploymentNode,
    pub(crate) project: ProjectNode,
    pub(crate) connections: Vec<ConnectionNode>,
}

macro_rules! check_limit {
    ($current:expr, $limit:expr, $error:ident) => {
        if let Some(limit) = $limit {
            let used = $current;
            if used > limit {
                return Err(UsageExceededError::$error { used, limit });
            }
        }
    };
}

// region:    --- Virtual Key Node
// Simplified version of VirtualKey for graph representation
#[derive(Clone, Debug, Serialize)]
pub(crate) struct VirtualKeyNode {
    pub(crate) data: VirtualKey,
    pub(crate) usage_stats: VirtualKeyUsageStats,
}

impl NodeLimitsChecker for VirtualKeyNode {
    fn validate_limits(&self) -> Result<(), UsageExceededError> {
        let b = self.usage_stats.budget();
        check_limit!(
            b.current_month.value().as_f64(),
            self.data.budget_limits.cost_per_month,
            MonthBudgetOverLimit
        );
        check_limit!(
            b.current_day.value().as_f64(),
            self.data.budget_limits.cost_per_day,
            DayBudgetOverLimit
        );
        check_limit!(
            b.current_hour.value().as_f64(),
            self.data.budget_limits.cost_per_hour,
            HourBudgetOverLimit
        );
        check_limit!(
            b.current_minute.value().as_f64(),
            self.data.budget_limits.cost_per_minute,
            MinuteBudgetOverLimit
        );

        let r = &self.usage_stats.requests();
        check_limit!(
            r.current_month.value().as_i64(),
            self.data.request_limits.requests_per_month,
            MonthRequestsOverLimit
        );
        check_limit!(
            r.current_day.value().as_i64(),
            self.data.request_limits.requests_per_day,
            DayRequestsOverLimit
        );
        check_limit!(
            r.current_hour.value().as_i64(),
            self.data.request_limits.requests_per_hour,
            HourRequestsOverLimit
        );
        check_limit!(
            r.current_minute.value().as_i64(),
            self.data.request_limits.requests_per_minute,
            MinuteRequestsOverLimit
        );

        let r = &self.usage_stats.tokens();
        check_limit!(
            r.current_month.value().as_i64(),
            self.data.token_limits.tokens_per_month,
            MonthTokensOverLimit
        );
        check_limit!(
            r.current_day.value().as_i64(),
            self.data.token_limits.tokens_per_day,
            DayTokensOverLimit
        );
        check_limit!(
            r.current_hour.value().as_i64(),
            self.data.token_limits.tokens_per_hour,
            HourTokensOverLimit
        );
        check_limit!(
            r.current_minute.value().as_i64(),
            self.data.token_limits.tokens_per_minute,
            MinuteTokensOverLimit
        );

        Ok(())
    }
}

// endregion: --- Virtual Key Node

// region:    --- Deployment Node
// Simplified version of Deployment for graph representation
#[derive(Clone, Debug, Serialize)]
pub(crate) struct DeploymentNode {
    pub(crate) data: Deployment,
    pub(crate) association_id: VirtualKeyDeploymentId,
    pub(crate) usage_stats: DeploymentUsageStats,
}

impl NodeLimitsChecker for DeploymentNode {
    fn validate_limits(&self) -> Result<(), UsageExceededError> {
        let b = self.usage_stats.budget();
        check_limit!(
            b.current_month.value().as_f64(),
            self.data.budget_limits.cost_per_month,
            MonthBudgetOverLimit
        );
        check_limit!(
            b.current_day.value().as_f64(),
            self.data.budget_limits.cost_per_day,
            DayBudgetOverLimit
        );
        check_limit!(
            b.current_hour.value().as_f64(),
            self.data.budget_limits.cost_per_hour,
            HourBudgetOverLimit
        );
        check_limit!(
            b.current_minute.value().as_f64(),
            self.data.budget_limits.cost_per_minute,
            MinuteBudgetOverLimit
        );

        let r = &self.usage_stats.requests();
        check_limit!(
            r.current_month.value().as_i64(),
            self.data.request_limits.requests_per_month,
            MonthRequestsOverLimit
        );
        check_limit!(
            r.current_day.value().as_i64(),
            self.data.request_limits.requests_per_day,
            DayRequestsOverLimit
        );
        check_limit!(
            r.current_hour.value().as_i64(),
            self.data.request_limits.requests_per_hour,
            HourRequestsOverLimit
        );
        check_limit!(
            r.current_minute.value().as_i64(),
            self.data.request_limits.requests_per_minute,
            MinuteRequestsOverLimit
        );

        let r = &self.usage_stats.tokens();
        check_limit!(
            r.current_month.value().as_i64(),
            self.data.token_limits.tokens_per_month,
            MonthTokensOverLimit
        );
        check_limit!(
            r.current_day.value().as_i64(),
            self.data.token_limits.tokens_per_day,
            DayTokensOverLimit
        );
        check_limit!(
            r.current_hour.value().as_i64(),
            self.data.token_limits.tokens_per_hour,
            HourTokensOverLimit
        );
        check_limit!(
            r.current_minute.value().as_i64(),
            self.data.token_limits.tokens_per_minute,
            MinuteTokensOverLimit
        );

        Ok(())
    }
}
// endregion: --- Deployment Node

// region:    --- Connection Node
// Simplified version of Connection for graph representation
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ConnectionNode {
    pub(crate) data: Connection,
    pub(crate) association_id: ConnectionDeploymentId,
    pub(crate) weight: u16,
    pub(crate) usage_stats: ConnectionUsageStats,
}

impl NodeLimitsChecker for ConnectionNode {
    fn validate_limits(&self) -> Result<(), UsageExceededError> {
        let b = self.usage_stats.budget();
        check_limit!(
            b.current_month.value().as_f64(),
            self.data.budget_limits.cost_per_month,
            MonthBudgetOverLimit
        );
        check_limit!(
            b.current_day.value().as_f64(),
            self.data.budget_limits.cost_per_day,
            DayBudgetOverLimit
        );
        check_limit!(
            b.current_hour.value().as_f64(),
            self.data.budget_limits.cost_per_hour,
            HourBudgetOverLimit
        );
        check_limit!(
            b.current_minute.value().as_f64(),
            self.data.budget_limits.cost_per_minute,
            MinuteBudgetOverLimit
        );

        let r = &self.usage_stats.requests();
        check_limit!(
            r.current_month.value().as_i64(),
            self.data.request_limits.requests_per_month,
            MonthRequestsOverLimit
        );
        check_limit!(
            r.current_day.value().as_i64(),
            self.data.request_limits.requests_per_day,
            DayRequestsOverLimit
        );
        check_limit!(
            r.current_hour.value().as_i64(),
            self.data.request_limits.requests_per_hour,
            HourRequestsOverLimit
        );
        check_limit!(
            r.current_minute.value().as_i64(),
            self.data.request_limits.requests_per_minute,
            MinuteRequestsOverLimit
        );

        let r = &self.usage_stats.tokens();
        check_limit!(
            r.current_month.value().as_i64(),
            self.data.token_limits.tokens_per_month,
            MonthTokensOverLimit
        );
        check_limit!(
            r.current_day.value().as_i64(),
            self.data.token_limits.tokens_per_day,
            DayTokensOverLimit
        );
        check_limit!(
            r.current_hour.value().as_i64(),
            self.data.token_limits.tokens_per_hour,
            HourTokensOverLimit
        );
        check_limit!(
            r.current_minute.value().as_i64(),
            self.data.token_limits.tokens_per_minute,
            MinuteTokensOverLimit
        );

        Ok(())
    }
}
// endregion: --- Connection Node

// region:    --- Project Node
// Simplified version of Project for graph representation
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ProjectNode {
    pub(crate) data: Project,
    pub(crate) usage_stats: ProjectUsageStats,
}

impl NodeLimitsChecker for ProjectNode {
    fn validate_limits(&self) -> Result<(), UsageExceededError> {
        let b = self.usage_stats.budget();
        check_limit!(
            b.current_month.value().as_f64(),
            self.data.budget_limits.cost_per_month,
            MonthBudgetOverLimit
        );
        check_limit!(
            b.current_day.value().as_f64(),
            self.data.budget_limits.cost_per_day,
            DayBudgetOverLimit
        );
        check_limit!(
            b.current_hour.value().as_f64(),
            self.data.budget_limits.cost_per_hour,
            HourBudgetOverLimit
        );
        check_limit!(
            b.current_minute.value().as_f64(),
            self.data.budget_limits.cost_per_minute,
            MinuteBudgetOverLimit
        );

        let r = &self.usage_stats.requests();
        check_limit!(
            r.current_month.value().as_i64(),
            self.data.request_limits.requests_per_month,
            MonthRequestsOverLimit
        );
        check_limit!(
            r.current_day.value().as_i64(),
            self.data.request_limits.requests_per_day,
            DayRequestsOverLimit
        );
        check_limit!(
            r.current_hour.value().as_i64(),
            self.data.request_limits.requests_per_hour,
            HourRequestsOverLimit
        );
        check_limit!(
            r.current_minute.value().as_i64(),
            self.data.request_limits.requests_per_minute,
            MinuteRequestsOverLimit
        );

        let r = &self.usage_stats.tokens();
        check_limit!(
            r.current_month.value().as_i64(),
            self.data.token_limits.tokens_per_month,
            MonthTokensOverLimit
        );
        check_limit!(
            r.current_day.value().as_i64(),
            self.data.token_limits.tokens_per_day,
            DayTokensOverLimit
        );
        check_limit!(
            r.current_hour.value().as_i64(),
            self.data.token_limits.tokens_per_hour,
            HourTokensOverLimit
        );
        check_limit!(
            r.current_minute.value().as_i64(),
            self.data.token_limits.tokens_per_minute,
            MinuteTokensOverLimit
        );

        Ok(())
    }
}
// endregion: --- Project Node
// endregion: --- Main Model

// region:    --- Traits
pub(crate) trait NodeLimitsChecker {
    fn validate_limits(&self) -> Result<(), UsageExceededError>;
}
// endregion: --- Traits

// region:    --- Data Access
impl DataAccess {
    #[instrument(
        level = "trace",
        name = "get.graph",
        skip(self, api_key, application_secret, metrics)
    )]
    pub(crate) async fn get_graph(
        &self,
        api_key: &str,
        model_name: &str,
        skip_local_cache: bool,
        local_cache_ttl_ms: u32,
        application_secret: &Uuid,
        ts: &DateTime<Utc>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<Graph, GraphLoadError> {
        // Step 1: Get Graph Data
        let graph_data = self
            .get_graph_data(
                api_key,
                model_name,
                skip_local_cache,
                &ts,
                local_cache_ttl_ms,
                application_secret,
                metrics,
            )
            .await?;

        let stats_span = trace_span!("get.graph.usage");
        let graph = async move {
            // Step 2: Load Usage Stats from External Cache or Fallback to Defaults TODO: Fallback to data from DB
            let stats_keys = graph_data.generate_all_usage_stats_keys(&ts);

            // Step 3: Retrieve usage stats from External Cache or Fallback to Defaults TODO: Fallback to data from DB
            let stats_map = if let Some(external_cache) = &self.cache.external {
                match external_cache
                    .get_values(&stats_keys.into_iter().collect())
                    .await
                {
                    Ok(map) => map,
                    Err(e) => {
                        error!("Failed to get values from cache: {:?}", e);
                        todo!("Handle external cache retrieval error")
                    } // Maybe fallback to local data?
                }
            } else {
                todo!("Handle case where no external cache is configured"); // Maybe store it locally?
            };

            // Step 4: Load cached data into Graph structure
            // Part 1: Virtual Key Node
            let virtual_key_stats = self
                .load_virtual_key_usage_and_set_cache(&graph_data.virtual_key.id, &stats_map, &ts)
                .await?;
            let virtual_key_node = VirtualKeyNode {
                data: graph_data.virtual_key.clone(),
                usage_stats: virtual_key_stats,
            };

            // Part 2: Deployment Node
            let deployment_stats = self
                .load_deployment_usage_and_set_cache(&graph_data.deployment.id, &stats_map, &ts)
                .await?;
            let deployment_node = DeploymentNode {
                data: graph_data.deployment.clone(),
                association_id: graph_data.virtual_key_deployment.id.clone(),
                usage_stats: deployment_stats,
            };

            // Part 3: Project Node
            let project_stats = self
                .load_project_usage_and_set_cache(&graph_data.project.id, &stats_map, &ts)
                .await?;
            let project_node = ProjectNode {
                data: graph_data.project.clone(),
                usage_stats: project_stats,
            };

            // Part 4: Connection Nodes
            let connection_nodes = try_join_all(graph_data.connections.iter().map(|conn| async {
                let conn_stats = self
                    .load_connection_usage_and_set_cache(&conn.id, &stats_map, &ts)
                    .await?;
                let (association_id, weight) = graph_data
                    .connection_deployments
                    .iter()
                    .find(|cd| &cd.connection_id == &conn.id)
                    .map(|cd| (cd.id.clone(), cd.weight))
                    .unwrap();
                Ok::<_, DataAccessError>(ConnectionNode {
                    data: conn.clone(),
                    association_id,
                    weight,
                    usage_stats: conn_stats,
                })
            }))
            .await?;

            Ok::<Graph, DataAccessError>(Graph {
                virtual_key: virtual_key_node,
                deployment: deployment_node,
                project: project_node,
                connections: connection_nodes,
            })
        }
        .instrument(stats_span)
        .await?;

        Ok(graph)
    }

    #[instrument(
        level = "trace",
        name = "get.graph.data",
        skip(self, api_key, application_secret, metrics)
    )]
    async fn get_graph_data(
        &self,
        api_key: &str,
        model_name: &str,
        skip_local_cache: bool,
        now_utc: &DateTime<Utc>,
        local_cache_ttl_ms: u32,
        application_secret: &Uuid,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<GraphData, GraphLoadError> {
        let id = GraphDataId::new(model_name, api_key);

        if !skip_local_cache {
            if let Some(cached) = self.cache.get_local_graph(&id) {
                if !cached.is_expired(now_utc, local_cache_ttl_ms) {
                    return Ok(cached.value);
                }
            }
        }

        let graph_data = self
            .get_graph_data_from_db(id, application_secret, metrics)
            .await?;

        self.cache.set_local_graph(graph_data.clone());

        Ok(graph_data)
    }

    async fn get_graph_data_from_db(
        &self,
        id: GraphDataId,
        application_secret: &Uuid,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<GraphData, GraphLoadError> {
        println!("Loading Graph");
        let virtual_key = self
            .get_virtual_key(&id.virtual_key_id, application_secret, metrics)
            .await?
            .ok_or(GraphLoadError::InvalidVirtualKey)?;
        println!("Loaded virtual key");

        let deployment: Deployment = self
            .search_deployments(&Some(id.model_name.to_string()), metrics)
            .await?
            .first()
            .ok_or(GraphLoadError::InvalidDeploymentName)?
            .clone();
        println!("Loaded deployment");

        let project = self
            .get_project(&virtual_key.project_id, metrics)
            .await?
            .ok_or(GraphLoadError::InconsistentGraphDataError(
                InconsistentGraphDataError::InvalidProject,
            ))?;
        println!("Loaded project");

        let virtual_key_deployment = self
            .search_virtual_key_deployments(
                &Some(id.virtual_key_id.clone()),
                &Some(deployment.id.clone()),
                metrics,
            )
            .await?
            .first()
            .ok_or(GraphLoadError::InvalidVirtualKeyDeployment)?
            .clone();
        println!("Loaded virtual key deployment");

        let (connection_deployments, connections) = if deployment.connections.is_empty() {
            (Vec::new(), Vec::new())
        } else {
            // Load connection deployments - If any None values are found it is an inconsistency and should error out
            let connection_deployments = self
                .get_connection_deployments(&deployment.connections, metrics)
                .await?
                .into_values()
                .collect::<Option<Vec<ConnectionDeployment>>>()
                .ok_or(GraphLoadError::InconsistentGraphDataError(
                    InconsistentGraphDataError::InvalidConnectionDeployments,
                ))?;
            println!("Loaded connection deployments");

            // Load connections - If any None values are found it is an inconsistency and should error out
            let connections = self
                .get_connections(
                    &connection_deployments
                        .iter()
                        .map(|cd| cd.connection_id)
                        .collect(),
                    application_secret,
                    metrics,
                )
                .await?
                .into_values()
                .collect::<Option<Vec<Connection>>>()
                .ok_or(GraphLoadError::InconsistentGraphDataError(
                    InconsistentGraphDataError::InvalidConnection,
                ))?;
            println!("Loaded connections");

            (connection_deployments, connections)
        };

        Ok(GraphData {
            id,
            virtual_key,
            deployment,
            project,
            virtual_key_deployment,
            connection_deployments,
            connections,
        })
    }
}
// endregion: --- Data Access
