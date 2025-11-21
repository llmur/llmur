use crate::configuration::Configuration;
use crate::utils::AsyncInto;
use axum::{Router, ServiceExt};
use clap::Parser;
use clap_derive::Parser;
use llmur::data::DataAccess;
use std::io::Error;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

mod configuration;
mod utils;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, short)]
    configuration: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .without_time() // For early local development.
        .with_target(true)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args: Args = Args::parse();

    let configuration: Configuration = Configuration::from_yaml_file(&args.configuration);

    let data_access: DataAccess = configuration.clone().into_async().await;

    let router: Router = llmur::router(
        data_access,
        configuration.application_secret,
        configuration.master_keys.map(|keys| keys.into_iter().collect())
    );

    let listener: TcpListener = TcpListener::bind(
        format!("{}:{}",
                &configuration.host.unwrap_or("0.0.0.0".to_string()),
                &configuration.port.unwrap_or(8082)
        )
    ).await.unwrap();

    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();

    Ok(())
}
