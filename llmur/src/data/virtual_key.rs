use std::collections::BTreeSet;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use sqlx::types::Json;
use uuid::Uuid;
use crate::data::utils::{decrypt, encrypt, generate_random_api_key, new_uuid_v5_from_string, ConvertInto};
use crate::{default_access_fns, default_database_access_fns, impl_local_store_accessors, impl_locally_stored, impl_structured_id_utils, impl_with_id_parameter_for_struct};
use crate::data::DataAccess;
use crate::data::errors::{DataConversionError, DatabaseError};
use crate::data::limits::{BudgetLimits, RequestLimits, TokenLimits};
use crate::data::project::ProjectId;
use crate::data::virtual_key_deployment::VirtualKeyDeploymentId;
use crate::errors::DataAccessError;

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
pub struct VirtualKeyId(pub Uuid);

impl VirtualKeyId {
    pub(crate) fn from_decrypted_key(key: &str) -> Self {
        new_uuid_v5_from_string(key).into()
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct VirtualKey {
    pub id: VirtualKeyId,
    pub key: String,
    pub alias: String,
    pub blocked: bool,
    pub project_id: ProjectId,

    pub budget_limits: BudgetLimits,
    pub request_limits: RequestLimits,
    pub token_limits: TokenLimits,
    
    pub deployments: BTreeSet<VirtualKeyDeploymentId>,
}

impl VirtualKey {
    pub(crate) fn new(
        id: VirtualKeyId,
        alias: String,
        description: Option<String>,
        salt: Uuid,
        encrypted_key: String,
        blocked: bool,
        project_id: ProjectId,

        budget_limits: BudgetLimits,
        request_limits: RequestLimits,
        token_limits: TokenLimits,
        
        deployments: BTreeSet<VirtualKeyDeploymentId>,
        application_secret: &Uuid
    ) -> Result<Self, DatabaseError> {
        let key = decrypt(&encrypted_key, &salt, application_secret)?;

        Ok(VirtualKey {
            id,
            key,
            alias,
            blocked,
            project_id,
            budget_limits,
            request_limits,
            token_limits,
            deployments
        })
    }
}

impl_structured_id_utils!(VirtualKeyId);
impl_with_id_parameter_for_struct!(VirtualKey, VirtualKeyId);
// endregion: --- Main Model

// region:    --- Data Access
impl DataAccess {
    pub async fn get_virtual_key(&self, id: &VirtualKeyId, application_secret: &Uuid) -> Result<Option<VirtualKey>, DataAccessError> {
        self.__get_virtual_key(id, &Some(application_secret.clone())).await
    }

    pub async fn create_virtual_key(
        &self,
        key_suffix_length: usize,
        alias: &Option<String>,
        description: &Option<String>,
        blocked: bool,
        project_id: &ProjectId,
        budget_limits: &Option<BudgetLimits>,
        request_limits: &Option<RequestLimits>,
        token_limits: &Option<TokenLimits>,
        application_secret: &Uuid
    ) -> Result<VirtualKey, DataAccessError> {
        let key = generate_random_api_key(key_suffix_length);
        let salt = Uuid::now_v7();
        let encrypted_key = encrypt(&key, &salt, application_secret).map_err(|_| DataAccessError::FailedToCreateKey)?;
        let id = VirtualKeyId::from_decrypted_key(&key);

        let alias = alias.clone().unwrap_or(format!("sk-...{}", key.char_indices().nth_back(4).unwrap().0));

        self.__create_virtual_key(
            &id,
            &alias,
            description,
            &salt,
            &encrypted_key,
            blocked,
            project_id,
            budget_limits,
            request_limits,
            token_limits,
            &Some(application_secret.clone()),
        ).await
    }

    pub async fn delete_virtual_key(&self, id: &VirtualKeyId) -> Result<u64, DataAccessError> {
        self.__delete_virtual_key(id).await
    }
}

default_access_fns!(
        VirtualKey,
        VirtualKeyId,
        virtual_key,
        virtual_keys,
        create => {
            id: &VirtualKeyId,
            alias: &str,
            description: &Option<String>,
            salt: &Uuid,
            encrypted_key: &str,
            blocked: bool,
            project_id: &ProjectId,
            budget_limits: &Option<BudgetLimits>,
            request_limits: &Option<RequestLimits>,
            token_limits: &Option<TokenLimits>
        },
        search => {}
    );
// endregion: --- Data Access

// region:    --- Database Access
default_database_access_fns!(
    DbVirtualKeyRecord,
    VirtualKeyId,
    virtual_key,
    virtual_keys,
    insert => {
        id: &VirtualKeyId,
        alias: &str,
        description: &Option<String>,
        salt: &Uuid, encrypted_key: &str,
        blocked: bool,
        project_id: &ProjectId,
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

pub(crate) fn pg_get(id: &VirtualKeyId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            vk.id,
            vk.alias,
            vk.description,

            vk.salt,
            vk.encrypted_key,
            vk.blocked,
            vk.project_id,

            vk.budget_limits,
            vk.request_limits,
            vk.token_limits,

            COALESCE(array_agg(DISTINCT vkd.id) FILTER (WHERE vkd.id IS NOT NULL), '{}'::uuid[]) AS deployments
        FROM
            virtual_keys vk
        LEFT JOIN virtual_keys_deployments_map vkd ON vkd.virtual_key_id = vk.id
        WHERE
            vk.id = "
    );
    // Push id
    query.push_bind(id);

    query.push(" GROUP BY vk.id, vk.alias, vk.description, vk.salt, vk.encrypted_key, vk.blocked, vk.project_id");
    // Build query
    query
}

pub(crate) fn pg_getm(ids: &Vec<VirtualKeyId>) -> QueryBuilder<Postgres> {

    /*
    -- Limits
    maximum_requests_per_minute INTEGER NULL,
    maximum_tokens_per_minute INTEGER NULL,
    maximum_budget INTEGER NULL,
    budget_rate budget_rate NOT NULL DEFAULT 'monthly',
    */
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        SELECT
            vk.id,
            vk.alias,
            vk.description,

            vk.salt,
            vk.encrypted_key,
            vk.blocked,
            vk.project_id,

            vk.budget_limits,
            vk.request_limits,
            vk.token_limits,

            COALESCE(array_agg(DISTINCT vkd.id) FILTER (WHERE vkd.id IS NOT NULL), '{}'::uuid[]) AS deployments
        FROM
            virtual_keys vk
        LEFT JOIN virtual_keys_deployments_map vkd ON vkd.virtual_key_id = vk.id
        WHERE
            vk.id IN ( "
    );
    // Push id
    query.push_bind(ids);

    query.push(" ) GROUP BY vk.id, vk.alias, vk.description, vk.salt, vk.encrypted_key, vk.blocked, vk.project_id");
    // Build query
    query
}

pub(crate) fn pg_delete(id: &VirtualKeyId) -> QueryBuilder<Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        DELETE FROM virtual_keys
        WHERE id="
    );
    // Push id
    query.push_bind(id);
    // Build query
    query
}

pub(crate) fn pg_insert<'a>(
    id: &'a VirtualKeyId,
    alias: &'a str,
    description: &'a Option<String>,
    salt: &'a Uuid,
    encrypted_key: &'a str,
    blocked: bool,
    project_id: &'a ProjectId,
    budget_limits: &'a Option<BudgetLimits>,
    request_limits: &'a Option<RequestLimits>,
    token_limits: &'a Option<TokenLimits>
) -> QueryBuilder<'a, Postgres> {
    let mut query: QueryBuilder<'_, Postgres> = QueryBuilder::new("
        INSERT INTO virtual_keys
        (
            id,
            alias,
            description,
            salt,
            encrypted_key,
            blocked,
            project_id");

    if budget_limits.is_some() {
        query.push(", budget_limits");
    }
    if request_limits.is_some() {
        query.push(", request_limits");
    }
    if token_limits.is_some() {
        query.push(", token_limits");
    }
    
    query.push(") VALUES (");
    // Push id
    query.push_bind(id);
    query.push(", ");
    // Push alias
    query.push_bind(alias);
    query.push(", ");
    // Push description
    query.push_bind(description);
    query.push(", ");
    // Push salt
    query.push_bind(salt);
    query.push(", ");
    // Push encrypted_key
    query.push_bind(encrypted_key);
    query.push(", ");
    // Push blocked
    query.push_bind(blocked);
    query.push(", ");
    // Push name
    query.push_bind(project_id);

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
pub(crate) struct DbVirtualKeyRecord {
    pub(crate) id: VirtualKeyId,
    pub(crate) alias: String,
    pub(crate) description: Option<String>,
    pub(crate) salt: Uuid,
    pub(crate) encrypted_key: String,
    pub(crate) blocked: bool,
    pub(crate) project_id: ProjectId,

    pub(crate) budget_limits: Option<sqlx::types::Json<BudgetLimits>>,
    pub(crate) request_limits: Option<sqlx::types::Json<RequestLimits>>,
    pub(crate) token_limits: Option<sqlx::types::Json<TokenLimits>>,

    pub(crate) deployments: Vec<VirtualKeyDeploymentId>,
}

impl ConvertInto<VirtualKey> for DbVirtualKeyRecord {
    fn convert(self, application_secret: &Option<Uuid>) -> Result<VirtualKey, DataConversionError> {
        let application_secret = application_secret.ok_or(DatabaseError::InvalidDatabaseRecord).map_err(|_| DataConversionError::DefaultError {cause: "Ups".to_string()})?; // TODO return Internal Server Error
        Ok(VirtualKey::new(
            self.id,
            self.alias,
            self.description,
            self.salt,
            self.encrypted_key,
            self.blocked,
            self.project_id,
            self.budget_limits.map(|l| l.0).unwrap_or_default(),
            self.request_limits.map(|l| l.0).unwrap_or_default(),
            self.token_limits.map(|l| l.0).unwrap_or_default(),
            self.deployments.into_iter().collect(),
            &application_secret,
        ).map_err(|_| DataConversionError::DefaultError {cause: "Ups".to_string()})?)
    }
}

impl_with_id_parameter_for_struct!(DbVirtualKeyRecord, VirtualKeyId);
// endregion: --- Database Model