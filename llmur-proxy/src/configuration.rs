use crate::otel::init_tracing_subscriber;
use crate::utils::AsyncFrom;
use async_trait::async_trait;
use llmur::data::DataAccessBuilder;
use llmur::metrics::Metrics;
use llmur::LLMurState;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Configuration {
    pub application_secret: String,
    pub log_level: Option<String>,
    pub otel: Option<OpenTelemetryConfiguration>,
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
pub(crate) struct OpenTelemetryConfiguration {
    exporter_otlp_endpoint: String,
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
/*
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
}*/


#[async_trait]
impl AsyncFrom<Configuration> for LLMurState {
    async fn from_async(value: Configuration) -> Self {
        if let Some(conf) =  value.otel {
            init_tracing_subscriber(
                "llmur",
                &conf.exporter_otlp_endpoint,
                &value.log_level.clone().unwrap_or(
                    std::env::var("LLMUR_LOG_LEVEL")
                        .unwrap_or(value.log_level.unwrap_or("info".to_string())),
                ),
            );
        };


        let meter = opentelemetry::global::meter("llmur");
        let metrics = Some(Arc::new(Metrics::new(meter)));

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

        let data_access = builder.build(metrics.clone()).unwrap();

        data_access.migrate_database().await.unwrap();

        let state = LLMurState {
            data: Box::leak(Box::new(data_access)),
            application_secret: Uuid::new_v5(&Uuid::NAMESPACE_DNS, value.application_secret.as_bytes()),
            master_keys: value.master_keys.unwrap_or_default().into_iter().collect(),
            metrics: metrics,
        };

        state
    }
}