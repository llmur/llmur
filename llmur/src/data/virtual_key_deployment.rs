use crate::data::deployment::DeploymentId;
use crate::data::limits::{BudgetLimits, RequestLimits, TokenLimits};
use crate::data::utils::ConvertInto;
use crate::data::virtual_key::VirtualKeyId;
use crate::data::{DataAccess, Database};
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
pub struct VirtualKeyDeploymentId(pub Uuid);
#[derive(Clone, Debug, Serialize)]
pub struct VirtualKeyDeployment {
    pub id: VirtualKeyDeploymentId,
    pub virtual_key_id: VirtualKeyId,
    pub deployment_id: DeploymentId,
    pub budget_limits: BudgetLimits,
    pub request_limits: RequestLimits,
    pub token_limits: TokenLimits,
}

impl VirtualKeyDeployment {
    pub fn new(
        id: VirtualKeyDeploymentId,
        virtual_key_id: VirtualKeyId,
        deployment_id: DeploymentId,
        budget_limits: BudgetLimits,
        request_limits: RequestLimits,
        token_limits: TokenLimits,
    ) -> Self {
        VirtualKeyDeployment {
            id,
            virtual_key_id,
            deployment_id,
            budget_limits,
            request_limits,
            token_limits,
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
        skip(self, id, metrics),
        fields(
            id = %id.0
        )
    )]
    pub async fn get_virtual_key_deployment(
        &self,
        id: &VirtualKeyDeploymentId,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<Option<VirtualKeyDeployment>, DataAccessError> {
        self.__get_virtual_key_deployment(id, &None, metrics).await
    }

    #[tracing::instrument(
        level="trace",
        name = "get.virtual_key_deployments",
        skip(self, ids, metrics),
        fields(
            ids = ?ids.iter().map(|id| id.0).collect::<Vec<Uuid>>()
        )
    )]
    pub async fn get_virtual_key_deployments(
        &self,
        ids: &BTreeSet<VirtualKeyDeploymentId>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<BTreeMap<VirtualKeyDeploymentId, Option<VirtualKeyDeployment>>, DataAccessError>
    {
        self.__get_virtual_key_deployments(ids, &None, metrics)
            .await
    }

    #[tracing::instrument(
        level="trace",
        name = "create.virtual_key_deployments",
        skip(self, metrics),
        fields(
            virtual_key_id = %virtual_key_id.0,
            deployment_id = %deployment_id.0
        )
    )]
    pub async fn create_virtual_key_deployment(
        &self,
        virtual_key_id: &VirtualKeyId,
        deployment_id: &DeploymentId,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<VirtualKeyDeployment, DataAccessError> {
        //self.cache.delete_cached_virtual_key(virtual_key_id).await;
        self.__create_virtual_key_deployment(
            virtual_key_id,
            deployment_id,
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
        name = "update.virtual_key_deployment",
        skip(self, id, budget_limits, request_limits, token_limits, metrics),
        fields(id = %id.0)
    )]
    pub async fn update_virtual_key_deployment(
        &self,
        id: &VirtualKeyDeploymentId,
        budget_limits: &Option<Option<BudgetLimits>>,
        request_limits: &Option<Option<RequestLimits>>,
        token_limits: &Option<Option<TokenLimits>>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<VirtualKeyDeployment, DataAccessError> {
        if budget_limits.is_none() && request_limits.is_none() && token_limits.is_none() {
            return self
                .get_virtual_key_deployment(id, metrics)
                .await?
                .ok_or(DataAccessError::ResourceNotFound);
        }

        let updated = self
            .database
            .update_virtual_key_deployment(id, budget_limits, request_limits, token_limits, metrics)
            .await?;

        if updated == 0 {
            return Err(DataAccessError::ResourceNotFound);
        }

        self.get_virtual_key_deployment(id, metrics)
            .await?
            .ok_or(DataAccessError::ResourceNotFound)
    }

    #[tracing::instrument(
        level="trace",
        name = "delete.virtual_key_deployment",
        skip(self, id, metrics),
        fields(
            id = %id.0
        )
    )]
    pub async fn delete_virtual_key_deployment(
        &self,
        id: &VirtualKeyDeploymentId,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<u64, DataAccessError> {
        self.__delete_virtual_key_deployment(id, metrics).await
    }

    #[tracing::instrument(
        level="trace",
        name = "search.virtual_key_deployment",
        skip(self, virtual_key_id, deployment_id, metrics),
        fields(
            virtual_key_id = %virtual_key_id.map(|id| id.0.to_string()).unwrap_or("*".to_string()),
            deployment_id = %deployment_id.map(|id| id.0.to_string()).unwrap_or("*".to_string()),
        )
    )]
    pub async fn search_virtual_key_deployments(
        &self,
        virtual_key_id: &Option<VirtualKeyId>,
        deployment_id: &Option<DeploymentId>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<Vec<VirtualKeyDeployment>, DataAccessError> {
        self.__search_virtual_key_deployments(virtual_key_id, deployment_id, &None, metrics)
            .await
    }
}

default_access_fns!(
    VirtualKeyDeployment,
    VirtualKeyDeploymentId,
    virtual_key_deployment,
    virtual_key_deployments,
    create => {
        virtual_key_id: &VirtualKeyId,
        deployment_id: &DeploymentId,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>
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
        deployment_id: &DeploymentId,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>
    },
    search => {
        virtual_key_id: &Option<VirtualKeyId>,
        deployment_id: &Option<DeploymentId>
    }
);
impl Database {
    #[tracing::instrument(
        level = "trace",
        name = "db.update.virtual_key_deployment",
        skip(self, id, budget_limits, request_limits, token_limits, metrics),
        fields(id = %id.0)
    )]
    pub(crate) async fn update_virtual_key_deployment(
        &self,
        id: &VirtualKeyDeploymentId,
        budget_limits: &Option<Option<BudgetLimits>>,
        request_limits: &Option<Option<RequestLimits>>,
        token_limits: &Option<Option<TokenLimits>>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<u64, DataAccessError> {
        use crate::metrics::RegisterDatabaseRequest;

        let operation = "db.update.virtual_key_deployment";
        let span = tracing::trace_span!("database_operation", operation= %operation);

        tracing::Instrument::instrument(
            async move {
                match self {
                    Database::Postgres { pool } => {
                        let start = std::time::Instant::now();
                        let Some(mut query) =
                            pg_update(id, budget_limits, request_limits, token_limits)
                        else {
                            return Ok(0);
                        };
                        let sql = query.build_query_as::<(VirtualKeyDeploymentId,)>();
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
pub(crate) fn pg_search<'a>(
    virtual_key_id: &'a Option<VirtualKeyId>,
    deployment_id: &'a Option<DeploymentId>,
) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
            id,
            virtual_key_id,
            deployment_id,
            budget_limits,
            request_limits,
            token_limits
        FROM
            virtual_keys_deployments_map
        WHERE true=true",
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
pub(crate) fn pg_get(id: &'_ VirtualKeyDeploymentId) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
            id,
            virtual_key_id,
            deployment_id,
            budget_limits,
            request_limits,
            token_limits
        FROM
            virtual_keys_deployments_map
        WHERE
            id =",
    );
    // Push id
    query.push_bind(id);
    // Build query
    query
}

pub(crate) fn pg_getm(ids: &'_ Vec<VirtualKeyDeploymentId>) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
            id,
            virtual_key_id,
            deployment_id,
            budget_limits,
            request_limits,
            token_limits
        FROM
            virtual_keys_deployments_map
        WHERE
            id IN ( ",
    );
    // Push ids
    let mut separated = query.separated(", ");
    for id in ids.iter() {
        separated.push_bind(id);
    }
    separated.push_unseparated(") ");

    query
}

pub(crate) fn pg_delete(id: &'_ VirtualKeyDeploymentId) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        DELETE FROM virtual_keys_deployments_map
        WHERE id=",
    );
    // Push id
    query.push_bind(id);
    // Return query
    query
}

pub(crate) fn pg_update<'a>(
    id: &'a VirtualKeyDeploymentId,
    budget_limits: &'a Option<Option<BudgetLimits>>,
    request_limits: &'a Option<Option<RequestLimits>>,
    token_limits: &'a Option<Option<TokenLimits>>,
) -> Option<QueryBuilder<'a, Postgres>> {
    let mut query: QueryBuilder<'_, Postgres> =
        QueryBuilder::new("UPDATE virtual_keys_deployments_map SET ");
    let mut has_updates = false;

    if let Some(limits) = budget_limits {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        match limits {
            Some(value) => {
                query.push("budget_limits = ");
                query.push_bind(Json::from(value));
            }
            None => {
                query.push("budget_limits = NULL");
            }
        }
    }

    if let Some(limits) = request_limits {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        match limits {
            Some(value) => {
                query.push("request_limits = ");
                query.push_bind(Json::from(value));
            }
            None => {
                query.push("request_limits = NULL");
            }
        }
    }

    if let Some(limits) = token_limits {
        if has_updates {
            query.push(", ");
        }
        has_updates = true;
        match limits {
            Some(value) => {
                query.push("token_limits = ");
                query.push_bind(Json::from(value));
            }
            None => {
                query.push("token_limits = NULL");
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

pub(crate) fn pg_insert<'a>(
    virtual_key_id: &'a VirtualKeyId,
    deployment_id: &'a DeploymentId,
    budget_limits: &'a Option<BudgetLimits>,
    request_limits: &'a Option<RequestLimits>,
    token_limits: &'a Option<TokenLimits>,
) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        INSERT INTO virtual_keys_deployments_map
            (id, virtual_key_id, deployment_id",
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
    query.push_bind(virtual_key_id);
    query.push(", ");
    // Push access
    query.push_bind(deployment_id);

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
pub(crate) struct DbVirtualKeyDeploymentRecord {
    pub(crate) id: VirtualKeyDeploymentId,
    pub(crate) virtual_key_id: VirtualKeyId,
    pub(crate) deployment_id: DeploymentId,
    pub(crate) budget_limits: Option<sqlx::types::Json<BudgetLimits>>,
    pub(crate) request_limits: Option<sqlx::types::Json<RequestLimits>>,
    pub(crate) token_limits: Option<sqlx::types::Json<TokenLimits>>,
}

impl ConvertInto<VirtualKeyDeployment> for DbVirtualKeyDeploymentRecord {
    fn convert(
        self,
        _application_secret: &Option<Uuid>,
    ) -> Result<VirtualKeyDeployment, DbRecordConversionError> {
        Ok(VirtualKeyDeployment::new(
            self.id,
            self.virtual_key_id,
            self.deployment_id,
            self.budget_limits.map(|l| l.0).unwrap_or_default(),
            self.request_limits.map(|l| l.0).unwrap_or_default(),
            self.token_limits.map(|l| l.0).unwrap_or_default(),
        ))
    }
}

impl_with_id_parameter_for_struct!(DbVirtualKeyDeploymentRecord, VirtualKeyDeploymentId);
// endregion: --- Database Model
