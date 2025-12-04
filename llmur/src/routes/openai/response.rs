use std::sync::Arc;
use crate::providers::ExposesUsage;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::errors::{ProxyRequestError};

/// Wrapper that carries the response wrapped in a trait that exposes usage information.
#[derive(Clone)]
pub struct OpenAiCompatibleResponse<T> {
    pub(crate) result: Result<OpenAiSuccessfulResponse<T>, ProxyRequestError>,
    pub(crate) request_ts: DateTime<Utc>,
    pub(crate) response_ts: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OpenAiSuccessfulResponse<T> {
    pub(crate) data: T,
    pub(crate) status_code: reqwest::StatusCode,
}

impl<T> OpenAiCompatibleResponse<T> {
    pub fn new(result: Result<OpenAiSuccessfulResponse<T>, ProxyRequestError>, start_ts: DateTime<Utc>) -> Self {
        Self { 
            result,
            request_ts: start_ts,
            response_ts: Utc::now(),
        }
    }
}

// Not sure if I like this approach. Want to inject the result to the extensions and then return the response with the result data duplicated...
impl<T> IntoResponse for OpenAiCompatibleResponse<T>
where
    T: Serialize + ExposesUsage + Send + Clone + 'static + Sync,
{
    fn into_response(self) -> Response {
        let mut resp = match self.result.clone() {
            Ok(data) => {
                // Build the JSON response.
                axum::Json(data.data.clone()).into_response()
            }
            Err(error) => {
                // Convert error to response
                error.clone().into_response()
            }
        };
        resp.extensions_mut().insert(Arc::new(self));
        resp
    }
}

