use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;
use llmur::data::{DataAccess, DataAccessBuilder};
use llmur::LLMurState;
use crate::utils::AsyncFrom;

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Configuration {
    pub application_secret: String,
    pub master_keys: Option<Vec<String>>,
    pub host: Option<String>,
    pub port: Option<u32>,
    pub database_configuration: DatabaseConfiguration,
    pub cache_configuration: CacheConfiguration,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "engine")]
pub(crate) enum DatabaseConfiguration {
    #[serde(alias = "postgres")]
    Postgres(PostgresConfiguration),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "engine")]
pub(crate) enum CacheConfiguration {
    #[serde(alias = "redis")]
    Redis(RedisConfiguration),
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct PostgresConfiguration {
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,

    min_connections: Option<u32>,
    max_connections: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct RedisConfiguration {
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl Configuration {
    pub fn from_yaml_file(file: &str) -> Self {
        let f = std::fs::File::open(file).expect(&format!("Failed to open file {}", file));

        // TODO: Improve error messages
        serde_yaml::from_reader(f).expect("Failed to deserialize configuration file. Please make sure it's a proper yaml file with the expected structure")
    }
}

// TODO: Handle unwraps - It is expected to fail-fast - Should have better error messages
#[async_trait]
impl AsyncFrom<Configuration> for DataAccess {
    async fn from_async(value: Configuration) -> Self {
        let mut builder = DataAccessBuilder::new();

        match value.database_configuration {
            DatabaseConfiguration::Postgres(conf) => {
                builder = builder.with_postgres_db(
                    &conf.host,
                    conf.port,
                    &conf.database,
                    &conf.username,
                    &conf.password,
                    &conf.min_connections,
                    &conf.max_connections
                ).await.unwrap();
            }
        }

        match value.cache_configuration { CacheConfiguration::Redis(conf) => {
            builder = builder.with_redis_standalone(
                &conf.host,
                conf.port,
                &conf.username,
                &conf.password,
            ).await.unwrap();
        } }

        let data_access = builder.build().unwrap();

        data_access.migrate_database().await.unwrap();

        data_access
    }
}