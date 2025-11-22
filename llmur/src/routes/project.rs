use std::sync::Arc;
use axum::{Extension, Json, Router};
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use serde::{Deserialize, Serialize};
use crate::errors::{DataAccessError, LLMurError};
use crate::{impl_from_vec_result, LLMurState};
use crate::data::project::{Project, ProjectId};
use crate::data::user::ApplicationRole;
use crate::routes::middleware::user_context::{AuthorizationManager, UserContext, UserContextExtractionResult};
use crate::routes::StatusResponse;

// region:    --- Routes
pub(crate) fn routes(state: Arc<LLMurState>) -> Router<Arc<LLMurState>> {
    Router::new()
        // Project routes
        //.route("/", get(list_projects))
        .route("/", post(create_project))
        .route("/{id}", get(get_project))
        .route("/{id}", delete(delete_project))
        //.route("/:id", put(update_project))
        // Project membership routes
        //.route("/:id/memberships", get(get_project_memberships))
        //.route("/:id/memberships", post(create_project_membership))
        // Project invite codes routes
        //.route("/:id/invite_codes", get(get_project_invite_codes))
        //.route("/:id/invite_codes", post(create_project_invite_code))

        .with_state(state.clone())
}

pub(crate) async fn create_project(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Json(payload): Json<CreateProjectPayload>,
) -> Result<Json<GetProjectResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;
    match user_context {
        UserContext::MasterUser => {
            let result = state.data.create_project(&payload.name, &None).await;
            println!("{:?}", result);
            let project = result?;
            Ok(Json(project.into()))
        }
        UserContext::WebAppUser { user, .. } => {
            if user.role != ApplicationRole::Admin {
                return Err(LLMurError::NotAuthorized)
            }

            let project = state.data.create_project(&payload.name, &Some(user.id)).await?;

            Ok(Json(project.into()))
        }
    }
}

pub(crate) async fn get_project(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<ProjectId>,
) -> Result<Json<GetProjectResult>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let project = state.data.get_project(&id).await?.ok_or(LLMurError::AdminResourceNotFound)?;

    match user_context {
        UserContext::MasterUser => {
            Ok(Json(project.into()))
        }
        UserContext::WebAppUser { user, .. } => {
            /*
            if user.role == ApplicationRole::Admin || !user.memberships.is_disjoint(&project.memberships){
                return Ok(Json(ProjectData {
                    id: project.id,
                    name: project.name,
                }))
            }
             */

            Err(LLMurError::NotAuthorized)
        }
    }
}

pub(crate) async fn delete_project(
    Extension(ctx): Extension<UserContextExtractionResult>,
    State(state): State<Arc<LLMurState>>,
    Path(id): Path<ProjectId>,
) -> Result<Json<StatusResponse>, LLMurError> {
    let user_context = ctx.require_authenticated_user()?;

    let project = state.data.get_project(&id).await?.ok_or(LLMurError::AdminResourceNotFound)?; // TODO

    match user_context {
        UserContext::MasterUser => {
            let result = state.data.delete_project(&project.id).await?;
            Ok(Json(StatusResponse {
                success: result != 0,
                message: None,
            }))
        }
        UserContext::WebAppUser { user, .. } => {
            // TODO
            /*
            let maybe_membership = state.data.memberships()
                .search_memberships(&None, &Some(id))
                .await?
                .iter()
                .find(|&mem| mem.user_id == user.id)
                .cloned();

            if let Some(membership) = maybe_membership {
                if membership.role == ProjectRole::Admin {
                    let result = state.data.projects().delete_project(&id).await?;
                    return Ok(Json(StatusResponse {
                        success: result != 0,
                        message: None,
                    }))
                }
            }
            */
            Err(LLMurError::NotAuthorized)
        }
    }
}


// endregion: --- Routes

// region:    --- Data Models
#[derive(Deserialize)]
pub(crate) struct CreateProjectPayload {
    pub(crate) name: String
}

#[derive(Serialize)]
pub(crate) struct GetProjectResult {
    pub(crate) id: ProjectId,
    pub(crate) name: String,
}

#[derive(Serialize)]
pub(crate) struct ListProjectsResult {
    pub(crate) projects: Vec<GetProjectResult>,
    pub(crate) total: usize
}

impl_from_vec_result!(GetProjectResult, ListProjectsResult, projects);

impl From<Project> for GetProjectResult {
    fn from(value: Project) -> Self {
        GetProjectResult {
            id: value.id,
            name: value.name,
        }
    }
}
// endregion: --- Data Models