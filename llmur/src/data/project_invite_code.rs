use std::sync::Arc;
use crate::data::project::{ProjectId, ProjectRole};
use crate::data::utils::{generate_random_alphanumeric_string, parse_and_add_to_current_ts, ConvertInto};
use crate::data::DataAccess;
use crate::errors::{DataAccessError, DbRecordConversionError, InvalidTimeFormatError};
use crate::{default_access_fns, default_database_access_fns, impl_structured_id_utils, impl_with_id_parameter_for_struct};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use uuid::Uuid;
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
pub struct ProjectInviteCodeId(pub Uuid);

#[derive(Clone, Debug)]
pub struct ProjectInviteCode {
    pub id: ProjectInviteCodeId,
    pub project_id: ProjectId,
    pub code: String,
    pub assign_role: ProjectRole,
    pub valid_until: Option<i64>,
}

impl ProjectInviteCode {
    pub(crate) fn new(id: ProjectInviteCodeId, project_id: ProjectId, code: String, assign_role: ProjectRole, valid_until: Option<i64>) -> Self {
        ProjectInviteCode {
            id,
            project_id,
            code,
            assign_role,
            valid_until,
        }
    }
}

impl_structured_id_utils!(ProjectInviteCodeId);
impl_with_id_parameter_for_struct!(ProjectInviteCode, ProjectInviteCodeId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {

    #[tracing::instrument(
        level="trace",
        name = "get.project_invite_code",
        skip(self, id, metrics),
        fields(
            id = %id.0
        )
    )]
    pub async fn get_invite_code(&self, id: &ProjectInviteCodeId, metrics: &Option<Arc<Metrics>>) -> Result<Option<ProjectInviteCode>, DataAccessError> {
        self.__get_project_invite_code(id, &None, metrics).await
    }
    
    #[tracing::instrument(
        level="trace",
        name = "create.project_invite_code",
        skip(self, project_id, validity, code_length, metrics),
        fields(
            project_id = %project_id.0,
            validity = %validity.clone().unwrap_or("Not set".to_string()),
            code_length = %code_length.unwrap_or(10)
        )
    )]
    pub async fn create_invite_code(&self, project_id: &ProjectId, assign_role: &ProjectRole, validity: &Option<String>, code_length: &Option<usize>, metrics: &Option<Arc<Metrics>>) -> Result<ProjectInviteCode, DataAccessError> {
        let code = &generate_random_alphanumeric_string(code_length.unwrap_or(10));

        let valid_until: Option<DateTime<Utc>> = if let Some(validity) = validity {
            let seconds = parse_and_add_to_current_ts(&validity)?;
            Some(
                chrono::DateTime::from_timestamp(seconds, 0).ok_or(InvalidTimeFormatError::TimestampOutOfRange(seconds))?
            )
        } else { None };

        self.__create_project_invite_code(project_id, &code, assign_role, &valid_until, &None, metrics).await
    }

    #[tracing::instrument(
        level="trace",
        name = "delete.project_invite_code",
        skip(self, id, metrics),
        fields(
            id = %id.0
        )
    )]
    pub async fn delete_invite_code(&self, id: &ProjectInviteCodeId, metrics: &Option<Arc<Metrics>>) -> Result<u64, DataAccessError> {
        self.__delete_project_invite_code(id, metrics).await
    }
}

default_access_fns!(
        ProjectInviteCode,
        ProjectInviteCodeId,
        project_invite_code,
        project_invite_codes,
        create => {
            project_id: &ProjectId,
            code: &str,
            assign_role: &ProjectRole,
            valid_until: &Option<DateTime<Utc>>
        },
        search => {}
    );
// endregion: --- Data Access

// region:    --- Database Access
default_database_access_fns!(
    DbProjectInviteCodeRecord,
    ProjectInviteCodeId,
    project_invite_code,
    project_invite_codes,
    insert => {
        project_id: &ProjectId,
        code: &str,
        assign_role: &ProjectRole,
        valid_until: &Option<DateTime<Utc>>
    },
    search => { }
);

// region:      --- Postgres Queries
#[allow(unused)]
pub(crate) fn pg_search() -> QueryBuilder<'static, Postgres> {
    unimplemented!()
}

pub(crate) fn pg_get(invite_id: &'_ ProjectInviteCodeId) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
    SELECT
        id,
        project_id,
        code,
        assign_role,
        valid_until,
        created_at
    FROM
        project_invites
    WHERE
        id = ");
    // Push code
    query.push_bind(invite_id);
    // Build query
    query
}

#[allow(unused)]
pub(crate) fn pg_getm(ids: &'_ Vec<ProjectInviteCodeId>) -> QueryBuilder<'_, Postgres> {
    unimplemented!()
}

pub(crate) fn pg_delete(invite_id: &'_ ProjectInviteCodeId) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        DELETE FROM project_invites
        WHERE id = "
    );
    // Push id
    query.push_bind(invite_id);
    // Build query
    query
}

pub(crate) fn pg_insert<'a>(project_id: &'a ProjectId, code: &'a str, assign_role: &'a ProjectRole, valid_until: &'a Option<DateTime<Utc>>) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        INSERT INTO project_invites
            (id, project_id, code, assign_role, valid_until)
        VALUES
            (gen_random_uuid(), "
    );

    // Push project id
    query.push_bind(project_id);
    query.push(",");
    // Push project code
    query.push_bind(code);
    query.push(",");
    // Push project role
    query.push_bind(assign_role);
    query.push(",");
    // Push project validity
    query.push_bind(valid_until);

    // Push the rest of the query
    query.push(") RETURNING id");
    // Return builder
    query
}
// endregion:   --- Postgres Queries
// endregion: --- Database Access

// region:    --- Database Model
#[derive(FromRow, Clone, Debug)]
pub(crate) struct DbProjectInviteCodeRecord {
    pub(crate) id: ProjectInviteCodeId,
    pub(crate) project_id: ProjectId,
    pub(crate) code: String,
    pub(crate) assign_role: ProjectRole,
    pub(crate) valid_until: Option<chrono::DateTime<chrono::Utc>>,
}

impl ConvertInto<ProjectInviteCode> for DbProjectInviteCodeRecord {
    fn convert(self, _application_secret: &Option<Uuid>) -> Result<ProjectInviteCode, DbRecordConversionError> {
        Ok(ProjectInviteCode::new(
            self.id,
            self.project_id,
            self.code,
            self.assign_role,
            self.valid_until.map(|t| t.timestamp()),
        ))
    }
}

impl_with_id_parameter_for_struct!(DbProjectInviteCodeRecord, ProjectInviteCodeId);
// endregion: --- Database Model   
