use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// Initialize OpenTelemetry tracer
fn init_tracer(service_name: &str, otlp_endpoint: &str) -> SdkTracerProvider {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .build()
        .expect("Failed to create span exporter");

    let service_name_resource = Resource::builder()
        .with_service_name(service_name.to_string())
        .build();

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(service_name_resource)
        .build();

    opentelemetry::global::set_tracer_provider(provider.clone());
    provider
}

fn init_meter(service_name: &str, otlp_endpoint: &str) -> SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .build()
        .expect("Failed to create metrics exporter");

    let service_name_resource = Resource::builder()
        .with_service_name(service_name.to_string())
        .build();

    let reader = PeriodicReader::builder(exporter).build();

    let provider = SdkMeterProvider::builder()
        .with_resource(service_name_resource)
        .with_reader(reader)
        .build();

    opentelemetry::global::set_meter_provider(provider.clone());
    provider
}

// Initialize tracing subscriber with OpenTelemetry
pub(crate) fn init_tracing_subscriber(
    service_name: &str,
    otlp_endpoint: &str,
    log_level: &str,
) -> (SdkTracerProvider, SdkMeterProvider) {
    let tracer_provider = init_tracer(service_name, otlp_endpoint);
    let meter_provider = init_meter(service_name, otlp_endpoint);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(log_level.to_string()))
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_opentelemetry::layer()
                .with_tracer(tracer_provider.tracer(service_name.to_string())),
        )
        .init();

    (tracer_provider, meter_provider)
}
