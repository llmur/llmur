use crate::data::connection::ConnectionId;
use crate::data::deployment::DeploymentId;
use opentelemetry::KeyValue;
use opentelemetry::metrics::{Counter, Histogram, Meter};
use std::sync::Arc;
use reqwest::StatusCode;

pub(crate) struct Metrics {
    // Metrics associated with all HTTP requests
    pub(crate) http_request_counter: Counter<u64>,
    pub(crate) http_request_duration: Histogram<u64>,

    // Metrics associated with the proxied requests to the LLM providers
    pub(crate) proxy_request_counter: Counter<u64>,
    pub(crate) proxy_request_duration: Histogram<u64>,
    pub(crate) proxy_request_input_tokens: Histogram<u64>,
    pub(crate) proxy_request_output_tokens: Histogram<u64>,
}

impl Metrics {
    pub(crate) fn new(meter: Meter) -> Self {
        Metrics {
            http_request_counter: meter
                .u64_counter("http_request_total")
                .with_description("Number of HTTP requests")
                .build(),
            http_request_duration: meter
                .u64_histogram("http_request_duration")
                .with_unit("ms")
                .with_description("HTTP request duration")
                .with_boundaries(vec![
                    0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0,
                    5000.0, 7500.0, 10000.0, 25000.0, 50000.0, 75000.0, 100000.0, 250000.0,
                    500000.0, 750000.0,
                ])
                .build(),

            proxy_request_counter: meter
                .u64_counter("proxy_requests_total")
                .with_description("Number of requests to LLM provider")
                .build(),
            proxy_request_duration: meter
                .u64_histogram("proxy_request_duration")
                .with_unit("ms")
                .with_description("LLM provider request duration")
                .with_boundaries(vec![
                    0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0,
                    5000.0, 7500.0, 10000.0, 25000.0, 50000.0, 75000.0, 100000.0, 250000.0,
                    500000.0, 750000.0,
                ])
                .build(),
            proxy_request_input_tokens: meter
                .u64_histogram("proxy_request_input_tokens")
                .with_description("LLM provider input tokens")
                .with_boundaries(vec![
                    0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0,
                    5000.0, 7500.0, 10000.0, 25000.0, 50000.0, 75000.0, 100000.0, 250000.0,
                    500000.0, 750000.0,
                ])
                .build(),
            proxy_request_output_tokens: meter
                .u64_histogram("proxy_request_output_tokens")
                .with_description("LLM provider output tokens")
                .with_boundaries(vec![
                    0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0,
                    5000.0, 7500.0, 10000.0, 25000.0, 50000.0, 75000.0, 100000.0, 250000.0,
                    500000.0, 750000.0,
                ])
                .build(),
        }
    }
}

pub(crate) trait RegisterHttpRequest {
    fn register_http_request(&self, path: String, method: String, elapsed: u64);
}

impl RegisterHttpRequest for Metrics {
    fn register_http_request(&self, path: String, method: String, elapsed: u64) {
        let path_attr = KeyValue::new("path", path);
        let method_attr = KeyValue::new("method", method);

        let attributes = vec![path_attr, method_attr];
        self.http_request_counter.add(1, &attributes);
        self.http_request_duration.record(elapsed, &attributes);
    }
}

pub(crate) trait RegisterProxyRequest {
    fn register_proxy_request(
        &self,
        deployment_id: &DeploymentId,
        connection_id: &ConnectionId,
        provider: String,
        path: String,
        input_tokens: Option<u64>,
        output_tokens: Option<u64>,
        elapsed: u64,
        status_code: Option<StatusCode>
    );
}

impl RegisterProxyRequest for Metrics {
    fn register_proxy_request(
        &self,
        deployment_id: &DeploymentId,
        connection_id: &ConnectionId,
        provider: String,
        path: String,
        input_tokens: Option<u64>,
        output_tokens: Option<u64>,
        elapsed: u64,
        status_code: Option<StatusCode>
    ) {
        let deployment_id_attr = KeyValue::new("deployment_id", deployment_id.0.to_string());
        let connection_id_attr = KeyValue::new("connection_id", connection_id.0.to_string());
        let provider_attr = KeyValue::new("provider", provider);

        let attributes = vec![deployment_id_attr, connection_id_attr, provider_attr];
        
        self.proxy_request_counter.add(1, &attributes);
        self.proxy_request_duration.record(elapsed, &attributes);
        if let Some(input_tokens) = input_tokens {
            self.proxy_request_input_tokens.record(input_tokens, &attributes);
        }
        if let Some(output_tokens) = output_tokens {
            self.proxy_request_output_tokens.record(output_tokens, &attributes);
        }
    }
}

impl RegisterHttpRequest for Option<Arc<Metrics>> {
    fn register_http_request(&self, path: String, method: String, elapsed: u64) {
        if let Some(metrics) = self {
            metrics.register_http_request(path, method, elapsed);
        }
    }
}

impl RegisterProxyRequest for Option<Arc<Metrics>> {
    fn register_proxy_request(
        &self,
        deployment_id: &DeploymentId,
        connection_id: &ConnectionId,
        provider: String,
        path: String,
        input_tokens: Option<u64>,
        output_tokens: Option<u64>,
        elapsed: u64,
        status_code: Option<StatusCode>
    ) {
        if let Some(metrics) = self {
            metrics.register_proxy_request(
                deployment_id,
                connection_id,
                provider,
                path,
                input_tokens,
                output_tokens,
                elapsed,
                status_code,
            );
        }
    }
}
