use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use crate::data::project::{ProjectId, ProjectRole};
use crate::data::user::UserId;
use crate::data::utils::ConvertInto;
use crate::{default_access_fns, default_database_access_fns, impl_structured_id_utils, impl_with_id_parameter_for_struct};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use uuid::Uuid;
use crate::data::DataAccess;
use crate::errors::{DataAccessError, DbRecordConversionError};
use crate::metrics::Metrics;
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
pub struct MembershipId(pub Uuid);

#[derive(Clone, Debug, Serialize)]
pub struct Membership {
    pub id: MembershipId,
    pub project_id: ProjectId,
    pub user_id: UserId,
    pub role: ProjectRole,
}

impl Membership {
    pub(crate) fn new(id: MembershipId, project_id: ProjectId, user_id: UserId, role: ProjectRole) -> Self {
        Membership {
            id,
            project_id,
            user_id,
            role,
        }
    }
}

impl_structured_id_utils!(MembershipId);
impl_with_id_parameter_for_struct!(Membership, MembershipId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {
    #[tracing::instrument(
        level="trace",
        name = "get.membership",
        skip(self, id, metrics),
        fields(
            id = %id.0
        )
    )]
    pub async fn get_membership(&self, id: &MembershipId, metrics: &Option<Arc<Metrics>>) -> Result<Option<Membership>, DataAccessError> {
        self.__get_membership(id, &None, metrics).await
    }

    #[tracing::instrument(
        level="trace",
        name = "get.membership",
        skip(self, ids, metrics),
        fields(
            ids = ?ids.iter().map(|id| id.0).collect::<Vec<Uuid>>()
        )
    )]
    pub async fn get_memberships(&self, ids: &BTreeSet<MembershipId>, metrics: &Option<Arc<Metrics>>) -> Result<BTreeMap<MembershipId, Option<Membership>>, DataAccessError> {
        self.__get_memberships(ids, &None, metrics).await
    }

    #[tracing::instrument(
        level="trace",
        name = "create.membership",
        skip(self, user_id, project_id, metrics),
        fields(
            user_id = %user_id.0,
            project_id = %project_id.0
        )
    )]
    pub async fn create_membership(&self, user_id: &UserId, project_id: &ProjectId, role: &ProjectRole, metrics: &Option<Arc<Metrics>>) -> Result<Membership, DataAccessError>{
        self.__create_membership(project_id, user_id, role, &None, metrics).await
    }


    #[tracing::instrument(
        level="trace",
        name = "delete.membership",
        skip(self, id, metrics),
        fields(
            id = %id.0
        )
    )]
    pub async fn delete_membership(&self, id: &MembershipId, metrics: &Option<Arc<Metrics>>) -> Result<u64, DataAccessError> {
        self.__delete_membership(id, metrics).await
    }
}

default_access_fns!(
        Membership,
        MembershipId,
        membership,
        memberships,
        create => {
            project_id: &ProjectId,
            user_id: &UserId,
            project_role: &ProjectRole
        },
        search => {}
    );
// endregion: --- Data Access

// region:    --- Database Access
default_database_access_fns!(
    DbMembershipRecord,
    MembershipId,
    membership,
    memberships,
    insert => {
        project_id: &ProjectId,
        user_id: &UserId,
        project_role: &ProjectRole
    },
    search => { }
);
// region:      --- Postgres Queries

#[allow(unused)]
pub(crate) fn pg_search() -> QueryBuilder<'static, Postgres> {
    unimplemented!()
}

pub(crate) fn pg_insert<'a>(project_id: &'a ProjectId, user_id: &'a UserId, project_role: &'a ProjectRole) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        INSERT INTO memberships (id, project_id, user_id, role)
          VALUES (gen_random_uuid(),"
    );
    query.push_bind(project_id);
    query.push(", ");

    query.push_bind(user_id);
    query.push(", ");

    query.push_bind(project_role);
    query.push(") RETURNING id");

    query
}

pub(crate) fn pg_delete(id: &'_ MembershipId) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        DELETE FROM memberships
        WHERE id="
    );
    // Push id
    query.push_bind(id);
    // Build query
    query
}

pub(crate) fn pg_get(id: &'_ MembershipId) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
          id,
          user_id,
          project_id,
          role
        FROM memberships
        WHERE id="
    );
    // Push id
    query.push_bind(id);
    // Build query
    query
}

pub(crate) fn pg_getm(ids: &'_ Vec<MembershipId>) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
          id,
          user_id,
          project_id,
          role
        FROM memberships
        WHERE id IN ( ",
    );
    // Push ids
    let mut separated = query.separated(", ");
    for id in ids.iter() {
        separated.push_bind(id);
    }
    separated.push_unseparated(") ");

    query
}

// endregion:   --- Postgres Queries
// endregion: --- Database Access

// region:    --- Database Model
#[derive(FromRow, Clone, Debug)]
pub(crate) struct DbMembershipRecord {
    pub(crate) id: MembershipId,
    pub(crate) project_id: ProjectId,
    pub(crate) user_id: UserId,
    pub(crate) role: ProjectRole,
}

impl ConvertInto<Membership> for DbMembershipRecord {
    fn convert(self, _application_secret: &Option<Uuid>) -> Result<Membership, DbRecordConversionError> {
        Ok(Membership::new(
            self.id,
            self.project_id,
            self.user_id,
            self.role,
        ))
    }
}

impl_with_id_parameter_for_struct!(DbMembershipRecord, MembershipId);
// endregion: --- Database Model
