use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Prediction {
    #[serde(rename = "content", alias = "content")]
    StaticContent { content: PredictionStaticContent },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PredictionStaticContent {
    Text(String),
    Array(Vec<PredictionStaticContentPart>)
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PredictionStaticContentPart {
    pub text: String,
    #[serde(rename = "type")]
    pub r#type: String,
}