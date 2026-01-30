use crate::configuration::Configuration;
use crate::utils::AsyncInto;
use axum::Router;
use clap::Parser;
use clap_derive::Parser;
use std::io::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use llmur::LLMurState;
use tracing::info;

mod configuration;
mod otel;
mod utils;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, short)]
    configuration: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    info!("Starting proxy");

    let args: Args = Args::parse();
    let configuration: Configuration = Configuration::from_yaml_file(&args.configuration);
    let state: LLMurState = configuration.clone().into_async().await;

    let router: Router = llmur::router(Arc::new(state));
    let router: Router = router.layer(TraceLayer::new_for_http());

    let listener: TcpListener = TcpListener::bind(format!(
        "{}:{}",
        &configuration.host.unwrap_or("0.0.0.0".to_string()),
        &configuration.port.unwrap_or(8082)
    ))
    .await
    .unwrap();

    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();

    //let _ = tracer_provider.shutdown();
    //let _ = meter_provider.shutdown();

    Ok(())
}
