use crate::data::graph::Graph;
use crate::errors::LLMurError;
use crate::providers::ExposesDeployment;
use crate::LLMurState;

use axum::{
    body::{Body, to_bytes},
    extract::{FromRef, FromRequest, Request},
    http::{header::CONTENT_LENGTH, Request as HttpRequest},
    Json,
};
use http_body_util::BodyExt; // for `.collect()`
use serde::de::DeserializeOwned;
use std::{ops::Deref, sync::Arc};
use crate::routes::middlewares::auth_token_extraction_mw::{
    AuthorizationHeader, AuthorizationHeaderExtractionResult,
};

// TODO: I can probably extract all the request info here and avoid doing this in the middleware again

/// Extracts payload `T` and computes a `Graph` using the auth header + payload.deployment. Also stores request for downstream use.
pub struct WithGraph<T> {
    pub payload: T,
    pub graph: Graph,
    pub request: Request,
}

impl<T> Deref for WithGraph<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}


impl<S, T> FromRequest<S, Body> for WithGraph<T>   // <— specialize B = Body to avoid overlap
where
    Arc<LLMurState>: FromRef<S>,
    S: Send + Sync,
    T: DeserializeOwned + ExposesDeployment + Send,
{
    type Rejection = LLMurError;

    async fn from_request(request: axum::http::Request<Body>, state: &S)
                          -> Result<Self, Self::Rejection>
    {
        let app_state = Arc::<LLMurState>::from_ref(state);

        // Split to inspect extensions and buffer the body once
        let (mut parts, body) = request.into_parts();

        let auth_ext = parts
            .extensions
            .get::<AuthorizationHeaderExtractionResult>()
            .cloned()
            .ok_or(LLMurError::NotAuthorized)?;

        // Save head
        let method = parts.method.clone();
        let uri = parts.uri.clone();
        let version = parts.version;
        let mut headers = parts.headers.clone();
        headers.remove(CONTENT_LENGTH);

        // Keep original extensions for the downstream request
        let orig_extensions = std::mem::take(&mut parts.extensions);

        // Buffer body bytes
        let body_bytes = to_bytes(body, usize::MAX)
            .await
            .map_err(|_| LLMurError::BadRequest("Failed to read body".into()))?;

        // Temporary request for Json<T>
        let mut b1 = HttpRequest::builder().method(method.clone()).uri(uri.clone()).version(version);
        *b1.headers_mut().unwrap() = headers.clone();
        let req_for_json = b1
            .body(Body::from(body_bytes.clone()))
            .map_err(|_| LLMurError::BadRequest("Failed to rebuild request".into()))?;

        let Json(payload) = Json::<T>::from_request(req_for_json, state)
            .await
            .map_err(|_| LLMurError::BadRequest("Invalid JSON".into()))?;

        // Build graph
        let auth_header = auth_ext?;
        let deployment = payload.get_deployment_ref();
        let graph = match auth_header {
            AuthorizationHeader::Bearer(api_key) => {
                app_state
                    .data
                    .get_graph(&api_key, deployment, false, 10_000, &app_state.application_secret)
                    .await?
            }
        };

        // Rebuild downstream request with original extensions
        let mut b2 = HttpRequest::builder().method(method).uri(uri).version(version);
        *b2.headers_mut().unwrap() = headers;
        let mut request_for_downstream = b2
            .body(Body::from(body_bytes))
            .map_err(|_| LLMurError::BadRequest("failed to rebuild request".into()))?;
        *request_for_downstream.extensions_mut() = orig_extensions;

        Ok(WithGraph {
            payload,
            graph,
            request: request_for_downstream, // = axum::extract::Request alias
        })
    }
}
