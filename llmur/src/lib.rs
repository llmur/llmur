//! # LLMUR

use std::collections::BTreeSet;
use std::sync::Arc;
use axum::Router;
use uuid::Uuid;
use data::DataAccess;
use crate::data::utils;
use crate::routes::{admin_routes, openai_v1_routes};

pub mod providers;
pub mod data;
pub mod routes;
pub mod errors;

#[derive(Clone)]
pub struct LLMurState {
    pub data: &'static DataAccess,
    pub application_secret: Uuid,
    pub master_keys: BTreeSet<String>
}

pub fn router(access: DataAccess, application_secret: String, master_keys: Option<BTreeSet<String>>) -> Router {
    let state: Arc<LLMurState> = Arc::new(LLMurState{
        data: Box::leak(Box::new(access)),
        application_secret: utils::new_uuid_v5_from_string(&application_secret),
        master_keys: master_keys.unwrap_or_default(),
    });

    Router::new()
        .nest("/admin", admin_routes(state.clone()))
        .nest("/v1", openai_v1_routes(state.clone()))
        .with_state(state)
}