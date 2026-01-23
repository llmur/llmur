use serde::{Deserialize, Serialize};

// region: --- Response structs
/// Azure OpenAI embeddings response payload.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Response {
    pub object: String,
    pub model: String,
    pub data: Vec<EmbeddingObject>,
    pub usage: ResponseUsage,
}

/// Embedding vector entry.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct EmbeddingObject {
    pub index: u64,
    pub object: String,
    pub embedding: Vec<f64>,
}

/// Token usage metadata for an embeddings response.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseUsage {
    pub prompt_tokens: u64,
    pub total_tokens: u64,
}
// endregion: --- Response structs
