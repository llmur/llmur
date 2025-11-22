use crate::providers::ExposesUsage;
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use serde::Serialize;
use crate::errors::{ProxyRequestError};

/// Wrapper that carries the response wrapped in a trait that exposes usage information.
#[derive(Clone)]
pub struct OpenAiCompatibleResponse<T> {
    pub(crate) result: Result<T, ProxyRequestError>
}

impl<T> OpenAiCompatibleResponse<T> {
    pub fn new(result: Result<T, ProxyRequestError>) -> Self {
        Self { result }
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
                axum::Json(data.clone()).into_response()
            }
            Err(error) => {
                // Convert error to response
                error.clone().into_response()
            }
        };
        resp.extensions_mut().insert(self);
        resp
    }
}