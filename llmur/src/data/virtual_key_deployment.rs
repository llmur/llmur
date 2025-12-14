use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use uuid::Uuid;
use crate::data::utils::{new_uuid_v5_from_string, ConvertInto};
use crate::{default_access_fns, default_database_access_fns, impl_local_store_accessors, impl_locally_stored, impl_structured_id_utils, impl_with_id_parameter_for_struct};
use crate::data::DataAccess;
use crate::data::deployment::{Deployment, DeploymentId};
use crate::data::errors::{DataConversionError, DatabaseError};
use crate::data::virtual_key::VirtualKeyId;
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
pub struct VirtualKeyDeploymentId(pub Uuid);
#[derive(Clone, Debug, Serialize)]
pub struct VirtualKeyDeployment {
    pub id: VirtualKeyDeploymentId,
    pub virtual_key_id: VirtualKeyId,
    pub deployment_id: DeploymentId,
}

impl VirtualKeyDeployment {
    pub fn new(id: VirtualKeyDeploymentId, virtual_key_id: VirtualKeyId, deployment_id: DeploymentId) -> Self {
        VirtualKeyDeployment {
            id,
            virtual_key_id,
            deployment_id,
        }
    }
}

impl_structured_id_utils!(VirtualKeyDeploymentId);
impl_with_id_parameter_for_struct!(VirtualKeyDeployment, VirtualKeyDeploymentId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {

    #[tracing::instrument(
        level="trace",
        name = "get.virtual_key_deployment",
        skip(self, id),
        fields(
            id = %id.0
        )
    )]
    pub async fn get_virtual_key_deployment(&self, id: &VirtualKeyDeploymentId) -> Result<Option<VirtualKeyDeployment>, DataAccessError> {
        self.__get_virtual_key_deployment(id, &None).await
    }

    #[tracing::instrument(
        level="trace",
        name = "get.virtual_key_deployments",
        skip(self, ids),
        fields(
            ids = ?ids.iter().map(|id| id.0).collect::<Vec<Uuid>>()
        )
    )]
    pub async fn get_virtual_key_deployments(&self, ids: &BTreeSet<VirtualKeyDeploymentId>) -> Result<BTreeMap<VirtualKeyDeploymentId, Option<VirtualKeyDeployment>>, DataAccessError> {
        self.__get_virtual_key_deployments(ids, &None).await
    }

    #[tracing::instrument(
        level="trace",
        name = "create.virtual_key_deployments",
        skip(self),
        fields(
            virtual_key_id = %virtual_key_id.0, 
            deployment_id = %deployment_id.0
        )
    )]
    pub async fn create_virtual_key_deployment(&self, virtual_key_id: &VirtualKeyId, deployment_id: &DeploymentId) -> Result<VirtualKeyDeployment, DataAccessError> {
        //self.cache.delete_cached_virtual_key(virtual_key_id).await;
        self.__create_virtual_key_deployment(virtual_key_id, deployment_id, &None).await
    }
    
    #[tracing::instrument(
        level="trace",
        name = "delete.virtual_key_deployment",
        skip(self, id),
        fields(
            id = %id.0
        )
    )]
    pub async fn delete_virtual_key_deployment(&self, id: &VirtualKeyDeploymentId) -> Result<u64, DataAccessError> {
        self.__delete_virtual_key_deployment(id).await
    }

    #[tracing::instrument(
        level="trace",
        name = "search.virtual_key_deployment",
        skip(self, virtual_key_id, deployment_id),
        fields(
            virtual_key_id = %virtual_key_id.map(|id| id.0.to_string()).unwrap_or("*".to_string()),
            deployment_id = %deployment_id.map(|id| id.0.to_string()).unwrap_or("*".to_string()),
        )
    )]
    pub async fn search_virtual_key_deployments(&self, virtual_key_id: &Option<VirtualKeyId>, deployment_id: &Option<DeploymentId>) -> Result<Vec<VirtualKeyDeployment>, DataAccessError> {
        self.__search_virtual_key_deployments(virtual_key_id, deployment_id, &None).await
    }

}

default_access_fns!(
        VirtualKeyDeployment,
        VirtualKeyDeploymentId,
        virtual_key_deployment,
        virtual_key_deployments,
        create => {
            virtual_key_id: &VirtualKeyId,
            deployment_id: &DeploymentId
        },
        search => {
            virtual_key_id: &Option<VirtualKeyId>,
            deployment_id: &Option<DeploymentId>
        }
    );
// endregion: --- Data Access

// region:    --- Database Access
default_database_access_fns!(
    DbVirtualKeyDeploymentRecord,
    VirtualKeyDeploymentId,
    virtual_key_deployment,
    virtual_key_deployments,
    insert => {
        virtual_key_id: &VirtualKeyId,
        deployment_id: &DeploymentId
    },
    search => {
        virtual_key_id: &Option<VirtualKeyId>,
        deployment_id: &Option<DeploymentId>
    }
);
// region:      --- Postgres Queries
pub(crate) fn pg_search<'a>(virtual_key_id: &'a Option<VirtualKeyId>, deployment_id: &'a Option<DeploymentId>) -> QueryBuilder<'a,  Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            id,
            virtual_key_id,
            deployment_id
        FROM
            virtual_keys_deployments_map
        WHERE true=true"
    );
    // If virtual_key_id is passed as a search parameter
    if let Some(virtual_key_id) = virtual_key_id {
        query.push(" AND virtual_key_id = ");
        query.push_bind(virtual_key_id);
    }

    // If deployment_id is passed as a search parameter
    if let Some(deployment_id) = deployment_id {
        query.push(" AND deployment_id = ");
        query.push_bind(deployment_id);
    }

    // Build query
    query
}
pub(crate) fn pg_get(id: &VirtualKeyDeploymentId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            id,
            virtual_key_id,
            deployment_id
        FROM
            virtual_keys_deployments_map
        WHERE
            id ="
    );
    // Push id
    query.push_bind(id);
    // Build query
    query
}

pub(crate) fn pg_getm(ids: &Vec<VirtualKeyDeploymentId>) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            id,
            virtual_key_id,
            deployment_id
        FROM
            virtual_keys_deployments_map
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

pub(crate) fn pg_delete(id: &VirtualKeyDeploymentId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        DELETE FROM virtual_keys_deployments_map
        WHERE id="
    );
    // Push id
    query.push_bind(id);
    // Return query
    query
}

pub(crate) fn pg_insert<'a>(virtual_key_id: &'a VirtualKeyId, deployment_id: &'a DeploymentId) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        INSERT INTO virtual_keys_deployments_map
            (id, virtual_key_id, deployment_id)
        VALUES
            (gen_random_uuid(), "
    );
    // Push name
    query.push_bind(virtual_key_id);
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
pub(crate) struct DbVirtualKeyDeploymentRecord {
    pub(crate) id: VirtualKeyDeploymentId,
    pub(crate) virtual_key_id: VirtualKeyId,
    pub(crate) deployment_id: DeploymentId,
}

impl ConvertInto<VirtualKeyDeployment> for DbVirtualKeyDeploymentRecord {
    fn convert(self, _application_secret: &Option<Uuid>) -> Result<VirtualKeyDeployment, DataConversionError> {
        Ok(
            VirtualKeyDeployment::new(
                self.id,
                self.virtual_key_id,
                self.deployment_id,
            )
        )
    }
}

impl_with_id_parameter_for_struct!(DbVirtualKeyDeploymentRecord, VirtualKeyDeploymentId);
// endregion: --- Database Model
