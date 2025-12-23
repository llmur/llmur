use std::sync::Arc;
use crate::data::limits::{BudgetLimits, RequestLimits, TokenLimits};
use crate::data::membership::MembershipId;
use crate::data::user::UserId;
use crate::data::utils::ConvertInto;
use crate::data::{DataAccess, Database};
use crate::errors::{DataAccessError, DbRecordConversionError};
use crate::{
    default_access_fns, default_database_access_fns, impl_structured_id_utils, impl_with_id_parameter_for_struct,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use sqlx::types::Json;
use uuid::Uuid;
use crate::metrics::Metrics;

// region:    --- Main Model
#[derive(Debug, Clone, sqlx::Type, PartialEq, Serialize, Deserialize)]
#[sqlx(type_name = "project_role", rename_all = "lowercase")]
pub enum ProjectRole {
    #[serde(rename = "admin")]
    #[sqlx(rename = "admin")]
    Admin,
    #[sqlx(rename = "developer")]
    #[serde(rename = "developer")]
    Developer,
    #[sqlx(rename = "guest")]
    #[serde(rename = "guest")]
    Guest,
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
pub struct ProjectId(pub Uuid);

#[derive(Clone, Debug, Serialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,

    pub budget_limits: BudgetLimits,
    pub request_limits: RequestLimits,
    pub token_limits: TokenLimits,
}

impl Project {
    pub(crate) fn new(
        id: ProjectId,
        name: String,
        budget_limits: BudgetLimits,
        request_limits: RequestLimits,
        token_limits: TokenLimits,
    ) -> Self {
        Project {
            id,
            name,
            budget_limits,
            request_limits,
            token_limits,
        }
    }
}

impl_structured_id_utils!(ProjectId);
impl_with_id_parameter_for_struct!(Project, ProjectId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {

    #[tracing::instrument(
        level="trace",
        name = "get.project",
        skip(self, id, metrics),
        fields(
            id = %id.0
        )
    )]
    pub async fn get_project(&self, id: &ProjectId, metrics: &Option<Arc<Metrics>>) -> Result<Option<Project>, DataAccessError> {
        self.__get_project(id, &None, metrics).await
    }

    #[tracing::instrument(
        level="trace",
        name = "create.project",
        skip(self, owner, metrics),
        fields(
            owner = %owner.map(|id| id.0.to_string()).unwrap_or("None".to_string()),
        )
    )]
    pub async fn create_project(
        &self,
        name: &str,
        owner: &Option<UserId>,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>,
        metrics: &Option<Arc<Metrics>>
    ) -> Result<Project, DataAccessError> {
        let project_id = match owner {
            None => {
                self.database
                    .insert_project(metrics, name, budget_limits, request_limits, token_limits)
                    .await?
            }
            Some(user) => self.database.insert_project_with_owner(name, user, budget_limits, request_limits, token_limits, metrics).await?.0,
        };

        let record = self
            .__get_project(&project_id, &None, metrics)
            .await
            .map_err(|e| crate::errors::DataAccessError::FailedToGetCreatedResource(Box::new(e), "project".to_string(), project_id.0))?
            .ok_or(crate::errors::DataAccessError::CreatedResourceNotFound("project".to_string(), project_id.0))?;

        Ok(record)
    }


    #[tracing::instrument(
        level="trace",
        name = "delete.project",
        skip(self, id, metrics),
        fields(
            id = %id.0
        )
    )]
    pub async fn delete_project(&self, id: &ProjectId, metrics: &Option<Arc<Metrics>>) -> Result<u64, DataAccessError> {
        self.__delete_project(id, metrics).await
    }
}

default_access_fns!(
    Project,
    ProjectId,
    project,
    projects,
    create => {
        name: &str,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>
    },
    search => {}
);
// endregion: --- Data Access

// region:    --- Database Access
impl Database {
    pub(crate) async fn insert_project_with_owner(
        &self,
        name: &str,
        owner: &UserId,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>,
        metrics: &Option<Arc<Metrics>>
    ) -> Result<(ProjectId, MembershipId), DataAccessError> {
        match self {
            Database::Postgres { pool } => {
                let mut tx = pool
                    .begin()
                    .await?;

                let mut insert_project_query = pg_insert(name, budget_limits, request_limits, token_limits);
                let sql = insert_project_query.build_query_as::<ProjectId>();
                let project_id = sql
                    .fetch_one(&mut *tx)
                    .await?;

                let mut ins_creator_query =
                    crate::data::membership::pg_insert(&project_id, &owner, &ProjectRole::Admin);
                let ins_creator_sql = ins_creator_query.build_query_as::<MembershipId>();
                let membership_id = ins_creator_sql
                    .fetch_one(&mut *tx)
                    .await?;

                tx.commit()
                    .await?;
                Ok((project_id, membership_id))
            }
        }
    }
}

default_database_access_fns!(
    DbProjectRecord,
    ProjectId,
    project,
    projects,
    insert => {
        name: &str,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>
    },
    search => { }
);
// region:      --- Postgres Queries
pub(crate) fn pg_search() -> QueryBuilder<'static, Postgres> {
    todo!()
}

pub(crate) fn pg_get(id: &ProjectId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
            id,
            name,
            budget_limits,
            request_limits,
            token_limits
        FROM projects
        WHERE id=",
    );
    // Push id
    query.push_bind(id);

    query
}

pub(crate) fn pg_getm(ids: &Vec<ProjectId>) -> QueryBuilder<Postgres> {
    todo!()
}

pub(crate) fn pg_delete(id: &ProjectId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        DELETE FROM projects
        WHERE id=",
    );
    // Push id
    query.push_bind(id);
    // Return query
    query
}

pub(crate) fn pg_insert<'a>(
    name: &'a str,
    budget_limits: &'a Option<BudgetLimits>,
    request_limits: &'a Option<RequestLimits>,
    token_limits: &'a Option<TokenLimits>,
) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'a, Postgres> = QueryBuilder::new(
        "
        INSERT INTO projects
            (id, name");

    if budget_limits.is_some() {
        query.push(", budget_limits");
    }
    if request_limits.is_some() {
        query.push(", request_limits");
    }
    if token_limits.is_some() {
        query.push(", token_limits");
    }

    query.push(")
        VALUES
            (gen_random_uuid(), ",
    );
    // Push name
    query.push_bind(name);

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
pub(crate) struct DbProjectRecord {
    pub(crate) id: ProjectId,
    pub(crate) name: String,

    pub(crate) budget_limits: Option<sqlx::types::Json<BudgetLimits>>,
    pub(crate) request_limits: Option<sqlx::types::Json<RequestLimits>>,
    pub(crate) token_limits: Option<sqlx::types::Json<TokenLimits>>,
}

impl ConvertInto<Project> for DbProjectRecord {
    fn convert(self, _application_secret: &Option<Uuid>) -> Result<Project, DbRecordConversionError> {
        Ok(Project::new(
            self.id,
            self.name,
            self.budget_limits.map(|l| l.0).unwrap_or_default(),
            self.request_limits.map(|l| l.0).unwrap_or_default(),
            self.token_limits.map(|l| l.0).unwrap_or_default(),
        ))
    }
}

impl_with_id_parameter_for_struct!(DbProjectRecord, ProjectId);
// endregion: --- Database Model
