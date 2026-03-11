use crate::data::DataAccess;
use crate::data::deployment::DeploymentId;
use crate::data::utils::ConvertInto;
use crate::data::virtual_batch::VirtualBatchId;
use crate::data::virtual_key::VirtualKeyId;
use crate::errors::{DataAccessError, DbRecordConversionError};
use crate::metrics::Metrics;
use crate::providers::openai::files::request::ListFilesOrder;
use crate::providers::openai::files::types::FilePurpose;
use crate::{
    default_access_fns, default_database_access_fns, impl_structured_id_utils,
    impl_with_id_parameter_for_struct,
};
use chrono::{DateTime, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, Postgres, QueryBuilder};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
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
pub struct VirtualFileId(pub Uuid);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VirtualFileKind {
    Input,
    Output,
    Error,
}

impl FromStr for VirtualFileKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "input" => Ok(Self::Input),
            "output" => Ok(Self::Output),
            "error" => Ok(Self::Error),
            _ => Err(format!("Invalid virtual file kind '{}'.", value)),
        }
    }
}

impl AsRef<str> for VirtualFileKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
            Self::Error => "error",
        }
    }
}

impl Display for VirtualFileKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderFileReference {
    pub provider: String,
    pub deployment_id: DeploymentId,
    pub provider_file_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VirtualFileLineMapEntry {
    pub line_index: i32,
    pub provider: String,
    pub deployment_id: DeploymentId,
    pub provider_file_id: String,
    pub provider_line_index: i32,
}

#[derive(Clone, Debug)]
pub struct VirtualFile {
    pub id: VirtualFileId,
    pub virtual_key_id: VirtualKeyId,
    pub purpose: FilePurpose,
    pub filename: String,
    pub bytes: i64,
    pub kind: VirtualFileKind,
    pub batch_id: Option<VirtualBatchId>,
    pub line_count: Option<i32>,
    pub provider_files: Option<Vec<ProviderFileReference>>,
    pub line_map: Option<Vec<VirtualFileLineMapEntry>>,
    pub metadata: Option<HashMap<String, String>>,
    pub created_at: i64,
    pub updated_at: i64,
    pub expires_at: Option<i64>,
}

impl VirtualFile {
    pub(crate) fn new(
        id: VirtualFileId,
        virtual_key_id: VirtualKeyId,
        purpose: FilePurpose,
        filename: String,
        bytes: i64,
        kind: VirtualFileKind,
        batch_id: Option<VirtualBatchId>,
        line_count: Option<i32>,
        provider_files: Option<Vec<ProviderFileReference>>,
        line_map: Option<Vec<VirtualFileLineMapEntry>>,
        metadata: Option<HashMap<String, String>>,
        created_at: i64,
        updated_at: i64,
        expires_at: Option<i64>,
    ) -> Self {
        VirtualFile {
            id,
            virtual_key_id,
            purpose,
            filename,
            bytes,
            kind,
            batch_id,
            line_count,
            provider_files,
            line_map,
            metadata,
            created_at,
            updated_at,
            expires_at,
        }
    }
}

impl_structured_id_utils!(VirtualFileId);
impl_with_id_parameter_for_struct!(VirtualFile, VirtualFileId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {
    #[tracing::instrument(
        level = "trace",
        name = "get.virtual_file",
        skip(self, id, metrics),
        fields(id = %id.0)
    )]
    pub async fn get_virtual_file(
        &self,
        id: &VirtualFileId,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<Option<VirtualFile>, DataAccessError> {
        self.__get_virtual_file(id, &None, metrics).await
    }

    #[tracing::instrument(
        level = "trace",
        name = "get.virtual_files",
        skip(self, ids, metrics),
        fields(ids = ?ids.iter().map(|id| id.0).collect::<Vec<Uuid>>())
    )]
    pub async fn get_virtual_files(
        &self,
        ids: &BTreeSet<VirtualFileId>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<BTreeMap<VirtualFileId, Option<VirtualFile>>, DataAccessError> {
        self.__get_virtual_files(ids, &None, metrics).await
    }

    #[tracing::instrument(
        level = "trace",
        name = "create.virtual_file",
        skip(
            self,
            id,
            virtual_key_id,
            purpose,
            filename,
            bytes,
            kind,
            batch_id,
            line_count,
            provider_files,
            line_map,
            metadata,
            expires_at,
            metrics
        ),
        fields(id = %id.0, virtual_key_id = %virtual_key_id.0)
    )]
    pub async fn create_virtual_file(
        &self,
        id: &VirtualFileId,
        virtual_key_id: &VirtualKeyId,
        purpose: &FilePurpose,
        filename: &str,
        bytes: i64,
        kind: &VirtualFileKind,
        batch_id: &Option<VirtualBatchId>,
        line_count: &Option<i32>,
        provider_files: &Option<Vec<ProviderFileReference>>,
        line_map: &Option<Vec<VirtualFileLineMapEntry>>,
        metadata: &Option<HashMap<String, String>>,
        expires_at: &Option<i64>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<VirtualFile, DataAccessError> {
        self.__create_virtual_file(
            id,
            virtual_key_id,
            purpose,
            filename,
            bytes,
            kind,
            batch_id,
            line_count,
            provider_files,
            line_map,
            metadata,
            expires_at,
            &None,
            metrics,
        )
        .await
    }

    #[tracing::instrument(
        level = "trace",
        name = "search.virtual_files",
        skip(self, virtual_key_id, purpose, limit, order, after, metrics),
        fields(virtual_key_id = %virtual_key_id.0)
    )]
    pub async fn search_virtual_files(
        &self,
        virtual_key_id: &VirtualKeyId,
        purpose: &Option<FilePurpose>,
        limit: &Option<i64>,
        order: &Option<ListFilesOrder>,
        after: &Option<VirtualFileId>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<Vec<VirtualFile>, DataAccessError> {
        self.__search_virtual_files(virtual_key_id, purpose, limit, order, after, &None, metrics)
            .await
    }

    #[tracing::instrument(
        level = "trace",
        name = "delete.virtual_file",
        skip(self, id, metrics),
        fields(id = %id.0)
    )]
    pub async fn delete_virtual_file(
        &self,
        id: &VirtualFileId,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<u64, DataAccessError> {
        self.__delete_virtual_file(id, metrics).await
    }
}

default_access_fns!(
    VirtualFile,
    VirtualFileId,
    virtual_file,
    virtual_files,
    create => {
        id: &VirtualFileId,
        virtual_key_id: &VirtualKeyId,
        purpose: &FilePurpose,
        filename: &str,
        bytes: i64,
        kind: &VirtualFileKind,
        batch_id: &Option<VirtualBatchId>,
        line_count: &Option<i32>,
        provider_files: &Option<Vec<ProviderFileReference>>,
        line_map: &Option<Vec<VirtualFileLineMapEntry>>,
        metadata: &Option<HashMap<String, String>>,
        expires_at: &Option<i64>
    },
    search => {
        virtual_key_id: &VirtualKeyId,
        purpose: &Option<FilePurpose>,
        limit: &Option<i64>,
        order: &Option<ListFilesOrder>,
        after: &Option<VirtualFileId>
    }
);
// endregion: --- Data Access

// region:    --- Database Access
// region:    --- Database Access

default_database_access_fns!(
    DbVirtualFileRecord,
    VirtualFileId,
    virtual_file,
    virtual_files,
    insert => {
        id: &VirtualFileId,
        virtual_key_id: &VirtualKeyId,
        purpose: &FilePurpose,
        filename: &str,
        bytes: i64,
        kind: &VirtualFileKind,
        batch_id: &Option<VirtualBatchId>,
        line_count: &Option<i32>,
        provider_files: &Option<Vec<ProviderFileReference>>,
        line_map: &Option<Vec<VirtualFileLineMapEntry>>,
        metadata: &Option<HashMap<String, String>>,
        expires_at: &Option<i64>
    },
    search => {
        virtual_key_id: &VirtualKeyId,
        purpose: &Option<FilePurpose>,
        limit: &Option<i64>,
        order: &Option<ListFilesOrder>,
        after: &Option<VirtualFileId>
    }
);

// region:      --- Postgres Queries
fn order_clause(order: &Option<ListFilesOrder>) -> &'static str {
    match order {
        Some(ListFilesOrder::Asc) => "ASC",
        Some(ListFilesOrder::Desc) | None => "DESC",
    }
}

pub(crate) fn pg_search<'a>(
    virtual_key_id: &'a VirtualKeyId,
    purpose: &'a Option<FilePurpose>,
    limit: &'a Option<i64>,
    order: &'a Option<ListFilesOrder>,
    after: &'a Option<VirtualFileId>,
) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
            id,
            virtual_key_id,
            purpose,
            filename,
            bytes,
            kind,
            batch_id,
            line_count,
            provider_files,
            line_map,
            metadata,
            expires_at,
            created_at,
            updated_at
        FROM
            virtual_files
        WHERE
            virtual_key_id = ",
    );

    query.push_bind(virtual_key_id);

    if let Some(purpose) = purpose {
        query.push(" AND purpose = ");
        query.push_bind(purpose.as_ref());
    }

    if let Some(after) = after {
        let order_op = match order {
            Some(ListFilesOrder::Asc) => ">",
            Some(ListFilesOrder::Desc) | None => "<",
        };
        query.push(" AND (created_at, id) ");
        query.push(order_op);
        query.push(" (SELECT created_at, id FROM virtual_files WHERE id = ");
        query.push_bind(after);
        query.push(")");
    }

    query.push(" ORDER BY created_at ");
    query.push(order_clause(order));
    query.push(", id ");
    query.push(order_clause(order));

    let limit_value = limit.unwrap_or(10_000);
    query.push(" LIMIT ");
    query.push_bind(limit_value);

    query
}

pub(crate) fn pg_get(id: &'_ VirtualFileId) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
            id,
            virtual_key_id,
            purpose,
            filename,
            bytes,
            kind,
            batch_id,
            line_count,
            provider_files,
            line_map,
            metadata,
            expires_at,
            created_at,
            updated_at
        FROM
            virtual_files
        WHERE
            id = ",
    );
    query.push_bind(id);
    query
}

pub(crate) fn pg_getm(ids: &'_ Vec<VirtualFileId>) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        SELECT
            id,
            virtual_key_id,
            purpose,
            filename,
            bytes,
            kind,
            batch_id,
            line_count,
            provider_files,
            line_map,
            metadata,
            expires_at,
            created_at,
            updated_at
        FROM
            virtual_files
        WHERE
            id IN (",
    );

    let mut separated = query.separated(", ");
    for id in ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");

    query
}

pub(crate) fn pg_delete(id: &'_ VirtualFileId) -> QueryBuilder<'_, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        DELETE FROM virtual_files
        WHERE id = ",
    );
    query.push_bind(id);
    query
}

pub(crate) fn pg_insert<'a>(
    id: &'a VirtualFileId,
    virtual_key_id: &'a VirtualKeyId,
    purpose: &'a FilePurpose,
    filename: &'a str,
    bytes: i64,
    kind: &'a VirtualFileKind,
    batch_id: &'a Option<VirtualBatchId>,
    line_count: &'a Option<i32>,
    provider_files: &'a Option<Vec<ProviderFileReference>>,
    line_map: &'a Option<Vec<VirtualFileLineMapEntry>>,
    metadata: &'a Option<HashMap<String, String>>,
    expires_at: &'a Option<i64>,
) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "
        INSERT INTO virtual_files
        (
            id,
            virtual_key_id,
            purpose,
            filename,
            bytes,
            kind,
            batch_id,
            line_count,
            provider_files,
            line_map,
            metadata,
            expires_at
        ) VALUES (",
    );

    query.push_bind(id);
    query.push(", ");
    query.push_bind(virtual_key_id);
    query.push(", ");
    query.push_bind(purpose.as_ref());
    query.push(", ");
    query.push_bind(filename);
    query.push(", ");
    query.push_bind(bytes);
    query.push(", ");
    query.push_bind(kind.as_ref());
    query.push(", ");
    query.push_bind(batch_id);
    query.push(", ");
    query.push_bind(line_count);
    query.push(", ");
    query.push_bind(provider_files.as_ref().map(Json::from));
    query.push(", ");
    query.push_bind(line_map.as_ref().map(Json::from));
    query.push(", ");
    query.push_bind(metadata.as_ref().map(Json::from));
    query.push(", ");
    let expires_at_value = expires_at
        .and_then(|ts| DateTime::from_timestamp(ts, 0))
        .map(|value| value.naive_utc());
    query.push_bind(expires_at_value);

    query.push(") RETURNING id");

    query
}
// endregion:   --- Postgres Queries
// endregion: --- Database Access

// region:    --- Database Model
#[derive(FromRow, Clone, Debug)]
pub(crate) struct DbVirtualFileRecord {
    pub(crate) id: VirtualFileId,
    pub(crate) virtual_key_id: VirtualKeyId,
    pub(crate) purpose: String,
    pub(crate) filename: String,
    pub(crate) bytes: i64,
    pub(crate) kind: String,
    pub(crate) batch_id: Option<VirtualBatchId>,
    pub(crate) line_count: Option<i32>,
    pub(crate) provider_files: Option<sqlx::types::Json<Vec<ProviderFileReference>>>,
    pub(crate) line_map: Option<sqlx::types::Json<Vec<VirtualFileLineMapEntry>>>,
    pub(crate) metadata: Option<sqlx::types::Json<HashMap<String, String>>>,
    pub(crate) expires_at: Option<NaiveDateTime>,
    pub(crate) created_at: NaiveDateTime,
    pub(crate) updated_at: NaiveDateTime,
}

impl ConvertInto<VirtualFile> for DbVirtualFileRecord {
    fn convert(
        self,
        _application_secret: &Option<Uuid>,
    ) -> Result<VirtualFile, DbRecordConversionError> {
        Ok(VirtualFile::new(
            self.id,
            self.virtual_key_id,
            self.purpose
                .parse::<FilePurpose>()
                .map_err(DbRecordConversionError::InternalError)?,
            self.filename,
            self.bytes,
            self.kind
                .parse::<VirtualFileKind>()
                .map_err(DbRecordConversionError::InternalError)?,
            self.batch_id,
            self.line_count,
            self.provider_files.map(|value| value.0),
            self.line_map.map(|value| value.0),
            self.metadata.map(|value| value.0),
            self.created_at.and_utc().timestamp(),
            self.updated_at.and_utc().timestamp(),
            self.expires_at.map(|value| value.and_utc().timestamp()),
        ))
    }
}

impl_with_id_parameter_for_struct!(DbVirtualFileRecord, VirtualFileId);
// endregion: --- Database Model
