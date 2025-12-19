use crate::errors::ProxyError;
use crate::providers::ExposesUsage;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;

/// Wrapper that carries the response wrapped in a trait that exposes usage information.
pub struct ProxyResponse<T> {
    pub(crate) result: Result<ProviderResponse<T>, ProxyError>,
    pub(crate) request_ts: DateTime<Utc>,
    pub(crate) response_ts: DateTime<Utc>,
}

pub enum ProviderResponse<T> {
    DecodedResponse { data: T, status_code: reqwest::StatusCode },
    JsonResponse { data: serde_json::Value, status_code: reqwest::StatusCode },
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
                        (*status_code, axum::Json::<serde_json::Value>(data.clone())).into_response()
                    }
                }
            }
            Err(error) => {
                error.into_response()
            }
        };
        resp.extensions_mut().insert(Arc::new(self));
        resp
    }
}


