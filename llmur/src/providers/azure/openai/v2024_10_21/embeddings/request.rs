use serde::{Deserialize, Serialize};

// region: --- Request structs
/// Azure OpenAI embeddings request payload.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Request {
    pub input: EmbeddingsInput,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u64>,
}
// endregion: --- Request structs

// region: --- Input structs
/// Embeddings input as a single string or a list of strings.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingsInput {
    Text(String),
    Array(Vec<String>),
    Null(()),
}
// endregion: --- Input structs
