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

// region: --- Transform methods
pub mod to_openai_transform {
    use super::*;
    use crate::providers::openai::embeddings::response::{
        EmbeddingObject as OpenAiEmbeddingObject, EmbeddingObjectType, Response as OpenAiResponse,
        ResponseObjectType, ResponseUsage as OpenAiResponseUsage,
    };
    use crate::providers::{Transformation, TransformationContext, TransformationLoss, Transformer};

    #[derive(Debug)]
    pub struct Loss {}

    #[derive(Debug)]
    pub struct Context {
        pub model: Option<String>,
    }

    impl TransformationContext<Response, OpenAiResponse> for Context {}
    impl TransformationLoss<Response, OpenAiResponse> for Loss {}

    impl Transformer<OpenAiResponse, Context, Loss> for Response {
        fn transform(self, context: Context) -> Transformation<OpenAiResponse, Loss> {
            Transformation {
                result: OpenAiResponse {
                    object: ResponseObjectType::List,
                    data: self
                        .data
                        .into_iter()
                        .map(|item| OpenAiEmbeddingObject {
                            embedding: item.embedding,
                            index: item.index,
                            object: EmbeddingObjectType::Embedding,
                        })
                        .collect(),
                    model: context.model.unwrap_or(self.model),
                    usage: OpenAiResponseUsage {
                        prompt_tokens: self.usage.prompt_tokens,
                        total_tokens: self.usage.total_tokens,
                    },
                },
                loss: Loss {},
            }
        }
    }
}
// endregion: --- Transform methods

#[cfg(test)]
mod tests {
    use super::to_openai_transform;
    use crate::providers::azure::openai::v2024_10_21::embeddings::response::{
        EmbeddingObject, Response, ResponseUsage,
    };
    use crate::providers::openai::embeddings::response::{EmbeddingObjectType, ResponseObjectType};
    use crate::providers::Transformer;

    #[test]
    fn transforms_azure_response_to_openai() {
        let response = Response {
            object: "list".to_string(),
            model: "azure-model".to_string(),
            data: vec![EmbeddingObject {
                index: 0,
                object: "embedding".to_string(),
                embedding: vec![0.1, 0.2],
            }],
            usage: ResponseUsage {
                prompt_tokens: 12,
                total_tokens: 12,
            },
        };

        let transformed = response.transform(to_openai_transform::Context {
            model: Some("client-model".to_string()),
        });

        assert_eq!(transformed.result.object, ResponseObjectType::List);
        assert_eq!(transformed.result.data.len(), 1);
        assert_eq!(
            transformed.result.data[0].object,
            EmbeddingObjectType::Embedding
        );
        assert_eq!(transformed.result.model, "client-model");
        assert_eq!(transformed.result.usage.prompt_tokens, 12);
        assert_eq!(transformed.result.usage.total_tokens, 12);
    }
}
