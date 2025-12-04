use crate::data::DataAccess;
use crate::data::connection_deployment::ConnectionDeploymentId;
use crate::data::errors::DataConversionError;
use crate::data::limits::{BudgetLimits, RequestLimits, TokenLimits};
use crate::data::utils::{ConvertInto, new_uuid_v5_from_string};
use crate::errors::DataAccessError;
use crate::{
    default_access_fns, default_database_access_fns, impl_local_store_accessors,
    impl_locally_stored, impl_structured_id_utils, impl_with_id_parameter_for_struct,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use std::collections::{BTreeMap, BTreeSet};
use sqlx::types::Json;
use uuid::Uuid;

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
    FromRow,
)]
#[sqlx(transparent)]
pub struct DeploymentId(pub Uuid);

#[derive(Clone, Debug, Serialize)]
pub struct Deployment {
    pub id: DeploymentId,
    pub name: String,
    pub access: DeploymentAccess,

    pub budget_limits: BudgetLimits,
    pub request_limits: RequestLimits,
    pub token_limits: TokenLimits,

    // Only track downstream dependencies
    pub connections: BTreeSet<ConnectionDeploymentId>,
}

impl Deployment {
    pub(crate) fn new(
        id: DeploymentId,
        name: String,
        access: DeploymentAccess,
        budget_limits: BudgetLimits,
        request_limits: RequestLimits,
        token_limits: TokenLimits,
        connections: BTreeSet<ConnectionDeploymentId>,
    ) -> Self {
        let concatenated_connections = connections
            .iter()
            .map(|foo| foo.to_string())
            .collect::<Vec<String>>()
            .join(":");

        Deployment {
            id,
            name,
            access,
            budget_limits,
            request_limits,
            token_limits,
            connections,
        }
    }
}

impl_structured_id_utils!(DeploymentId);
impl_with_id_parameter_for_struct!(Deployment, DeploymentId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {
    pub async fn get_deployment(
        &self,
        id: &DeploymentId,
    ) -> Result<Option<Deployment>, DataAccessError> {
        self.__get_deployment(id, &None).await
    }

    pub async fn get_deployments(
        &self,
        ids: &BTreeSet<DeploymentId>,
    ) -> Result<BTreeMap<DeploymentId, Option<Deployment>>, DataAccessError> {
        self.__get_deployments(ids, &None).await
    }

    pub async fn search_deployments(
        &self,
        name: &Option<String>,
    ) -> Result<Vec<Deployment>, DataAccessError> {
        self.__search_deployments(name, &None).await
    }

    pub async fn create_deployment(
        &self,
        name: &str,
        access: &DeploymentAccess,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>,
    ) -> Result<Deployment, DataAccessError> {
        self.__create_deployment(name, access, budget_limits, request_limits, token_limits, &None).await
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
        deployment_access: &DeploymentAccess,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>
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
        access: &DeploymentAccess,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>
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
            d.budget_limits,
            d.request_limits,
            d.token_limits,
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
            d.budget_limits,
            d.request_limits,
            d.token_limits,
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
            d.budget_limits,
            d.request_limits,
            d.token_limits,
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
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        DELETE FROM deployments
        WHERE id=",
    );
    // Push id
    query.push_bind(id);
    // Return query
    query
}

pub(crate) fn pg_insert<'a>(
    name: &'a str,
    access: &'a DeploymentAccess,
    budget_limits: &'a Option<BudgetLimits>,
    request_limits: &'a Option<RequestLimits>,
    token_limits: &'a Option<TokenLimits>,
) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        INSERT INTO deployments
            (id, name, access",
    );

    if budget_limits.is_some() {
        query.push(", budget_limits");
    }
    if request_limits.is_some() {
        query.push(", request_limits");
    }
    if token_limits.is_some() {
        query.push(", token_limits");
    }
    query.push(
        ")
        VALUES
            (gen_random_uuid(), ",
    );
    // Push name
    query.push_bind(name);
    query.push(", ");
    // Push access
    query.push_bind(access);

    if let Some(limits) = budget_limits {
        query.push(", ");
        query.push_bind(Json::from(limits));
    }

    if let Some(limits) = request_limits {
        query.push(", ");
        query.push_bind(Json::from(limits));
    }

    if let Some(limits) = token_limits {
        query.push(", ");
        query.push_bind(Json::from(limits));
    }

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

    pub(crate) budget_limits: Option<sqlx::types::Json<BudgetLimits>>,
    pub(crate) request_limits: Option<sqlx::types::Json<RequestLimits>>,
    pub(crate) token_limits: Option<sqlx::types::Json<TokenLimits>>,

    pub(crate) connections: Vec<ConnectionDeploymentId>,
}

impl ConvertInto<Deployment> for DbDeploymentRecord {
    fn convert(
        self,
        _application_secret: &Option<Uuid>,
    ) -> Result<Deployment, DataConversionError> {
        Ok(Deployment::new(
            self.id,
            self.name,
            self.access,
            self.budget_limits.map(|l| l.0).unwrap_or_default(),
            self.request_limits.map(|l| l.0).unwrap_or_default(),
            self.token_limits.map(|l| l.0).unwrap_or_default(),
            self.connections.into_iter().collect(),
        ))
    }
}

impl_with_id_parameter_for_struct!(DbDeploymentRecord, DeploymentId);
// endregion: --- Database Model
