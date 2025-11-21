use crate::data::connection::Connection;
use crate::data::connection_deployment::ConnectionDeployment;
use crate::data::deployment::Deployment;
use crate::data::project::Project;
use crate::data::virtual_key::{VirtualKey, VirtualKeyId};
use crate::data::virtual_key_deployment::VirtualKeyDeployment;
use crate::{impl_local_store_accessors, impl_locally_stored, impl_with_id_parameter_for_struct};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

// region:    --- Auxiliary Graph Model
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize
)]
pub struct GraphDataId {
    pub(crate) model_name: String,
    pub(crate) virtual_key_id: VirtualKeyId,
}

impl GraphDataId {
    pub fn new(model_name: &str, api_key: &str) -> Self {
        GraphDataId {
            model_name: model_name.to_string(),
            virtual_key_id: VirtualKeyId::from_decrypted_key(api_key),
        }
    }
}

impl Display for GraphDataId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GraphDataId({}:{})", self.model_name, self.virtual_key_id)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct GraphData {
    pub(crate) id: GraphDataId, // For caching purposes - not stored in DB
    pub(crate) virtual_key: VirtualKey,
    pub(crate) deployment: Deployment,
    pub(crate) project: Project,
    pub(crate) virtual_key_deployment: VirtualKeyDeployment,
    pub(crate) connection_deployments: Vec<ConnectionDeployment>,
    pub(crate) connections: Vec<Connection>
}

impl_with_id_parameter_for_struct!(GraphData, GraphDataId);
impl_locally_stored!(GraphData, GraphDataId, graphs);
impl_local_store_accessors!(GraphData, GraphDataId, graph, graphs);
// endregion: --- Auxiliary Graph Model
