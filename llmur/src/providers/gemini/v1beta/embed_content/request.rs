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

// region: --- Transform methods
pub mod from_openai_transform {
    use super::*;
    use crate::providers::openai::embeddings::request::{
        EmbeddingsInput as OpenAiEmbeddingsInput, Request as OpenAiRequest,
    };
    use crate::providers::{Transformation, TransformationContext, TransformationLoss, Transformer};

    #[derive(Debug)]
    pub struct Loss {
        pub model: String,
    }

    #[derive(Debug)]
    pub struct Context {
        pub model: Option<String>,
    }

    impl TransformationContext<OpenAiRequest, Request> for Context {}
    impl TransformationLoss<OpenAiRequest, Request> for Loss {}

    impl Transformer<Request, Context, Loss> for OpenAiRequest {
        fn transform(self, context: Context) -> Transformation<Request, Loss> {
            Transformation {
                result: Request {
                    model: None,
                    content: Content {
                        role: None,
                        parts: transform_input(self.input),
                    },
                    task_type: None,
                    title: None,
                    output_dimensionality: self.dimensions.map(|value| value.get()),
                },
                loss: Loss {
                    model: context.model.unwrap_or(self.model),
                },
            }
        }
    }

    fn transform_input(input: OpenAiEmbeddingsInput) -> Vec<Part> {
        match input {
            OpenAiEmbeddingsInput::Text(text) => vec![Part { text: Some(text) }],
            OpenAiEmbeddingsInput::Array(values) => values
                .into_iter()
                .map(|value| Part { text: Some(value) })
                .collect(),
            OpenAiEmbeddingsInput::TokenArray(_) => Vec::new(),
            OpenAiEmbeddingsInput::TokenArrayBatch(_) => Vec::new(),
        }
    }
}
// endregion: --- Transform methods

#[cfg(test)]
mod tests {
    use super::from_openai_transform;
    use crate::providers::openai::embeddings::request::{
        EmbeddingsInput as OpenAiEmbeddingsInput, Request as OpenAiRequest,
    };
    use crate::providers::Transformer;
    use std::num::NonZeroU64;

    #[test]
    fn transforms_openai_request_to_gemini() {
        let request = OpenAiRequest {
            model: "text-embedding-3-small".to_string(),
            input: OpenAiEmbeddingsInput::Array(vec![
                "first".to_string(),
                "second".to_string(),
            ]),
            dimensions: NonZeroU64::new(768),
            encoding_format: None,
            user: None,
        };

        let transformed = request.transform(from_openai_transform::Context {
            model: Some("gemini-embedding-1".to_string()),
        });

        assert_eq!(transformed.loss.model, "gemini-embedding-1");
        assert_eq!(transformed.result.content.parts.len(), 2);
        assert_eq!(transformed.result.output_dimensionality, Some(768));
    }
}
