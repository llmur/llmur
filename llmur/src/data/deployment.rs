use crate::data::DataAccess;
use crate::data::connection::ConnectionId;
use crate::data::limits::{BudgetLimits, RequestLimits, TokenLimits};
use crate::data::utils::ConvertInto;
use crate::errors::{DataAccessError, DbRecordConversionError};
use crate::metrics::Metrics;
use crate::{
    default_access_fns, default_database_access_fns, impl_structured_id_utils,
    impl_with_id_parameter_for_struct,
};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, Postgres, QueryBuilder};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
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
    pub connection_id: ConnectionId,

    pub budget_limits: BudgetLimits,
    pub request_limits: RequestLimits,
    pub token_limits: TokenLimits,
}

impl Deployment {
    pub(crate) fn new(
        id: DeploymentId,
        name: String,
        access: DeploymentAccess,
        connection_id: ConnectionId,
        budget_limits: BudgetLimits,
        request_limits: RequestLimits,
        token_limits: TokenLimits,
    ) -> Self {
        Deployment {
            id,
            name,
            access,
            connection_id,
            budget_limits,
            request_limits,
            token_limits,
        }
    }
}

impl_structured_id_utils!(DeploymentId);
impl_with_id_parameter_for_struct!(Deployment, DeploymentId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {
    #[tracing::instrument(
        level="trace",
        name = "get.deployment",
        skip(self, id, metrics),
        fields(
            id = %id.0
        )
    )]
    pub async fn get_deployment(
        &self,
        id: &DeploymentId,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<Option<Deployment>, DataAccessError> {
        self.__get_deployment(id, &None, metrics).await
    }

    #[tracing::instrument(
        level="trace",
        name = "get.deployments",
        skip(self, ids, metrics),
        fields(
            ids = ?ids.iter().map(|id| id.0).collect::<Vec<Uuid>>()
        )
    )]
    pub async fn get_deployments(
        &self,
        ids: &BTreeSet<DeploymentId>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<BTreeMap<DeploymentId, Option<Deployment>>, DataAccessError> {
        self.__get_deployments(ids, &None, metrics).await
    }

    #[tracing::instrument(
        level="trace",
        name = "search.deployments",
        skip(self, name, metrics),
        fields(
            id = %name.clone().unwrap_or("*".to_string()),
        )
    )]
    pub async fn search_deployments(
        &self,
        name: &Option<String>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<Vec<Deployment>, DataAccessError> {
        self.__search_deployments(name, &None, metrics).await
    }

    #[tracing::instrument(level = "trace", name = "create.deployment", skip(self, metrics))]
    pub async fn create_deployment(
        &self,
        name: &str,
        access: &DeploymentAccess,
        connection_id: &ConnectionId,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<Deployment, DataAccessError> {
        self.__create_deployment(
            name,
            access,
            connection_id,
            budget_limits,
            request_limits,
            token_limits,
            &None,
            metrics,
        )
        .await
    }

    #[tracing::instrument(
        level="trace",
        name = "delete.deployment",
        skip(self, id, metrics),
        fields(
            id = %id.0
        )
    )]
    pub async fn delete_deployment(
        &self,
        id: &DeploymentId,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<u64, DataAccessError> {
        self.__delete_deployment(id, metrics).await
    }
}

default_access_fns!(
    Deployment,
    DeploymentId,
    deployment,
    deployments,
    create => {
        name: &str,
        access: &DeploymentAccess,
        connection_id: &ConnectionId,
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
        connection_id: &ConnectionId,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>
    },
    search => {
        name: &Option<String>
    }
);
// region:      --- Postgres Queries
pub(crate) fn pg_search(name: &'_ Option<String>) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            d.id,
            d.name,
            d.access,
            d.connection_id,
            d.budget_limits,
            d.request_limits,
            d.token_limits
        FROM
            deployments d
        WHERE true=true"
    );
    // If name is passed as a search parameter
    if let Some(name) = name {
        query.push(" AND d.name = ");
        query.push_bind(name);
    }

    // Build query
    query
}

pub(crate) fn pg_get(id: &'_ DeploymentId) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            d.id,
            d.name,
            d.access,
            d.connection_id,
            d.budget_limits,
            d.request_limits,
            d.token_limits
        FROM
            deployments d
        WHERE
            d.id ="
    );
    // Push id
    query.push_bind(id);
    // Build query
    query
}

pub(crate) fn pg_getm(ids: &'_ Vec<DeploymentId>) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            d.id,
            d.name,
            d.access,
            d.connection_id,
            d.budget_limits,
            d.request_limits,
            d.token_limits
        FROM
            deployments d
        WHERE
            d.id IN ( "
    );
    // Push ids
    let mut separated = query.separated(", ");
    for id in ids.iter() {
        separated.push_bind(id);
    }
    separated.push_unseparated(") ");

    query
}

pub(crate) fn pg_delete(id: &'_ DeploymentId) -> QueryBuilder<'_, Postgres> {
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
    connection_id: &'a ConnectionId,
    budget_limits: &'a Option<BudgetLimits>,
    request_limits: &'a Option<RequestLimits>,
    token_limits: &'a Option<TokenLimits>,
) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        INSERT INTO deployments
            (id, name, access, connection_id",
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
    query.push(", ");
    // Push connection id
    query.push_bind(connection_id);

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
    pub(crate) connection_id: ConnectionId,

    pub(crate) budget_limits: Option<sqlx::types::Json<BudgetLimits>>,
    pub(crate) request_limits: Option<sqlx::types::Json<RequestLimits>>,
    pub(crate) token_limits: Option<sqlx::types::Json<TokenLimits>>,
}

impl ConvertInto<Deployment> for DbDeploymentRecord {
    fn convert(
        self,
        _application_secret: &Option<Uuid>,
    ) -> Result<Deployment, DbRecordConversionError> {
        Ok(Deployment::new(
            self.id,
            self.name,
            self.access,
            self.connection_id,
            self.budget_limits.map(|l| l.0).unwrap_or_default(),
            self.request_limits.map(|l| l.0).unwrap_or_default(),
            self.token_limits.map(|l| l.0).unwrap_or_default(),
        ))
    }
}

impl_with_id_parameter_for_struct!(DbDeploymentRecord, DeploymentId);
// endregion: --- Database Model
