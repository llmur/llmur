use crate::errors::ProxyError;
use crate::providers::ExposesUsage;
use axum::response::{IntoResponse, Response};
use axum::body::Body;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::HeaderValue;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Wrapper that carries the response wrapped in a trait that exposes usage information.
pub struct ProxyResponse<T> {
    pub(crate) result: Result<ProviderResponse<T>, ProxyError>,
    pub(crate) request_ts: DateTime<Utc>,
    pub(crate) response_ts: DateTime<Utc>,
}

pub enum ProviderResponse<T> {
    DecodedResponse {
        data: T,
        status_code: reqwest::StatusCode,
    },
    JsonResponse {
        data: serde_json::Value,
        status_code: reqwest::StatusCode,
    },
    Stream {
        body: Arc<Mutex<Option<Body>>>,
        status_code: reqwest::StatusCode,
        content_type: Option<String>,
    },
}
impl<T> ProviderResponse<T>
where
    T: Serialize + ExposesUsage + Send + Clone + 'static + Sync,
{
    pub(crate) fn get_input_tokens(&self) -> Option<u64> {
        match self {
            ProviderResponse::DecodedResponse { data, .. } => Some(data.get_input_tokens()),
            ProviderResponse::JsonResponse { .. } => None,
            ProviderResponse::Stream { .. } => None,
        }
    }

    pub(crate) fn get_output_tokens(&self) -> Option<u64> {
        match self {
            ProviderResponse::DecodedResponse { data, .. } => Some(data.get_output_tokens()),
            ProviderResponse::JsonResponse { .. } => None,
            ProviderResponse::Stream { .. } => None,
        }
    }

    pub(crate) fn get_status_code(&self) -> reqwest::StatusCode {
        match self {
            ProviderResponse::DecodedResponse { status_code, .. } => status_code.clone(),
            ProviderResponse::JsonResponse { status_code, .. } => status_code.clone(),
            ProviderResponse::Stream { status_code, .. } => status_code.clone(),
        }
    }
}

impl<T> ProxyResponse<T> {
    pub fn new(result: Result<ProviderResponse<T>, ProxyError>, start_ts: DateTime<Utc>) -> Self {
        Self {
            result,
            request_ts: start_ts,
            response_ts: Utc::now(),
        }
    }
}

// Not sure if I like this approach. Want to inject the result to the extensions and then return the response with the result data duplicated...
impl<T> IntoResponse for ProxyResponse<T>
where
    T: Serialize + ExposesUsage + Send + Clone + 'static + Sync,
{
    fn into_response(self) -> Response {
        let mut resp = match &self.result {
            Ok(data) => {
                // Build the JSON response.
                match data {
                    ProviderResponse::DecodedResponse { data, status_code } => {
                        (*status_code, axum::Json::<T>(data.clone())).into_response()
                    }
                    ProviderResponse::JsonResponse { data, status_code } => {
                        (*status_code, axum::Json::<serde_json::Value>(data.clone()))
                            .into_response()
                    }
                    ProviderResponse::Stream { body, status_code, content_type } => {
                        let stream_body = body
                            .lock()
                            .ok()
                            .and_then(|mut guard| guard.take())
                            .unwrap_or_else(|| Body::from(""));
                        let mut resp = Response::new(stream_body);
                        *resp.status_mut() = *status_code;
                        resp.headers_mut().insert(
                            CACHE_CONTROL,
                            HeaderValue::from_static("no-cache"),
                        );
                        let content_type = content_type
                            .as_deref()
                            .unwrap_or("text/event-stream");
                        if let Ok(value) = HeaderValue::from_str(content_type) {
                            resp.headers_mut().insert(CONTENT_TYPE, value);
                        }
                        resp
                    }
                }
            }
            Err(error) => error.into_response(),
        };
        resp.extensions_mut().insert(Arc::new(self));
        resp
    }
}
