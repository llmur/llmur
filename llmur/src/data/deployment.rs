use crate::data::connection_deployment::ConnectionDeploymentId;
use crate::data::errors::DataConversionError;
use crate::data::utils::{new_uuid_v5_from_string, ConvertInto};
use crate::{default_access_fns, default_database_access_fns, impl_local_store_accessors, impl_locally_stored, impl_structured_id_utils, impl_with_id_parameter_for_struct};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;
use crate::data::DataAccess;
use crate::errors::DataAccessError;

// region:    --- Main Model
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, sqlx::Type)]
#[sqlx(type_name = "deployment_access", rename_all = "lowercase")]
pub enum DeploymentAccess {
    #[sqlx(rename = "private")]
    #[serde(rename = "private")]
    Private,
    #[sqlx(rename = "public")]
    #[serde(rename = "public")]
    Public,
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
pub struct DeploymentId(pub Uuid);

#[derive(Clone, Debug, Serialize)]
pub struct Deployment {
    pub id: DeploymentId,
    pub name: String,
    pub access: DeploymentAccess,

    // Only track downstream dependencies
    pub connections: BTreeSet<ConnectionDeploymentId>,
}

impl Deployment {
    pub(crate) fn new(id: DeploymentId, name: String, access: DeploymentAccess, connections: BTreeSet<ConnectionDeploymentId>) -> Self {
        let concatenated_connections = connections
            .iter()
            .map(|foo| foo.to_string())
            .collect::<Vec<String>>()
            .join(":");

        Deployment {
            id,
            name,
            access,
            connections,
        }
    }
}

impl_structured_id_utils!(DeploymentId);
impl_with_id_parameter_for_struct!(Deployment, DeploymentId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {
    pub async fn get_deployment(&self, id: &DeploymentId) -> Result<Option<Deployment>, DataAccessError> {
        self.__get_deployment(id, &None).await
    }

    pub async fn get_deployments(&self, ids: &BTreeSet<DeploymentId>) -> Result<BTreeMap<DeploymentId, Option<Deployment>>, DataAccessError> {
        self.__get_deployments(ids, &None).await
    }

    pub async fn search_deployments(&self, name: &Option<String>) -> Result<Vec<Deployment>, DataAccessError> {
        self.__search_deployments(name, &None).await
    }

    pub async fn create_deployment(&self, name: &str, access: &DeploymentAccess) -> Result<Deployment, DataAccessError> {
        self.__create_deployment(name, access, &None).await
    }

    pub async fn delete_deployment(&self, id: &DeploymentId) -> Result<u64, DataAccessError> {
        self.__delete_deployment(id).await
    }
}


default_access_fns!(
        Deployment,
        DeploymentId,
        deployment,
        deployments,
        create => {
            name: &str,
            deployment_access: &DeploymentAccess
        },
        search => {
            name: &Option<String>
        }
    );
// endregion: --- Data Access

// region:    --- Database Access
default_database_access_fns!(
    DbDeploymentRecord,
    DeploymentId,
    deployment,
    deployments,
    insert => {
        name: &str,
        access: &DeploymentAccess
    },
    search => {
        name: &Option<String>
    }
);
// region:      --- Postgres Queries
pub(crate) fn pg_search(name: &Option<String>) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            d.id,
            d.name,
            d.access,
            COALESCE(array_agg(DISTINCT dc.id) FILTER (WHERE dc.id IS NOT NULL), '{}'::uuid[]) AS connections
        FROM
            deployments d
        LEFT JOIN deployments_connections_map dc ON dc.deployment_id = d.id
        WHERE true=true"
    );
    // If name is passed as a search parameter
    if let Some(name) = name {
        query.push(" AND d.name = ");
        query.push_bind(name);
    }

    query.push(" GROUP BY d.id, d.name, d.access");
    // Build query
    query
}

pub(crate) fn pg_get(id: &DeploymentId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            d.id,
            d.name,
            d.access,
            COALESCE(array_agg(DISTINCT dc.id) FILTER (WHERE dc.id IS NOT NULL), '{}'::uuid[]) AS connections
        FROM
            deployments d
        LEFT JOIN deployments_connections_map dc ON dc.deployment_id = d.id
        WHERE
            d.id ="
    );
    // Push id
    query.push_bind(id);
    // Group results
    query.push(" GROUP BY d.id, d.name, d.access");
    // Build query
    query
}

pub(crate) fn pg_getm(ids: &Vec<DeploymentId>) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            d.id,
            d.name,
            d.access,
            COALESCE(array_agg(DISTINCT dc.id) FILTER (WHERE dc.id IS NOT NULL), '{}'::uuid[]) AS connections
        FROM
            deployments d
        LEFT JOIN deployments_connections_map dc ON dc.deployment_id = d.id
        WHERE
            d.id IN ( "
    );
    // Push ids
    let mut separated = query.separated(", ");
    for id in ids.iter() {
        separated.push_bind(id);
    }
    separated.push_unseparated(") ");

    query.push(" GROUP BY d.id, d.name, d.access");

    query
}

pub(crate) fn pg_delete(id: &DeploymentId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        DELETE FROM deployments
        WHERE id="
    );
    // Push id
    query.push_bind(id);
    // Return query
    query
}

pub(crate) fn pg_insert<'a>(name: &'a str, access: &'a DeploymentAccess) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        INSERT INTO deployments
            (id, name, access)
        VALUES
            (gen_random_uuid(), "
    );
    // Push name
    query.push_bind(name);
    query.push(", ");
    // Push access
    query.push_bind(access);

    // Push the rest of the query
    query.push(") RETURNING id");
    // Return builder
    query
}
// endregion:   --- Postgres Queries
// endregion: --- Database Access

// region:    --- Database Model
#[derive(FromRow, Clone, Debug)]
pub(crate) struct DbDeploymentRecord {
    pub(crate) id: DeploymentId,
    pub(crate) name: String,
    pub(crate) access: DeploymentAccess,
    pub(crate) connections: Vec<ConnectionDeploymentId>,
}


impl ConvertInto<Deployment> for DbDeploymentRecord {
    fn convert(self, _application_secret: &Option<Uuid>) -> Result<Deployment, DataConversionError> {
        Ok(
            Deployment::new(
                self.id,
                self.name,
                self.access,
                self.connections.into_iter().collect(),
            )
        )
    }
}

impl_with_id_parameter_for_struct!(DbDeploymentRecord, DeploymentId);
// endregion: --- Database Model
