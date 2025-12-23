use std::sync::Arc;
use crate::data::user::UserId;
use crate::data::utils::{new_uuid_v5_from_string, parse_and_add_to_current_ts, ConvertInto};
use crate::{default_access_fns, default_database_access_fns, impl_local_store_accessors, impl_locally_stored, impl_structured_id_utils, impl_with_id_parameter_for_struct};
use chrono::{DateTime, Utc};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use uuid::Uuid;
use crate::data::DataAccess;
use crate::errors::{DataAccessError, DbRecordConversionError, InvalidTimeFormatError};
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
pub struct SessionTokenId(pub Uuid);

#[derive(Clone, Debug)]
pub struct SessionToken {
    pub id: SessionTokenId,
    pub user_id: UserId,
    pub created_at: i64,
    pub updated_at: i64,
    pub expires_at: i64,
    pub revoked: bool,
}

impl SessionToken {
    pub(crate) fn new(id: SessionTokenId, user_id: UserId, created_at: i64, updated_at: i64, expires_at: i64, revoked: bool) -> Self {

        SessionToken {
            id,
            user_id,
            created_at,
            updated_at,
            expires_at,
            revoked
        }
    }
}

impl SessionToken {
    pub fn generate_id(token: &str, application_secret: &Uuid) -> Uuid {
        new_uuid_v5_from_string(&format!("{token}:{application_secret}"))
    }

    pub fn generate_random_token() -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }
}

impl_structured_id_utils!(SessionTokenId);
impl_with_id_parameter_for_struct!(SessionToken, SessionTokenId);
impl_locally_stored!(SessionToken, SessionTokenId, session_tokens);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {

    #[tracing::instrument(
        level="trace",
        name = "get.session_token",
        skip(self, id, metrics),
        fields(
            id = %id.0
        )
    )]
    pub async fn get_session_token(&self, id: &SessionTokenId, metrics: &Option<Arc<Metrics>>) -> Result<Option<SessionToken>, DataAccessError> {
        self.__get_session_token(id, &None, metrics).await // TODO : Parameterise
    }


    #[tracing::instrument(
        level="trace",
        name = "create.session_token",
        skip(self, id, user_id, metrics),
        fields(
            user_id = %user_id.0
        )
    )]
    pub async fn create_session_token(&self, id: &SessionTokenId, user_id: &UserId, metrics: &Option<Arc<Metrics>>) -> Result<SessionToken, DataAccessError> {
        let seconds = parse_and_add_to_current_ts("30d")?; // Should never fail - Still better handle it
        let expires_at = chrono::DateTime::from_timestamp(seconds, 0).ok_or(InvalidTimeFormatError::TimestampOutOfRange(seconds))?;

        self.__create_session_token(id, user_id, &expires_at, &None, metrics).await
    }


    #[tracing::instrument(
        level="trace",
        name = "delete.session_token",
        skip(self, id, metrics),
        fields(
            id = %id.0
        )
    )]
    pub async fn delete_session_token(&self, id: &SessionTokenId, metrics: &Option<Arc<Metrics>>) -> Result<u64, DataAccessError> {
        self.__delete_session_token(id, metrics).await
    }
}

default_access_fns!(
        SessionToken,
        SessionTokenId,
        session_token,
        session_tokens,
        create => {
            id: &SessionTokenId,
            user_id: &UserId,
            expires_at: &DateTime<Utc>
        },
        search => {}
    );
// endregion: --- Data Access

// region:    --- Database Access
default_database_access_fns!(
    DbSessionTokenRecord,
    SessionTokenId,
    session_token,
    session_tokens,
    insert => {
        id: &SessionTokenId,
        user_id: &UserId,
        expires_at: &DateTime<Utc>
    },
    search => { }
);
// region:      --- Postgres Queries

pub(crate) fn pg_search() -> QueryBuilder<'static, Postgres> {
    todo!()
}
pub(crate) fn pg_insert<'a>(id: &'a SessionTokenId, user_id: &'a UserId, expires_at: &'a DateTime<Utc>) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        INSERT INTO session_tokens
            (id, user_id, expires_at)
        VALUES ("
    );
    // Push id
    query.push_bind(id);
    query.push(", ");
    // Push user id
    query.push_bind(user_id);
    query.push(", ");
    // Push expiry date
    query.push_bind(expires_at);
    // Push the rest of the query
    query.push(") RETURNING id");
    // Return builder
    query
}
pub(crate) fn pg_get(id: &SessionTokenId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT id, user_id, expires_at, revoked, created_at, updated_at
        FROM session_tokens
        WHERE id="
    );
    // Push id
    query.push_bind(id);

    // Return builder
    query
}
pub(crate) fn pg_getm(ids: &Vec<SessionTokenId>) -> QueryBuilder<Postgres> {
    todo!()
}
pub(crate) fn pg_delete(id: &SessionTokenId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        DELETE FROM session_tokens
        WHERE id="
    );
    // Push id
    query.push_bind(id);
    // Return query
    query
}
// endregion:   --- Postgres Queries
// endregion: --- Database Access

// region:    --- Cache Access
impl_local_store_accessors!(SessionToken, SessionTokenId, session_token, session_tokens);
// endregion: --- Cache Access

// region:    --- Database Model
#[derive(FromRow, Clone, Debug)]
pub struct DbSessionTokenRecord {
    pub id: SessionTokenId,
    pub user_id: UserId,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub revoked: bool,
}

impl ConvertInto<SessionToken> for DbSessionTokenRecord {
    fn convert(self, _application_secret: &Option<Uuid>) -> Result<SessionToken, DbRecordConversionError> {
        Ok(SessionToken::new(
            self.id,
            self.user_id,
            self.created_at.timestamp(),
            self.updated_at.timestamp(),
            self.expires_at.timestamp(),
            self.revoked,
        ))
    }
}

impl_with_id_parameter_for_struct!(DbSessionTokenRecord, SessionTokenId);
// endregion: --- Database Model
