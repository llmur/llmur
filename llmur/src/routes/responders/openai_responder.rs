use crate::providers::ExposesUsage;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Wrapper that carries the response wrapped in a trait that exposes usage information.
#[derive(Clone)]
pub struct OpenAiCompatibleResponse<T> {
    pub(crate) inner: T
}

impl<T> OpenAiCompatibleResponse<T> {
    pub fn new(inner: T ) -> Self {
        Self { inner }
    }
}

impl<T> IntoResponse for OpenAiCompatibleResponse<T>
where
    T: Serialize + ExposesUsage + Send + Clone + 'static + Sync,
{
    fn into_response(self) -> Response {
        // Build the JSON response.
        let mut resp = axum::Json(self.inner.clone()).into_response();

        // Attach structured data for middleware.
        resp.extensions_mut().insert(self);

        resp
    }
}