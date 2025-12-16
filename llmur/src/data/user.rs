use std::collections::BTreeSet;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use uuid::Uuid;
use crate::data::utils::{new_uuid_v5_from_string, ConvertInto};
use crate::{default_access_fns, default_database_access_fns, impl_local_store_accessors, impl_locally_stored, impl_structured_id_utils, impl_with_id_parameter_for_struct};
use crate::data::{DataAccess, Database};
use crate::data::membership::MembershipId;
use crate::data::password::hash_password;
use crate::errors::{DataAccessError, DbRecordConversionError};

// region:    --- Main Model
#[derive(Debug, Clone, sqlx::Type, PartialEq, Serialize, Deserialize)]
#[sqlx(type_name = "application_role", rename_all = "lowercase")]
pub enum ApplicationRole {
    #[serde(rename = "admin")]
    #[sqlx(rename = "admin")]
    Admin,
    #[sqlx(rename = "member")]
    #[serde(rename = "member")]
    Member,
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
pub struct UserId(pub Uuid);

#[derive(Clone, Debug, Serialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub name: String,
    pub hashed_password: String,
    pub salt: Uuid,
    pub email_verified: bool,
    pub blocked: bool,
    pub role: ApplicationRole,
    pub memberships: BTreeSet<MembershipId>,
}

impl User {
    pub(crate) fn new(id: UserId, email: String, name: String, hashed_password: String, salt: Uuid, email_verified: bool, blocked: bool, role: ApplicationRole, memberships: BTreeSet<MembershipId>) -> Self {
        User {
            id,
            email,
            name,
            hashed_password,
            salt,
            email_verified,
            blocked,
            role,
            memberships,
        }
    }
}

impl_structured_id_utils!(UserId);
impl_with_id_parameter_for_struct!(User, UserId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {

    #[tracing::instrument(
        level="trace",
        name = "get.user",
        skip(self, id),
        fields(
            id = %id.0
        )
    )]
    pub async fn get_user(&self, id: &UserId) -> Result<Option<User>, DataAccessError> {
        self.__get_user(id, &None).await
    }
   
    #[tracing::instrument(
        level="trace",
        name = "get.user",
        skip(self)
    )]
    pub async fn get_user_with_email(&self, email: &str) -> Result<Option<User>, DataAccessError> {
        let maybe_value: Option<User> = self.database.get_user_with_email(email).await?
            .map(|v| v.convert(&None)) // Returns Option<Result<X,Y>>
            .transpose()?;

        Ok(maybe_value)
    }

    #[tracing::instrument(
        level="trace",
        name = "create.user",
        skip(self, name, password, email_verified, blocked, application_secret)
    )]
    pub async fn create_user(&self, email: &str, name: &Option<String>, password: &str, email_verified: bool, blocked: bool, application_role: &ApplicationRole, application_secret: &Uuid) -> Result<User, DataAccessError>{
        let salt = Uuid::now_v7();
        let hashed_password = hash_password(password.to_string(), salt.clone(), application_secret.clone()).await?;
        let name: &str = name
            .as_deref() // Convert &Option<String> to Option<&str>
            .filter(|s| !s.is_empty()) // Only use non-empty names
            .unwrap_or_else(|| {
                // Fallback: split email at '@' and take the part before it
                email
                    .split('@')
                    .next()
                    .filter(|s| !s.is_empty()) // Ensure it's not empty
                    .unwrap_or("user") // Default to "user" if needed
            });

        self.__create_user(email, name, &hashed_password, email_verified, blocked, &salt, application_role, &None).await
    }

    #[tracing::instrument(
        level="trace",
        name = "delete.user",
        skip(self, id),
        fields(
            id = %id.0
        )
    )]
    pub async fn delete_user(&self, id: &UserId) -> Result<u64, DataAccessError> {
        self.__delete_user(id).await
    }
}

default_access_fns!(
        User,
        UserId,
        user,
        users,
        create => {
            email: &str,
            name: &str,
            hashed_password: &str,
            email_verified: bool,
            blocked: bool,
            salt: &Uuid,
            application_role: &ApplicationRole
        },
        search => { }
    );
// endregion: --- Data Access

// region:    --- Database Access
impl Database {
    #[tracing::instrument(
        level="trace",
        name = "db.get.user",
        skip(self)
    )]
    pub(crate) async fn get_user_with_email(&self, email: &str) -> Result<Option<DbUserRecord>, DataAccessError> {
        match self {
            Database::Postgres { pool } => {
                let mut query = pg_get_user_with_email_query(email);
                let sql= query.build_query_as::<DbUserRecord>();
                let result = sql.fetch_optional(pool).await?;

                Ok(result)
            }
        }
    }
}

default_database_access_fns!(
    DbUserRecord,
    UserId,
    user,
    users,
    insert => {
        email: &str,
        name: &str,
        hashed_password: &str,
        email_verified: bool,
        blocked: bool,
        salt: &Uuid,
        application_role: &ApplicationRole
    },
    search => { }
);

// region:      --- Postgres Queries
pub(crate) fn pg_search() -> QueryBuilder<'static, Postgres> {
    todo!()
}

pub(crate) fn pg_get(id: &UserId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            u.id,
            u.email,
            u.name,
            u.hashed_password,
            u.salt,
            u.email_verified,
            u.blocked,
            u.role,
            COALESCE(array_agg(DISTINCT m.project_id) FILTER (WHERE m.project_id IS NOT NULL), '{}'::uuid[]) AS memberships
        FROM users u
        LEFT JOIN memberships m ON u.id = m.user_id
        WHERE u.id="
    );
    // Push id
    query.push_bind(id);

    // Add group by clause in the end
    query.push(" GROUP BY u.id");

    // Build query
    query
}

pub(crate) fn pg_getm(ids: &Vec<UserId>) -> QueryBuilder<Postgres> {
    todo!()
}

pub(crate) fn pg_delete(id: &UserId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        DELETE FROM users
        WHERE id="
    );
    // Push id
    query.push_bind(id);
    // Build query
    query
}

pub(crate) fn pg_insert<'a>(
    email: &'a str,
    name: &'a str,
    hashed_password: &'a str,
    email_verified: bool,
    blocked: bool,
    salt: &'a Uuid,
    role: &'a ApplicationRole,
) -> QueryBuilder<'a, Postgres> {
    let mut query = QueryBuilder::<Postgres>::new("
        INSERT INTO users
            (id, email, name, salt, hashed_password, email_verified, blocked, role)
        VALUES (gen_random_uuid(), ");

    query.push_bind(email);
    query.push(", ");

    query.push_bind(name);
    query.push(", ");

    query.push_bind(salt);
    query.push(", ");

    query.push_bind(hashed_password);
    query.push(", ");

    query.push_bind(email_verified);
    query.push(", ");

    query.push_bind(blocked);
    query.push(", ");

    query.push_bind(role);
    query.push(") RETURNING id");

    query
}


fn pg_get_user_with_email_query(email: &str) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            u.id,
            u.email,
            u.name,
            u.hashed_password,
            u.salt,
            u.email_verified,
            u.blocked,
            u.role,
            COALESCE(array_agg(DISTINCT m.project_id) FILTER (WHERE m.project_id IS NOT NULL), '{}'::uuid[]) AS memberships
        FROM users u
        LEFT JOIN memberships m ON u.id = m.user_id
        WHERE u.email="
    );
    // Push id
    query.push_bind(email);

    // Add group by clause in the end
    query.push(" GROUP BY u.id");

    // Build query
    query
}

// endregion:   --- Postgres Queries
// endregion: --- Database Access

// region:    --- Database Model
#[derive(FromRow, Clone, Debug)]
pub(crate) struct DbUserRecord {
    pub(crate) id: UserId,
    pub(crate) email: String,
    pub(crate) name: String,

    pub(crate) salt: Uuid,
    pub(crate) hashed_password: String,

    pub(crate) email_verified: bool,
    pub(crate) blocked: bool,

    pub(crate) role: ApplicationRole,

    pub(crate) memberships: Vec<MembershipId>,
}

impl ConvertInto<User> for DbUserRecord {
    fn convert(self, _application_secret: &Option<Uuid>) -> Result<User, DbRecordConversionError> {
        Ok(User::new(
            self.id,
            self.email,
            self.name,
            self.hashed_password,
            self.salt,
            self.email_verified,
            self.blocked,
            self.role,
            self.memberships.into_iter().collect(),
        ))
    }
}

impl_with_id_parameter_for_struct!(DbUserRecord, UserId);
// endregion: --- Database Model   