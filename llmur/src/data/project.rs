use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use uuid::Uuid;
use crate::data::utils::{new_uuid_v5_from_string, ConvertInto};
use crate::{default_access_fns, default_database_access_fns, impl_local_store_accessors, impl_locally_stored, impl_structured_id_utils, impl_with_id_parameter_for_struct};
use crate::data::{DataAccess, Database};
use crate::data::errors::{DataConversionError, DatabaseError};
use crate::data::membership::MembershipId;
use crate::data::user::UserId;
use crate::errors::DataAccessError;

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
    FromRow
)]
#[sqlx(transparent)]
pub struct ProjectId(pub Uuid);

#[derive(Clone, Debug, Serialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
}

impl Project {
    pub(crate) fn new(id: ProjectId, name: String) -> Self {
        Project {
            id,
            name,
        }
    }
}

impl_structured_id_utils!(ProjectId);
impl_with_id_parameter_for_struct!(Project, ProjectId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {
    pub async fn get_project(&self, id: &ProjectId) -> Result<Option<Project>, DataAccessError> {
        self.__get_project(id, &None).await
    }

    pub async fn create_project(&self, name: &str, owner: &Option<UserId>) -> Result<Project, DataAccessError> {
        let project_id = match owner {
            None => { self.database.insert_project(name).await? }
            Some(user) => { self.database.insert_project_with_owner(name, user).await?.0 }
        };

        let record = self.__get_project(&project_id, &None).await?.ok_or(DataAccessError::FailedToCreateKey)?; // TODO
        Ok(record)
    }

    pub async fn delete_project(&self, id: &ProjectId) -> Result<u64, DataAccessError> {
        self.__delete_project(id).await
    }
}

default_access_fns!(
        Project,
        ProjectId,
        project,
        projects,
        create => {
            name: &str
        },
        search => {}
    );
// endregion: --- Data Access

// region:    --- Database Access
impl Database {
    pub(crate) async fn insert_project_with_owner(&self, name: &str, owner: &UserId) -> Result<(ProjectId, MembershipId), DatabaseError> {
        match self {
            Database::Postgres { pool } => {
                let mut tx = pool.begin().await?;

                let mut insert_project_query = pg_insert(name);
                let sql = insert_project_query.build_query_as::<ProjectId>();
                let project_id = sql.fetch_one(&mut *tx).await?;

                let mut ins_creator_query = crate::data::membership::pg_insert(&project_id, &owner, &ProjectRole::Admin);
                let ins_creator_sql = ins_creator_query.build_query_as::<MembershipId>();
                let membership_id = ins_creator_sql.fetch_one(&mut *tx).await?;

                tx.commit().await?;
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
        name: &str
    },
    search => { }
);
// region:      --- Postgres Queries
pub(crate) fn pg_search() -> QueryBuilder<'static, Postgres> {
    todo!()
}

pub(crate) fn pg_get(id: &ProjectId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            id,
            name
        FROM projects
        WHERE id="
    );
    // Push id
    query.push_bind(id);

    query
}

pub(crate) fn pg_getm(ids: &Vec<ProjectId>) -> QueryBuilder<Postgres> {
    todo!()
}

pub(crate) fn pg_delete(id: &ProjectId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        DELETE FROM projects
        WHERE id="
    );
    // Push id
    query.push_bind(id);
    // Return query
    query
}

pub(crate) fn pg_insert(name: &str) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        INSERT INTO projects
            (id, name)
        VALUES
            (gen_random_uuid(), "
    );
    // Push name
    query.push_bind(name);
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
}

impl ConvertInto<Project> for DbProjectRecord {
    fn convert(self, _application_secret: &Option<Uuid>) -> Result<Project, DataConversionError> {
        Ok(Project::new(
            self.id,
            self.name,
        ))
    }
}

impl_with_id_parameter_for_struct!(DbProjectRecord, ProjectId);
// endregion: --- Database Model   
