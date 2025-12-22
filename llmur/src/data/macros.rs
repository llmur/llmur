
// region:    --- Data Access Macros

#[macro_export]
macro_rules! default_get_fn {
    ($type:ty, $id_type:ty, $singular:ident) => {
        paste::paste! {
            async fn [<__get_ $singular>](&self, id: &$id_type, application_secret: &Option<uuid::Uuid>, metrics: &Option<std::sync::Arc<crate::metrics::Metrics>>) -> Result<Option<$type>, crate::errors::DataAccessError> {
                // Hit the DB and convert the result
                let maybe_value: Option<$type> = self
                    .database
                    .[<get_ $singular>](metrics, &id.clone()) // Returns promise Option<DbXXXRecord>
                    .await?
                    .map(|v| v.convert(application_secret)) // Returns Option<Result<X,Y>>
                    .transpose()?;

                Ok(maybe_value)
            }
        }
    };
}

#[macro_export]
macro_rules! default_getm_fn {
    ($type:ty, $id_type:ty, $plural:ident) => {
        paste::paste! {
            async fn [<__get_ $plural>](&self, ids: &std::collections::BTreeSet<$id_type>, application_secret: &Option<uuid::Uuid>, metrics: &Option<std::sync::Arc<crate::metrics::Metrics>>) -> Result<std::collections::BTreeMap<$id_type, Option<$type>>, crate::errors::DataAccessError> {
                // Hit the DB and convert the result
                let result: std::collections::BTreeMap<$id_type, Option<$type>> = self
                    .database
                    .[<get_ $plural>](metrics, &ids.iter().cloned().collect())
                    .await? // Returns Result<BTreeMap<$id_type, Option<Obj>>, Error>
                    .into_iter()
                    .map(|(k, v)| {
                        v.map(|record| record.convert(application_secret))  // Option<Result<Obj, Error>>
                        .transpose()                                        // Result<Option<Obj>, Error>
                        .map(|converted| (k, converted))                    // Result<(Id, Option<Obj>), Error>
                    })
                    .collect::<Result<std::collections::BTreeMap<$id_type, Option<$type>>, crate::errors::DbRecordConversionError>>()?;

                Ok(result)
            }
        }
    };
}

#[macro_export]
macro_rules! default_create_fn {
    (
        $type:ty,
        $singular:ident,
        $( $param_name:ident : $param_ty:ty ),* $(,)?
    ) => {
        paste::paste! {
            async fn [<__create_ $singular>](&self, $( $param_name: $param_ty ),* , application_secret: &Option<uuid::Uuid>, metrics: &Option<std::sync::Arc<crate::metrics::Metrics>>) -> Result<$type, crate::errors::DataAccessError> {
                let record_id = self.database.[<insert_ $singular>](metrics, $( $param_name ),*).await?;
                let record = self.[<__get_ $singular>](&record_id, application_secret, metrics)
                    .await
                    .map_err(|e| crate::errors::DataAccessError::FailedToGetCreatedResource(Box::new(e), stringify!($singular).to_string(), record_id.0))?
                    .ok_or(crate::errors::DataAccessError::CreatedResourceNotFound(stringify!($singular).to_string(), record_id.0))?;
                Ok(record)
            }
        }
    };
}

#[macro_export]
macro_rules! default_search_fn {
    // Case 1: No search parameters
    (
        $type:ty,
        $plural:ident,
    ) => {
        paste::paste! {
            async fn [<__search_ $plural>](&self, application_secret: &Option<uuid::Uuid>, metrics: &Option<std::sync::Arc<crate::metrics::Metrics>>) -> Result<Vec<$type>, crate::errors::DataAccessError> {
                let values: Vec<$type> = self
                    .database
                    .[<search_ $plural>](metrics) // Returns promise Result<Vec<DbXXXRecord>, Error>
                    .await?
                    .into_iter()
                    .map(|v| v.convert(application_secret)) // convert is defined and returns Result<$type, Error>
                    .collect::<Result<Vec<_>, _>>()?; // Specify the collect type

                Ok(values)
            }
        }
    };
    // Case 2: With search parameters
    (
        $type:ty,
        $plural:ident,
        $( $param_name:ident : $param_ty:ty ),+ $(,)?
    ) => {
        paste::paste! {
            async fn [<__search_ $plural >](&self, $( $param_name: $param_ty ),+, application_secret: &Option<uuid::Uuid>, metrics: &Option<std::sync::Arc<crate::metrics::Metrics>>) -> Result<Vec<$type>, crate::errors::DataAccessError> {
                let values: Vec<$type> = self
                    .database
                    .[<search_ $plural>](metrics, $( $param_name ),+) // Returns promise Result<Vec<DbXXXRecord>, Error>
                    .await?
                    .into_iter()
                    .map(|v| v.convert(application_secret)) // convert is defined and returns Result<$type, Error>
                    .collect::<Result<Vec<_>, _>>()?; // Specify the collect type

                Ok(values)
            }
        }
    };
}

#[macro_export]
macro_rules! default_delete_fn {
    ($type:ty, $id_type:ty, $singular:ident) => {
        paste::paste! {
            async fn [<__delete_ $singular>](&self, id: &$id_type, metrics: &Option<std::sync::Arc<crate::metrics::Metrics>>) -> Result<u64, crate::errors::DataAccessError> {
                let n = self.database.[<delete_ $singular>](metrics, id).await?;

                Ok(n)
            }
        }
    };
}

#[macro_export]
macro_rules! default_access_fns {
    (
        $type:ty,
        $id_type:ty,
        $singular:ident,
        $plural:ident,
        create => { $( $create_param_name:ident : $create_param_ty:ty ),* $(,)? },
        search => { }
    ) => {
        impl crate::data::DataAccess {
            crate::default_get_fn!($type, $id_type, $singular);
            crate::default_getm_fn!($type, $id_type, $plural);
            crate::default_delete_fn!($type, $id_type, $singular);
            crate::default_create_fn!(
                $type,
                $singular,
                $( $create_param_name : $create_param_ty ),*
            );
            crate::default_search_fn!(
                $type,
                $plural,
            );
        }
    };
    (
        $type:ty,
        $id_type:ty,
        $singular:ident,
        $plural:ident,
        create => { $( $create_param_name:ident : $create_param_ty:ty ),* $(,)? },
        search => { $( $search_param_name:ident : $search_param_ty:ty ),+ $(,)? }
    ) => {
        impl crate::data::DataAccess {
            crate::default_get_fn!($type, $id_type, $singular);
            crate::default_getm_fn!($type, $id_type, $plural);
            crate::default_delete_fn!($type, $id_type, $singular);
            crate::default_create_fn!(
                $type,
                $singular,
                $( $create_param_name : $create_param_ty ),*
            );
            crate::default_search_fn!(
                $type,
                $plural,
                $( $search_param_name : $search_param_ty ),+
            );
        }
    };
}
// endregion: --- Data Access Macros

// region:    --- Database Access Macros
#[macro_export]
macro_rules! default_db_insert_fn {
    (
        $type:ty,
        $id_type:ty,
        $singular:ident,
        $pg_query_fn:path,
        $( $param_name:ident : $param_ty:ty ),* $(,)?
    ) => {
        paste::paste! {
            pub(crate) async fn [<insert_ $singular>](&self, metrics: &Option<std::sync::Arc<crate::metrics::Metrics>>, $( $param_name: $param_ty ),*) -> Result<$id_type, crate::errors::DataAccessError> {
                use crate::metrics::RegisterDatabaseRequest;

                let operation = concat!("db.insert.", stringify!($singular));
                let span = tracing::trace_span!("database_operation", operation= %operation);

                tracing::Instrument::instrument(
                    async move {
                        match self {
                            crate::data::Database::Postgres { pool } => {
                                let start = std::time::Instant::now();

                                let mut query = $pg_query_fn($( $param_name ),*);
                                let sql = query.build_query_as::<$id_type>();
                                let result = sql.fetch_one(pool).await;

                                metrics.register_database_request(&operation, start.elapsed().as_millis() as u64, result.is_ok());

                                Ok(result?)
                            }
                        }
                    }, span
                ).await
            }
        }
    };
}

#[macro_export]
macro_rules! default_db_search_fn {
    // Variant without parameters
    (
        $type:ty,
        $plural:ident,
        $pg_query_fn:path
    ) => {
        paste::paste! {
            pub(crate) async fn [<search_ $plural>](&self, metrics: &Option<std::sync::Arc<crate::metrics::Metrics>>) -> Result<Vec<$type>, crate::errors::DataAccessError> {
                use crate::metrics::RegisterDatabaseRequest;

                let operation = concat!("db.search.", stringify!($plural));
                let span = tracing::trace_span!("database_operation", operation= %operation);

                tracing::Instrument::instrument(
                    async move {
                        match self {
                            crate::data::Database::Postgres { pool } => {
                                let start = std::time::Instant::now();

                                let mut query = $pg_query_fn();
                                let sql = query.build_query_as::<$type>();
                                let result = sql.fetch_all(pool).await;

                                metrics.register_database_request(&operation, start.elapsed().as_millis() as u64, result.is_ok());

                                Ok(result?)
                            }
                        }
                    }, span
                ).await
            }
        }
    };

    // Variant with parameters
    (
        $type:ty,
        $plural:ident,
        $pg_query_fn:path,
        $( $param_name:ident : $param_ty:ty ),+ $(,)?
    ) => {
        paste::paste! {
            pub(crate) async fn [<search_ $plural>](&self, metrics: &Option<std::sync::Arc<crate::metrics::Metrics>>, $( $param_name: $param_ty ),+) -> Result<Vec<$type>, crate::errors::DataAccessError> {
                use crate::metrics::RegisterDatabaseRequest;

                let operation = concat!("db.search.", stringify!($plural));
                let span = tracing::trace_span!("database_operation", operation= %operation);

                tracing::Instrument::instrument(
                    async move {
                        match self {
                            crate::data::Database::Postgres { pool } => {
                                let start = std::time::Instant::now();

                                let mut query = $pg_query_fn($( $param_name ),*);
                                let sql = query.build_query_as::<$type>();
                                let result = sql.fetch_all(pool).await;
        
                                metrics.register_database_request(&operation, start.elapsed().as_millis() as u64, result.is_ok());

                                Ok(result?)
                            }
                        }
                    }, span
                ).await
            }
        }
    };
}

#[macro_export]
macro_rules! default_db_get_fn {
    (
        $type:ty,
        $id_type: ty,
        $singular:ident,
        $pg_query_fn:path
    ) => {
        paste::paste! {
            pub(crate) async fn [<get_ $singular>](&self, metrics: &Option<std::sync::Arc<crate::metrics::Metrics>>, id: &$id_type) -> Result<Option<$type>, crate::errors::DataAccessError> {
                use crate::metrics::RegisterDatabaseRequest;

                let operation = concat!("db.get.", stringify!($singular));
                let span = tracing::trace_span!("database_operation", operation= %operation);

                tracing::Instrument::instrument(
                    async move {
                        match self {
                            crate::data::Database::Postgres { pool } => {
                                let start = std::time::Instant::now();

                                let mut query = $pg_query_fn(id);
                                let sql= query.build_query_as::<$type>();
                                let result = sql.fetch_optional(pool).await;
        
                                metrics.register_database_request(&operation, start.elapsed().as_millis() as u64, result.is_ok());

                                Ok(result?)
                            }
                        }
                    }, span
                ).await
            }
        }
    };
}

#[macro_export]
macro_rules! default_db_get_multiple_fn {
    (
        $type:ty,
        $id_type: ty,
        $plural:ident,
        $pg_query_fn:path
    ) => {
        paste::paste! {
            pub(crate) async fn [<get_ $plural>](&self, metrics: &Option<std::sync::Arc<crate::metrics::Metrics>>, ids: &Vec<$id_type>) -> Result<std::collections::BTreeMap<$id_type, Option<$type>>, crate::errors::DataAccessError> {
                use crate::metrics::RegisterDatabaseRequest;

                let operation = concat!("db.get.", stringify!($plural));
                let span = tracing::trace_span!("database_operation", operation= %operation);

                tracing::Instrument::instrument(
                    async move {
                        match self {
                            crate::data::Database::Postgres { pool } => {
                                let start = std::time::Instant::now();

                                let mut query = $pg_query_fn(ids);
                                let sql= query.build_query_as::<$type>();
                                let result = sql.fetch_all(pool).await;

                                metrics.register_database_request(&operation, start.elapsed().as_millis() as u64, result.is_ok());

                                let result = result?;
        
                                // Create a map of found items using owned values
                                let found_items: std::collections::BTreeMap<$id_type, $type> = result
                                    .into_iter()
                                    .map(|item| (<$type as crate::data::WithIdParameter<$id_type>>::get_id_ref(&item).clone(), item))
                                    .collect();
        
                                // Create final map with all requested IDs
                                let map: std::collections::BTreeMap<$id_type, Option<$type>> = ids
                                    .iter()
                                    .map(|id| (id.clone(), found_items.get(id).cloned()))
                                    .collect();
        
                                Ok(map)
                            }
                        }
                    }, span
                ).await
            }
        }
    };
}
#[macro_export]
macro_rules! default_db_delete_fn {
    (
        $id_type:ty,
        $singular:ident,
        $pg_query_fn:path
    ) => {
        paste::paste! {
            pub(crate) async fn [<delete_ $singular>](&self, metrics: &Option<std::sync::Arc<crate::metrics::Metrics>>, id: &$id_type) -> Result<u64, crate::errors::DataAccessError> {
                use crate::metrics::RegisterDatabaseRequest;

                let operation = concat!("db.delete.", stringify!($singular));
                let span = tracing::trace_span!("database_operation", operation= %operation);

                tracing::Instrument::instrument(
                    async move {
                        match self {
                            crate::data::Database::Postgres { pool } => {
                                let start = std::time::Instant::now();

                                let mut query = $pg_query_fn(id);
                                let sql = query.build();
                                let result = sql.execute(pool).await;

                                metrics.register_database_request(&operation, start.elapsed().as_millis() as u64, result.is_ok());

                                Ok(result?.rows_affected())
                            }
                        }
                    }, span
                ).await
            }
        }
    };
}

#[macro_export]
macro_rules! default_database_access_fns {
    (
        $type:ty,
        $id_type:ty,
        $singular:ident,
        $plural:ident,
        insert => { $( $create_param_name:ident : $create_param_ty:ty ),* $(,)? },
        search => { }
    ) => {
        impl crate::data::Database {
            crate::default_db_insert_fn!(
                $type,
                $id_type,
                $singular,
                crate::data::$singular::pg_insert,
                $( $create_param_name : $create_param_ty ),*
            );
            crate::default_db_get_fn!(
                $type,
                $id_type,
                $singular,
                crate::data::$singular::pg_get
            );
            crate::default_db_delete_fn!(
                $id_type,
                $singular,
                crate::data::$singular::pg_delete
            );
            crate::default_db_get_multiple_fn!(
                $type,
                $id_type,
                $plural,
                crate::data::$singular::pg_getm
            );
            crate::default_db_search_fn!(
                $type,
                $plural,
                crate::data::$singular::pg_search
            );
        }
    };
    (
        $type:ty,
        $id_type:ty,
        $singular:ident,
        $plural:ident,
        insert => { $( $create_param_name:ident : $create_param_ty:ty ),* $(,)? },
        search => { $( $search_param_name:ident : $search_param_ty:ty ),+ $(,)? }
    ) => {
        impl crate::data::Database {
            crate::default_db_insert_fn!(
                $type,
                $id_type,
                $singular,
                crate::data::$singular::pg_insert,
                $( $create_param_name : $create_param_ty ),*
            );
            crate::default_db_get_fn!(
                $type,
                $id_type,
                $singular,
                crate::data::$singular::pg_get
            );
            crate::default_db_delete_fn!(
                $id_type,
                $singular,
                crate::data::$singular::pg_delete
            );
            crate::default_db_get_multiple_fn!(
                $type,
                $id_type,
                $plural,
                crate::data::$singular::pg_getm
            );
            crate::default_db_search_fn!(
                $type,
                $plural,
                crate::data::$singular::pg_search,
                $( $search_param_name : $search_param_ty ),+
            );
        }
    };
}
// endregion: --- Database Access Macros

// region:    --- Cache Access Macros
#[macro_export]
macro_rules! impl_locally_stored {
    ($type:ty, $id_type:ty, $field:ident) => {
        #[async_trait::async_trait]
        impl crate::data::LocallyStored<$id_type> for $type {
            fn get_local_map(local: &crate::data::LocalStore) -> &std::sync::Mutex<std::collections::BTreeMap<$id_type, crate::data::LocallyStoredValue<Self>>> {
                &local.$field
            }
        }
    };
}

#[macro_export]
macro_rules! impl_local_store_accessors {
    ($type:ty, $id_type:ty, $singular:ident, $plural:ident) => {
        paste::paste! {
            impl crate::data::Cache {
                pub(crate) fn [<get_local_ $singular>](&self, id: &$id_type) -> Option<crate::data::LocallyStoredValue<$type>> {
                    let _span = tracing::trace_span!(concat!("local.get.", stringify!($singular))).entered();
                    self.get_local_record::<$type, $id_type>(id)
                }

                pub(crate) fn [<get_local_ $plural>](&self, ids: &std::collections::BTreeSet<$id_type>) -> std::collections::BTreeMap<$id_type, Option<crate::data::LocallyStoredValue<$type>>> {
                    let _span = tracing::trace_span!(concat!("local.get.", stringify!($plural))).entered();
                    self.get_local_records::<$type, $id_type>(ids)
                }

                pub(crate) fn [<set_local_ $singular>](&self, value: $type) -> () {
                    let _span = tracing::trace_span!(concat!("local.set.", stringify!($singular))).entered();
                    self.set_local_record::<$type, $id_type>(value)
                }

                pub(crate) fn [<set_local_ $plural>](&self, values: Vec<$type>) -> () {
                    let _span = tracing::trace_span!(concat!("local.set.", stringify!($plural))).entered();
                    self.set_local_records::<$type, $id_type>(values)
                }

                pub(crate) fn [<delete_local_ $singular>](&self, id: &$id_type) -> () {
                    let _span = tracing::trace_span!(concat!("local.delete.", stringify!($singular))).entered();
                    self.delete_local_record::<$type, $id_type>(id)
                }
            }
        }
    };
}
// endregion: --- Cache Access Macros

#[macro_export]
macro_rules! impl_with_id_parameter_for_struct {
    ($type:ty, $id_type:ty) => {
        impl crate::data::WithIdParameter<$id_type> for $type {
            fn get_id_ref(&self) -> &$id_type {
                &self.id
            }
        }
    };
}

#[macro_export]
macro_rules! impl_structured_id_utils {
    ($id_type:ty) => {
        impl From<Uuid> for $id_type {
            fn from(id: Uuid) -> Self {
                Self(id)
            }
        }

        impl From<$id_type> for Uuid {
            fn from(id: $id_type) -> Self {
                id.0
            }
        }

        impl std::fmt::Display for $id_type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::str::FromStr for $id_type {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<$id_type, Self::Err> {
                Ok(Self(Uuid::from_str(s)?))
            }
        }
    };
}
