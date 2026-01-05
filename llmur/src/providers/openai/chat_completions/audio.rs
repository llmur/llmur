use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Audio {
    pub format: String,
    pub voice: Voice
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Voice {
    BuiltIn(String),
    Custom { id: String },
}