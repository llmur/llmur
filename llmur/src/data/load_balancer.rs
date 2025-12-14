use crate::data::connection::ConnectionId;
use crate::data::deployment::DeploymentId;
use crate::data::graph::{ConnectionNode, Graph};
use crate::data::{DataAccess, LocallyStoredValue};
use crate::errors::DataAccessError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, sqlx::Type)]
#[sqlx(type_name = "load_balancing_strategy", rename_all = "lowercase")]
pub enum LoadBalancingStrategy {
    #[sqlx(rename = "round_robin")]
    #[serde(rename = "round_robin")]
    RoundRobin,
    #[sqlx(rename = "weighted_round_robin")]
    #[serde(rename = "weighted_round_robin")]
    WeightedRoundRobin,
    #[sqlx(rename = "least_connections")]
    #[serde(rename = "least_connections")]
    LeastConnections,
    #[sqlx(rename = "weighted_least_connections")]
    #[serde(rename = "weighted_least_connections")]
    WeightedLeastConnections,
}

impl DataAccess {
    #[tracing::instrument(
        level="trace",
        name = "get.next.connection",
        skip(self, graph),
        fields(
            deployment_id = %graph.deployment.data.id.0,
            strategy = ?graph.deployment.data.strategy
        )
    )]
    pub fn get_next_connection<'a>(&self, graph: &'a Graph) -> Result<&'a ConnectionNode, DataAccessError> {
        if graph.connections.is_empty() {
            return Err(DataAccessError::NoConnectionsAvailable);
        }

        let deployment_id = &graph.deployment.data.id;

        match graph.deployment.data.strategy {
            LoadBalancingStrategy::RoundRobin => {
                self.round_robin_select(deployment_id, &graph.connections)
            }
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.weighted_round_robin_select(deployment_id, &graph.connections)
            }
            LoadBalancingStrategy::LeastConnections => {
                self.least_connections_select(&graph.connections)
            }
            LoadBalancingStrategy::WeightedLeastConnections => {
                self.weighted_least_connections_select(&graph.connections)
            }
        }
    }

    fn round_robin_select<'a>(
        &self,
        deployment_id: &DeploymentId,
        connections: &'a [ConnectionNode],
    ) -> Result<&'a ConnectionNode, DataAccessError> {
        let mut index_map = self.cache.local.deployment_rr_index.lock().unwrap();

        let entry = index_map
            .entry(*deployment_id)
            .or_insert(LocallyStoredValue::new(0));

        let index = entry.value % connections.len();
        entry.value = (entry.value + 1) % connections.len();

        Ok(&connections[index])
    }

    fn weighted_round_robin_select<'a>(
        &self,
        deployment_id: &DeploymentId,
        connections: &'a [ConnectionNode],
    ) -> Result<&'a ConnectionNode, DataAccessError> {
        // Calculate total weight
        let total_weight: u64 = connections.iter().map(|c| c.weight as u64).sum();

        if total_weight == 0 {
            return self.round_robin_select(deployment_id, connections);
        }

        let mut index_map = self.cache.local.deployment_rr_index.lock().unwrap();

        let entry = index_map
            .entry(*deployment_id)
            .or_insert(LocallyStoredValue::new(0));

        // Get position in weighted sequence
        let position = entry.value % total_weight as usize;
        entry.value = (entry.value + 1) % total_weight as usize;

        // Find connection based on cumulative weight
        let mut cumulative_weight = 0u64;
        for connection in connections {
            cumulative_weight += connection.weight as u64;
            if position < cumulative_weight as usize {
                return Ok(connection);
            }
        }

        // Fallback (shouldn't reach here)
        Ok(&connections[0])
    }

    fn least_connections_select<'a>(
        &self,
        connections: &'a [ConnectionNode],
    ) -> Result<&'a ConnectionNode, DataAccessError> {
        let counter_map = self.cache.local.opened_connections_counter.lock().unwrap();

        connections
            .iter()
            .min_by_key(|conn| {
                counter_map
                    .get(&conn.data.id)
                    .map(|stored| stored.value)
                    .unwrap_or(0)
            })
            .ok_or(DataAccessError::NoConnectionsAvailable)
    }

    fn weighted_least_connections_select<'a>(
        &self,
        connections: &'a [ConnectionNode],
    ) -> Result<&'a ConnectionNode, DataAccessError> {
        let counter_map = self.cache.local.opened_connections_counter.lock().unwrap();

        connections
            .iter()
            .min_by(|a, b| {
                let a_count = counter_map
                    .get(&a.data.id)
                    .map(|s| s.value)
                    .unwrap_or(0) as f64;
                let b_count = counter_map
                    .get(&b.data.id)
                    .map(|s| s.value)
                    .unwrap_or(0) as f64;

                // Normalize by weight (higher weight means it can handle more)
                let a_weight = a.weight.max(1) as f64;
                let b_weight = b.weight.max(1) as f64;

                let a_ratio = a_count / a_weight;
                let b_ratio = b_count / b_weight;

                a_ratio.partial_cmp(&b_ratio).unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or(DataAccessError::NoConnectionsAvailable)
    }

    // Helper method to increment connection counter when a connection is opened
    #[tracing::instrument(
        level = "trace",
        name = "local.increment.opened.connection",
        skip(self, connection_id),
        fields(
            connection_id = %connection_id.0
        )
    )]
    pub fn increment_opened_connection_count(&self, connection_id: &ConnectionId) {
        let mut counter_map = self.cache.local.opened_connections_counter.lock().unwrap();
        counter_map
            .entry(connection_id.clone())
            .or_insert(LocallyStoredValue::new(0))
            .value += 1;
    }

    // Helper method to decrement connection counter when a connection is closed
    #[tracing::instrument(
        level = "trace",
        name = "local.decrement.opened.connection",
        skip(self, connection_id),
        fields(
            connection_id = %connection_id.0
        )
    )]
    pub fn decrement_opened_connection_count(&self, connection_id: &ConnectionId) {
        let mut counter_map = self.cache.local.opened_connections_counter.lock().unwrap();
        if let Some(stored) = counter_map.get_mut(connection_id) {
            stored.value = stored.value.saturating_sub(1);
        }
    }
}
