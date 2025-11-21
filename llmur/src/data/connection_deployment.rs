use std::collections::{BTreeMap, BTreeSet};
use crate::data::connection::ConnectionId;
use crate::data::deployment::DeploymentId;
use crate::data::errors::DataConversionError;
use crate::data::utils::{new_uuid_v5_from_string, ConvertInto};
use crate::{default_access_fns, default_database_access_fns, impl_local_store_accessors, impl_locally_stored, impl_structured_id_utils, impl_with_id_parameter_for_struct};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use uuid::Uuid;
use crate::data::DataAccess;
use crate::data::virtual_key::VirtualKeyId;
use crate::data::virtual_key_deployment::VirtualKeyDeployment;
use crate::errors::DataAccessError;

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
pub struct ConnectionDeploymentId(pub Uuid);

#[derive(Clone, Debug, Serialize)]
pub struct ConnectionDeployment {
    pub id: ConnectionDeploymentId,
    pub connection_id: ConnectionId,
    pub deployment_id: DeploymentId
}

impl ConnectionDeployment {
    pub fn new(id: ConnectionDeploymentId, connection_id: ConnectionId, deployment_id: DeploymentId) -> Self {
        ConnectionDeployment {
            id,
            connection_id,
            deployment_id
        }
    }
}


impl_structured_id_utils!(ConnectionDeploymentId);
impl_with_id_parameter_for_struct!(ConnectionDeployment, ConnectionDeploymentId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {
    pub async fn get_connection_deployment(&self, id: &ConnectionDeploymentId) -> Result<Option<ConnectionDeployment>, DataAccessError> {
        self.__get_connection_deployment(id, &None).await
    }
    pub async fn get_connection_deployments(&self, ids: &BTreeSet<ConnectionDeploymentId>) -> Result<BTreeMap<ConnectionDeploymentId, Option<ConnectionDeployment>>, DataAccessError> {
        self.__get_connection_deployments(ids, &None).await
    }

    pub async fn create_connection_deployment(&self, connection_id: &ConnectionId, deployment_id: &DeploymentId) -> Result<ConnectionDeployment, DataAccessError> {
        //self.cache.delete_cached_deployment(deployment_id).await;
        self.__create_connection_deployment(connection_id, deployment_id, &None).await
    }

    pub async fn delete_connection_deployment(&self, id: &ConnectionDeploymentId) -> Result<u64, DataAccessError> {
        self.__delete_connection_deployment(id).await
    }

    pub async fn search_connection_deployments(&self, connection_id: &Option<ConnectionId>, deployment_id: &Option<DeploymentId>) -> Result<Vec<ConnectionDeployment>, DataAccessError> {
        self.__search_connection_deployments(connection_id, deployment_id, &None).await
    }
}

default_access_fns!(
        ConnectionDeployment,
        ConnectionDeploymentId,
        connection_deployment,
        connection_deployments,
        create => {
            connection_id: &ConnectionId,
            deployment_id: &DeploymentId
        },
        search => {
            connection_id: &Option<ConnectionId>,
            deployment_id: &Option<DeploymentId>
        }
    );
// endregion: --- Data Access

// region:    --- Database Access
default_database_access_fns!(
    DbConnectionDeploymentRecord,
    ConnectionDeploymentId,
    connection_deployment,
    connection_deployments,
    insert => {
        connection_id: &ConnectionId,
        deployment_id: &DeploymentId
    },
    search => {
        connection_id: &Option<ConnectionId>,
        deployment_id: &Option<DeploymentId>
    }
);
// region:      --- Postgres Queries
pub(crate) fn pg_search<'a>(connection_id: &'a Option<ConnectionId>, deployment_id: &'a Option<DeploymentId>) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            id,
            connection_id,
            deployment_id
        FROM
            deployments_connections_map
        WHERE true=true"
    );
    // If connection_id is passed as a search parameter
    if let Some(connection_id) = connection_id {
        query.push(" AND connection_id = ");
        query.push_bind(connection_id);
    }

    // If deployment_id is passed as a search parameter
    if let Some(deployment_id) = deployment_id {
        query.push(" AND deployment_id = ");
        query.push_bind(deployment_id);
    }

    // Build query
    query
}
pub(crate) fn pg_get(id: &ConnectionDeploymentId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            id,
            connection_id,
            deployment_id
        FROM
            deployments_connections_map
        WHERE
            id ="
    );
    // Push id
    query.push_bind(id);
    // Build query
    query
}


pub(crate) fn pg_getm(ids: &Vec<ConnectionDeploymentId>) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            id,
            connection_id,
            deployment_id
        FROM
            deployments_connections_map
        WHERE
            id IN ( "
    );
    // Push ids
    let mut separated = query.separated(", ");
    for id in ids.iter() {
        separated.push_bind(id);
    }
    separated.push_unseparated(") ");

    query
}

pub(crate) fn pg_delete(id: &ConnectionDeploymentId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        DELETE FROM deployments_connections_map
        WHERE id="
    );
    // Push id
    query.push_bind(id);
    // Return query
    query
}

pub(crate) fn pg_insert<'a>(connection_id: &'a ConnectionId, deployment_id: &'a DeploymentId) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        INSERT INTO deployments_connections_map
            (id, connection_id, deployment_id)
        VALUES
            (gen_random_uuid(), "
    );
    // Push name
    query.push_bind(connection_id);
    query.push(", ");
    // Push access
    query.push_bind(deployment_id);

    // Push the rest of the query
    query.push(") RETURNING id");
    // Return builder
    query
}

// endregion:   --- Postgres Queries
// endregion: --- Database Access

// region:    --- Database Model
#[derive(FromRow, Clone, Debug)]
pub(crate) struct DbConnectionDeploymentRecord {
    pub(crate) id: ConnectionDeploymentId,
    pub(crate) connection_id: ConnectionId,
    pub(crate) deployment_id: DeploymentId,
}

impl ConvertInto<ConnectionDeployment> for DbConnectionDeploymentRecord {
    fn convert(self, _application_secret: &Option<Uuid>) -> Result<ConnectionDeployment, DataConversionError> {
        Ok(
            ConnectionDeployment::new(
                self.id,
                self.connection_id,
                self.deployment_id,
            )
        )
    }
}

impl_with_id_parameter_for_struct!(DbConnectionDeploymentRecord, ConnectionDeploymentId);
// endregion: --- Database Model
