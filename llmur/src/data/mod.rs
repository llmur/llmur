use crate::data::connection::ConnectionId;
use crate::data::deployment::DeploymentId;
use crate::data::graph::local_store::{GraphData, GraphDataId};
use crate::data::request_log::RequestLogData;
use crate::data::session_token::{SessionToken, SessionTokenId};
use crate::data::utils::current_timestamp_ms;
use crate::errors::{CacheAccessError, SetupError};
use chrono::{DateTime, Utc};
use redis::{
    AsyncCommands, ConnectionAddr, ConnectionInfo, FromRedisValue, ProtocolVersion,
    RedisConnectionInfo, RedisWrite, ToRedisArgs,
};
use reqwest::Client;
use serde::Serialize;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::{select, sync::mpsc, time::interval};
use crate::metrics::Metrics;

pub(crate) mod commons;
pub(crate) mod macros;
pub(crate) mod password;
pub(crate) mod utils;

pub mod connection;
pub mod connection_deployment;
pub mod deployment;
pub mod graph;
pub mod limits;
pub mod load_balancer;
pub mod membership;
pub mod project;
pub mod project_invite_code;
pub mod request_log;
pub mod session_token;
pub mod usage;
pub mod user;
pub mod virtual_key;
pub mod virtual_key_deployment;

// region:    --- Data Access
pub struct DataAccess {
    pub(crate) database: Database,
    pub(crate) cache: Arc<Cache>,
    pub(crate) http_client: reqwest::Client,

    pub(crate) request_log_tx: mpsc::Sender<Arc<RequestLogData>>,
    pub(crate) usage_log_tx: mpsc::Sender<Arc<RequestLogData>>,
}

impl DataAccess {
    pub async fn migrate_database(&self) -> Result<(), SetupError> {
        self.database.migrate().await
    }
}
// endregion: --- Data Access

// region:    --- Data Access Builder
pub struct DataAccessBuilder {
    database: Option<Database>,
    cache: Option<Cache>,
    http_client: Option<Client>,
}

impl DataAccessBuilder {
    pub fn new() -> Self {
        Self {
            database: None,
            cache: None,
            http_client: None,
        }
    }

    // Database
    pub async fn with_postgres_db(
        mut self,
        host: &str,
        port: u16,
        database: &str,
        username: &str,
        password: &str,
        min_connections: &Option<u32>,
        max_connections: &Option<u32>,
    ) -> Result<Self, SetupError> {
        if self.database.is_some() {
            return Err(SetupError::DatabaseAlreadySet);
        }
        let db = Database::new_postgres(
            host,
            port,
            database,
            username,
            password,
            min_connections,
            max_connections,
        )
        .await?;
        self.database = Some(db);
        Ok(self)
    }

    // Cache
    pub async fn with_redis_standalone(
        mut self,
        host: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> Result<Self, SetupError> {
        if self.cache.is_some() {
            return Err(SetupError::ExternalCacheAlreadySet);
        }
        let cache = Cache::new_redis_standalone(host, port, username, password).await?;
        self.cache = Some(cache);
        Ok(self)
    }

    // HTTP client
    pub fn with_http_client(
        mut self,
        builder: reqwest::ClientBuilder,
    ) -> Result<Self, SetupError> {
        if self.http_client.is_some() {
            return Err(SetupError::HttpClientAlreadySet);
        }
        let client = builder.build()?; // may error -> BuilderError::Http
        self.http_client = Some(client);
        Ok(self)
    }

    // Finalize
    pub fn build(self, metrics: Option<Arc<Metrics>>,) -> Result<DataAccess, SetupError> {
        let database = self.database.ok_or(SetupError::MissingDatabase)?;
        let cache = Arc::new(self.cache.unwrap_or(Cache::local_only()));
        let http_client = self.http_client.unwrap_or_else(Client::new);

        let (request_log_tx, request_log_rx) = mpsc::channel::<Arc<RequestLogData>>(256); // TODO: Parameterise buffer size
        let (usage_log_tx, usage_log_rx) = mpsc::channel::<Arc<RequestLogData>>(256); // TODO: Parameterise buffer size

        // Background writer (flush every 750ms or 500 events)
        // TODO: Make parameters configurable
        spawn_request_log_writer(
            database.clone(),
            request_log_rx,
            Duration::from_millis(750),
            500,
            metrics
        );
        spawn_usage_writer(cache.clone(), usage_log_rx, Duration::from_millis(50), 10);

        Ok(DataAccess {
            database,
            cache,
            http_client,
            request_log_tx,
            usage_log_tx,
        })
    }
}
// endregion: --- Data Access Builder

// region:    --- Database
pub(crate) enum Database {
    Postgres { pool: sqlx::Pool<sqlx::Postgres> },
}

impl Clone for Database {
    fn clone(&self) -> Self {
        match self {
            Database::Postgres { pool } => Database::Postgres { pool: pool.clone() },
        }
    }
}

impl Database {
    pub async fn new_postgres(
        host: &str,
        port: u16,
        database: &str,
        username: &str,
        password: &str,
        min_connections: &Option<u32>,
        max_connections: &Option<u32>,
    ) -> Result<Database, SetupError> {
        let mut pool_options = PgPoolOptions::new();

        if let Some(min) = min_connections {
            pool_options = pool_options.max_connections(min.clone());
        }
        if let Some(max) = max_connections {
            pool_options = pool_options.max_connections(max.clone());
        }

        Ok(Database::Postgres {
            pool: pool_options
                .connect(&format!(
                    "postgres://{}:{}@{}:{}/{}",
                    username, password, host, port, database
                ))
                .await?,
        })
    }

    pub async fn migrate(&self) -> Result<(), SetupError> {
        match self {
            Database::Postgres { pool } => {
                let m = Migrator::new(Path::new(&format!(
                    "{}/postgres/migration",
                    env!("CARGO_MANIFEST_DIR")
                )))
                .await?;
                let _ = m.run(pool).await?;
                Ok(())
            }
        }
    }
}
// endregion: --- Database

// region:    --- Cache
pub(crate) struct Cache {
    pub(crate) local: LocalStore,
    pub(crate) external: Option<ExternalCache>,
}

impl Cache {
    pub(crate) fn local_only() -> Self {
        Cache {
            local: LocalStore::new(),
            external: None,
        }
    }

    pub(crate) async fn new_redis_standalone(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> Result<Self, SetupError> {
        let client = redis::Client::open(ConnectionInfo {
            addr: ConnectionAddr::Tcp(host.to_string(), port),
            redis: RedisConnectionInfo {
                db: 0,
                username: Some(username.to_string()),
                password: Some(password.to_string()),
                protocol: ProtocolVersion::default(),
            },
        })?;

        let mut con = client
            .get_multiplexed_async_connection()
            .await?;

        Ok(Cache {
            local: LocalStore::new(),
            external: Some(ExternalCache::Redis { connection: con }),
        })
    }
}

// region:    --- Cache Access
impl Cache {
    pub(crate) fn get_local_record<R, K>(&self, id: &K) -> Option<LocallyStoredValue<R>>
    where
        R: LocallyStored<K>,
        K: Ord + Clone + Display,
    {
        let map = R::get_local_map(&self.local);
        let cached = {
            let guard = map.lock().ok()?;
            guard.get(id)?.clone()
        };
        Some(cached)
    }

    pub(crate) fn get_local_records<R, K>(
        &self,
        ids: &BTreeSet<K>,
    ) -> BTreeMap<K, Option<LocallyStoredValue<R>>>
    where
        R: LocallyStored<K>,
        K: Ord + Clone + Display,
    {
        let map = R::get_local_map(&self.local);
        let guard = match map.lock() {
            Ok(guard) => guard,
            Err(_) => return BTreeMap::new(),
        };

        ids.iter()
            .map(|id| {
                let cached = guard.get(id).cloned();
                (id.clone(), cached)
            })
            .collect()
    }

    pub(crate) fn set_local_record<R, K>(&self, record: R) -> ()
    where
        R: LocallyStored<K>,
        K: Ord + Clone + Display,
    {
        if let Ok(mut map) = R::get_local_map(&self.local).lock() {
            map.insert(
                record.get_id_ref().clone(),
                LocallyStoredValue::new(record.clone()),
            );
        }
    }

    pub(crate) fn set_local_records<R, K>(&self, records: Vec<R>) -> ()
    where
        R: LocallyStored<K>,
        K: Ord + Clone + Display,
    {
        if let Ok(mut map) = R::get_local_map(&self.local).lock() {
            map.extend(
                records
                    .into_iter()
                    .map(|record| (record.get_id_ref().clone(), LocallyStoredValue::new(record))),
            );
        }
    }

    pub(crate) fn delete_local_record<R, K>(&self, id: &K) -> ()
    where
        R: LocallyStored<K>,
        K: Ord + Clone + Display,
    {
        if let Ok(mut map) = R::get_local_map(&self.local).lock() {
            map.remove(id);
        }
    }

    pub(crate) fn delete_local_records<R, K>(&self, ids: &BTreeSet<K>) -> ()
    where
        R: LocallyStored<K>,
        K: Ord + Clone + Display,
    {
        if let Ok(mut map) = R::get_local_map(&self.local).lock() {
            for id in ids {
                map.remove(&id);
            }
        }
    }
}

// endregion: --- Cache Access

// region:    --- External Cache
pub(crate) enum ExternalCache {
    Redis {
        connection: redis::aio::MultiplexedConnection,
    },
}

impl ExternalCache {
    pub(crate) async fn get_values(
        &self,
        keys: &BTreeSet<String>,
    ) -> Result<BTreeMap<String, Option<String>>, CacheAccessError> {
        match self {
            ExternalCache::Redis { connection } => {
                let mut conn = connection.clone();

                // Convert BTreeSet to Vec for Redis mget
                let keys_vec: Vec<&String> = keys.iter().collect();

                // Use mget to retrieve all values for the given keys
                let values: Vec<Option<String>> =
                    conn.mget(keys_vec)
                        .await?;

                // Convert into BTreeMap
                let result = keys
                    .iter()
                    .cloned()
                    .zip(values)
                    .collect::<BTreeMap<String, Option<String>>>();

                Ok(result)
            }
        }
    }
}

// endregion: --- External Cache

// region:    --- Local Cache
#[derive(Debug, Clone)]
pub(crate) struct LocallyStoredValue<T> {
    pub(crate) value: T,
    pub(crate) timestamp_ms: i64,
}

impl<T> LocallyStoredValue<T> {
    pub(crate) fn new(value: T) -> Self {
        LocallyStoredValue {
            value,
            timestamp_ms: current_timestamp_ms(),
        }
    }

    pub(crate) fn is_expired(&self, now: &DateTime<Utc>, ttl: u32) -> bool {
        now.timestamp_millis() - self.timestamp_ms > ttl as i64
    }
}

pub(crate) struct LocalStore {
    pub(crate) session_tokens: Mutex<BTreeMap<SessionTokenId, LocallyStoredValue<SessionToken>>>,
    pub(crate) graphs: Mutex<BTreeMap<GraphDataId, LocallyStoredValue<GraphData>>>,

    // Tracks current connections per ConnectionId
    pub(crate) opened_connections_counter: Mutex<BTreeMap<ConnectionId, LocallyStoredValue<u32>>>,

    // Tracks round-robin index per deployment
    pub(crate) deployment_rr_index: Mutex<BTreeMap<DeploymentId, LocallyStoredValue<usize>>>,
}

impl LocalStore {
    fn new() -> Self {
        LocalStore {
            session_tokens: Default::default(),
            graphs: Default::default(),
            opened_connections_counter: Default::default(),
            deployment_rr_index: Default::default(),
        }
    }
}
// endregion: --- Local Cache

#[async_trait::async_trait]
pub(crate) trait LocallyStored<K>:
    WithIdParameter<K> + Clone + Send + Sync + 'static
where
    K: Ord + Clone + Display,
{
    fn get_local_map(local: &LocalStore) -> &Mutex<BTreeMap<K, LocallyStoredValue<Self>>>;
}

// endregion: --- Cache

// region:    --- Commons
pub trait WithIdParameter<K> {
    fn get_id_ref(&self) -> &K;
}
// endregion: --- Commons

// region:    --- Request Log Data
fn spawn_request_log_writer(
    database: Database,
    rx: mpsc::Receiver<Arc<RequestLogData>>,
    flush_every: Duration,
    max_batch: usize,
    metrics: Option<Arc<Metrics>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut rx = rx;
        let mut tick = interval(flush_every);
        let mut batch: Vec<Arc<RequestLogData>> = Vec::with_capacity(max_batch);

        println!("### request log writer started");

        loop {
            select! {
                _ = tick.tick() => {
                    if !batch.is_empty() {
                        println!("### request log tick flush triggered with {} events", batch.len());
                        let to_write = std::mem::take(&mut batch);
                        if let Err(e) = database.insert_request_logs(&to_write, &metrics).await {
                            println!("### request log tick flush failed {:?}", e);
                        }
                    }
                }
                msg = rx.recv() => {
                    match msg {
                        Some(ev) => {
                            batch.push(ev);
                            if batch.len() >= max_batch {
                                let to_write = std::mem::take(&mut batch);
                                if let Err(e) = database.insert_request_logs(&to_write, &metrics).await {
                                    println!("### request log size flush failed");
                                }
                            }
                        }
                        None => {
                            if !batch.is_empty() {
                                println!("### request log shutdown flush triggered with {} events", batch.len());
                                let _ = database.insert_request_logs(&batch, &metrics).await;
                            }
                            break;
                        }
                    }
                }
            }
        }
    })
}

fn spawn_usage_writer(
    cache: Arc<Cache>,
    rx: mpsc::Receiver<Arc<RequestLogData>>,
    flush_every: Duration,
    max_batch: usize,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut rx = rx;
        let mut tick = interval(flush_every);
        let mut batch: Vec<Arc<RequestLogData>> = Vec::with_capacity(max_batch);

        println!("### usage writer started");

        loop {
            select! {
                _ = tick.tick() => {
                    if !batch.is_empty() {
                        println!("### usage writer tick flush triggered with {} events", batch.len());
                        let to_write = std::mem::take(&mut batch);
                        if let Err(e) = cache.increment_usage_data(to_write).await {
                            println!("### usage writer tick flush failed {:?}", e);
                        }
                    }
                }
                msg = rx.recv() => {
                    match msg {
                        Some(ev) => {
                            batch.push(ev);
                            if batch.len() >= max_batch {
                                let to_write = std::mem::take(&mut batch);
                                if let Err(e) = cache.increment_usage_data(to_write).await {
                                    println!("### usage writer size flush failed {:?}", e);
                                }
                            }
                        }
                        None => {
                            if !batch.is_empty() {
                                println!("### usage writer shutdown flush triggered with {} events", batch.len());
                                if let Err(e) = cache.increment_usage_data(batch).await {
                                    println!("### usage writer shutdown flush failed {:?}", e);
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    })
}

// endregion: --- Request Log Data
