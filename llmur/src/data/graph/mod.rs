use crate::data::connection::Connection;
use crate::data::connection_deployment::{ConnectionDeployment, ConnectionDeploymentId};
use crate::data::deployment::{Deployment, DeploymentId};
use crate::data::graph::local_store::{GraphData, GraphDataId};
use crate::data::graph::usage_stats::{ConnectionUsageStats, DeploymentUsageStats, ProjectUsageStats, VirtualKeyUsageStats};
use crate::data::project::ProjectId;
use crate::data::virtual_key::VirtualKeyId;
use crate::data::virtual_key_deployment::VirtualKeyDeploymentId;
use crate::data::DataAccess;
use crate::errors::{DataAccessError, GraphLoadError, InconsistentGraphDataError};
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

pub(crate) mod usage_stats;
pub(crate) mod local_store;
pub(crate) mod external_cache;

// region:    --- Main Model
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Graph {
    pub(crate) virtual_key: VirtualKeyNode,
    pub(crate) deployment: DeploymentNode,
    pub(crate) project: ProjectNode,
    pub(crate) connections: Vec<ConnectionNode>,
}

// region:    --- Virtual Key Node
// Simplified version of VirtualKey for graph representation
#[derive(Clone, Debug, Serialize)]
pub(crate) struct VirtualKeyNode {
    pub(crate) data: VirtualKeyData,
    pub(crate) usage_stats: VirtualKeyUsageStats,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct VirtualKeyData {
    pub(crate) id: VirtualKeyId,
    pub(crate) alias: String,
    pub(crate) blocked: bool,
}
// endregion: --- Virtual Key Node

// region:    --- Deployment Node
// Simplified version of Deployment for graph representation
#[derive(Clone, Debug, Serialize)]
pub(crate)struct DeploymentNode {
    pub(crate) data: DeploymentData,
    pub(crate) association_id: VirtualKeyDeploymentId,
    pub(crate) usage_stats: DeploymentUsageStats,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DeploymentData {
    pub(crate) id: DeploymentId,
    pub(crate) name: String,
}
// endregion: --- Deployment Node

// region:    --- Connection Node
// Simplified version of Connection for graph representation
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ConnectionNode {
    pub(crate) data: Connection,
    pub(crate) association_id: ConnectionDeploymentId,
    pub(crate) usage_stats: ConnectionUsageStats,
}
// endregion: --- Connection Node

// region:    --- Project Node
// Simplified version of Project for graph representation
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ProjectNode {
    pub(crate) data: ProjectData,
    pub(crate) usage_stats: ProjectUsageStats,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ProjectData {
    pub(crate) id: ProjectId,
    pub(crate) name: String,
}
// endregion: --- Project Node
// endregion: --- Main Model

// region:    --- Data Access

impl Graph {}

impl DataAccess {
    pub async fn get_graph(&self, api_key: &str, model_name: &str, skip_local_cache: bool, local_cache_ttl_ms: u32, application_secret: &Uuid) -> Result<Graph, DataAccessError> {
        let now_utc: DateTime<Utc> = Utc::now();

        // Step 1: Get Graph Data
        let graph_data = self.get_graph_data(api_key, model_name, skip_local_cache, &now_utc, local_cache_ttl_ms, application_secret).await?;

        // Step 2: Load Usage Stats from External Cache or Fallback to Defaults TODO: Fallback to data from DB
        let stats_keys = graph_data.generate_all_usage_stats_keys(&now_utc);

        // Step 3: Retrieve usage stats from External Cache or Fallback to Defaults TODO: Fallback to data from DB
        let stats_map = if let Some(external_cache) = &self.cache.external {
            match external_cache.get_values(&stats_keys.into_iter().collect()).await {
                Ok(map) => { map }
                Err(_) => { todo!("Handle external cache retrieval error") } // Maybe fallback to local data?
            }
        } else {
            todo!("Handle case where no external cache is configured"); // Maybe store it locally?
        };

        // Step 4: Load cached data into Graph structure
        // Part 1: Virtual Key Node
        let virtual_key_stats = VirtualKeyUsageStats::extract_from_map(&graph_data.virtual_key.id, &now_utc, &stats_map);
        let virtual_key_node = VirtualKeyNode {
            data: VirtualKeyData {
                id: graph_data.virtual_key.id.clone(),
                alias: graph_data.virtual_key.alias.clone(),
                blocked: graph_data.virtual_key.blocked,
            },
            usage_stats: virtual_key_stats,
        };

        // Part 2: Deployment Node
        let deployment_stats = DeploymentUsageStats::extract_from_map(&graph_data.deployment.id, &now_utc, &stats_map);
        let deployment_node = DeploymentNode {
            data: DeploymentData {
                id: graph_data.deployment.id.clone(),
                name: graph_data.deployment.name.clone(),
            },
            association_id: graph_data.virtual_key_deployment.id.clone(),
            usage_stats: deployment_stats,
        };

        // Part 3: Project Node
        let project_stats = ProjectUsageStats::extract_from_map(&graph_data.project.id, &now_utc, &stats_map);
        let project_node = ProjectNode {
            data: ProjectData {
                id: graph_data.project.id.clone(),
                name: graph_data.project.name.clone(),
            },
            usage_stats: project_stats,
        };

        // Part 4: Connection Nodes
        let connection_nodes = graph_data.connections.iter().map(|conn| {
            let conn_stats = ConnectionUsageStats::extract_from_map(&conn.id, &now_utc, &stats_map);
            let association_id = graph_data.connection_deployments.iter()
                .find(|cd| &cd.connection_id == &conn.id)
                .map(|cd| cd.id.clone())
                .unwrap(); // Should always find a matching connection deployment
            ConnectionNode {
                data: conn.clone(),
                association_id,
                usage_stats: conn_stats,
            }
        }).collect();

        Ok(Graph {
            virtual_key: virtual_key_node,
            deployment: deployment_node,
            project: project_node,
            connections: connection_nodes,
        })
    }

    async fn get_graph_data(&self, api_key: &str, model_name: &str, skip_local_cache: bool, now_utc: &DateTime<Utc>, local_cache_ttl_ms: u32, application_secret: &Uuid) -> Result<GraphData, DataAccessError> {
        let id = GraphDataId::new(model_name, api_key);

        if !skip_local_cache {
            if let Some(cached) = self.cache.get_local_graph(&id) {
                if !cached.is_expired(now_utc, local_cache_ttl_ms) { return Ok(cached.value); }
            }
        }

        let graph_data = self.get_graph_data_from_db(id, application_secret).await?;

        self.cache.set_local_graph(graph_data.clone());

        Ok(graph_data)
    }

    async fn get_graph_data_from_db(&self, id: GraphDataId, application_secret: &Uuid) -> Result<GraphData, DataAccessError> {
        let virtual_key = self.get_virtual_key(&id.virtual_key_id, application_secret).await?
            .ok_or(GraphLoadError::InvalidVirtualKey)?;

        let deployment: Deployment = self.search_deployments(&Some(id.model_name.to_string())).await?
            .first()
            .ok_or(GraphLoadError::InvalidDeploymentName)?
            .clone();

        let project = self.get_project(&virtual_key.project_id).await?
            .ok_or(GraphLoadError::InconsistentGraphDataError(InconsistentGraphDataError::InvalidProject))?;

        let virtual_key_deployment = self.search_virtual_key_deployments(&Some(id.virtual_key_id.clone()), &Some(deployment.id.clone())).await?
            .first()
            .ok_or(GraphLoadError::InvalidVirtualKeyDeployment)?
            .clone();

        // Load connection deployments - If any None values are found it is an inconsistency and should error out
        let connection_deployments = self.get_connection_deployments(&deployment.connections).await?
            .into_values()
            .collect::<Option<Vec<ConnectionDeployment>>>()
            .ok_or(GraphLoadError::InconsistentGraphDataError(InconsistentGraphDataError::InvalidConnectionDeployments))?;

        // Load connections - If any None values are found it is an inconsistency and should error out
        let connections = self.get_connections(&connection_deployments.iter().map(|cd| cd.connection_id).collect(), application_secret).await?
            .into_values()
            .collect::<Option<Vec<Connection>>>()
            .ok_or(GraphLoadError::InconsistentGraphDataError(InconsistentGraphDataError::InvalidConnection))?;

        Ok(
            GraphData {
                id,
                virtual_key,
                deployment,
                project,
                virtual_key_deployment,
                connection_deployments,
                connections,
            }
        )
    }
}
// endregion: --- Data Access