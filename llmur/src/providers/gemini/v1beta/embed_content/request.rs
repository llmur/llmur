use serde::{Deserialize, Serialize};

// region: --- Request structs
/// EmbedContent request payload for Gemini.
///
/// Defines the text content to embed along with optional task tuning and
/// output dimensionality controls.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    pub content: Content,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<TaskType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_dimensionality: Option<u64>,
}
// endregion: --- Request structs

// region: --- Content structs
/// Text content to embed.
///
/// Embedding requests accept one or more text parts.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub parts: Vec<Part>,
}

/// Content part containing plain text.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}
// endregion: --- Content structs

// region: --- Task tuning enums
/// Task type for embedding optimization.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskType {
    TaskTypeUnspecified,
    SemanticSimilarity,
    Classification,
    Clustering,
    RetrievalDocument,
    RetrievalQuery,
    CodeRetrievalQuery,
    QuestionAnswering,
    FactVerification,
}
// endregion: --- Task tuning enums
