use crate::data::graph::Graph;
use crate::errors::LLMurError;
use crate::providers::ExposesDeployment;
use crate::LLMurState;

use axum::{
    body::Body,
    extract::{FromRef, FromRequest}
    ,
    Json,
};
use http_body_util::BodyExt;
use crate::routes::middlewares::auth_token_extraction_mw::{
    AuthorizationHeader, AuthorizationHeaderExtractionResult,
};

use serde::de::DeserializeOwned;
use std::{ops::Deref, sync::Arc};

/// Consumes the request and extracts/builds all required information from it
pub struct OpenAiRequestData<T> {
    pub payload: T,
    pub graph: Graph,
    pub method: String,
    pub path: String
}

impl<T> Deref for OpenAiRequestData<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}


impl<S, T> FromRequest<S, Body> for OpenAiRequestData<T>   // <— specialize B = Body to avoid overlap
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

        let auth_ext = request
            .extensions()
            .get::<AuthorizationHeaderExtractionResult>()
            .cloned()
            .ok_or(LLMurError::NotAuthorized)?;

        // Save head
        let method = request.method().to_string().clone();
        let path = request.uri().path().to_string();

        // Consumes request
        let Json(payload) = Json::<T>::from_request(request, state)
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

        Ok(OpenAiRequestData {
            payload,
            graph,
            method,
            path
        })
    }
}
