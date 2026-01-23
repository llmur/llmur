use serde::{Deserialize, Serialize};

use crate::providers::ExposesUsage;

/// EmbedContent response payload from Gemini.
///
/// Returns embeddings plus optional usage metadata.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Embedding>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeddings: Option<Vec<Embedding>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_metadata: Option<UsageMetadata>,
}

/// Embedding vector values.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Embedding {
    pub values: Vec<f64>,
}

/// Token usage metadata for an embeddings response.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_token_count: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_token_count: Option<u64>,
}

impl ExposesUsage for Response {
    fn get_input_tokens(&self) -> u64 {
        self.usage_metadata
            .as_ref()
            .and_then(|usage| usage.prompt_token_count)
            .unwrap_or(0)
    }

    fn get_output_tokens(&self) -> u64 {
        0
    }
}
