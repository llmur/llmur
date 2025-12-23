use crate::configuration::Configuration;
use crate::utils::AsyncInto;
use axum::Router;
use clap::Parser;
use clap_derive::Parser;
use std::io::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use tracing::{info, info_span};
use llmur::LLMurState;

mod configuration;
mod utils;
mod otel;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, short)]
    configuration: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://0.0.0.0:4317".into());

    info!("Starting proxy");

    let args: Args = Args::parse();

    let configuration: Configuration = Configuration::from_yaml_file(&args.configuration);

    let state: LLMurState = configuration.clone().into_async().await;

    let router: Router = llmur::router(
        Arc::new(state),
    );

    let router: Router = router.layer(TraceLayer::new_for_http().make_span_with(
        |request: &axum::http::Request<_>| {
            let matched_path = request
                .extensions()
                .get::<axum::extract::MatchedPath>()
                .map(|path| path.as_str())
                .unwrap_or("unknown");

            info_span!(
                "request",
                method = ?request.method(),
                path = matched_path,
            )
        },
    ));

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


/*
pub fn init_tracing(service_name: &str, otlp_endpoint: &str) {
    // Use W3C `traceparent`/`tracestate` for propagation
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    // Build OTLP exporter using gRPC (Tonic). This will send to an OTLP endpoint,
    // e.g. Jaeger or an OpenTelemetry Collector.
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint.to_string())
        .build()
        .expect("failed to create OTLP span exporter");

    // Attach standard resource attributes such as service.name

    let resource = Resource::builder()
        .with_service_name(service_name.to_string())
        .build();

    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();

    let tracer = tracer_provider.tracer(service_name.to_string());

    // Make this the global provider so any code using `global::tracer` can export.
    opentelemetry::global::set_tracer_provider(tracer_provider);

    // Wire OpenTelemetry into `tracing`
    let otel_layer = tracing_opentelemetry::OpenTelemetryLayer::new(tracer);
    let fmt_layer = tracing_subscriber::fmt::layer();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(fmt_layer)
        .with(otel_layer)
        .init();

    // For a production app, hold onto `tracer_provider.clone()` in your
    // shutdown logic and call `tracer_provider.shutdown()` to flush spans.
}*/
