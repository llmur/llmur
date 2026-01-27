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

// region: --- Transform methods
pub mod to_openai_transform {
    use super::*;
    use crate::providers::openai::embeddings::response::{
        EmbeddingObject, EmbeddingObjectType, Response as OpenAiResponse, ResponseObjectType,
        ResponseUsage as OpenAiResponseUsage,
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
            let embeddings = transform_embeddings(self.embedding, self.embeddings);
            let usage = transform_usage(self.usage_metadata);

            Transformation {
                result: OpenAiResponse {
                    object: ResponseObjectType::List,
                    data: embeddings,
                    model: context.model.unwrap_or_else(|| "gemini".to_string()),
                    usage,
                },
                loss: Loss {},
            }
        }
    }

    fn transform_embeddings(
        embedding: Option<Embedding>,
        embeddings: Option<Vec<Embedding>>,
    ) -> Vec<EmbeddingObject> {
        if let Some(embeddings) = embeddings {
            return embeddings
                .into_iter()
                .enumerate()
                .map(|(index, item)| EmbeddingObject {
                    embedding: item.values,
                    index: index as u64,
                    object: EmbeddingObjectType::Embedding,
                })
                .collect();
        }

        embedding
            .map(|item| vec![EmbeddingObject {
                embedding: item.values,
                index: 0,
                object: EmbeddingObjectType::Embedding,
            }])
            .unwrap_or_default()
    }

    fn transform_usage(usage: Option<UsageMetadata>) -> OpenAiResponseUsage {
        let prompt_tokens = usage
            .as_ref()
            .and_then(|metadata| metadata.prompt_token_count)
            .unwrap_or(0);
        let total_tokens = usage
            .as_ref()
            .and_then(|metadata| metadata.total_token_count)
            .unwrap_or(prompt_tokens);

        OpenAiResponseUsage {
            prompt_tokens,
            total_tokens,
        }
    }
}
// endregion: --- Transform methods

#[cfg(test)]
mod tests {
    use super::to_openai_transform;
    use crate::providers::gemini::v1beta::embed_content::response::{
        Embedding, Response, UsageMetadata,
    };
    use crate::providers::openai::embeddings::response::{EmbeddingObjectType, ResponseObjectType};
    use crate::providers::Transformer;

    #[test]
    fn transforms_gemini_response_to_openai() {
        let response = Response {
            embedding: None,
            embeddings: Some(vec![
                Embedding {
                    values: vec![0.1, 0.2],
                },
                Embedding {
                    values: vec![0.3, 0.4],
                },
            ]),
            usage_metadata: Some(UsageMetadata {
                prompt_token_count: Some(4),
                total_token_count: Some(4),
            }),
        };

        let transformed = response.transform(to_openai_transform::Context {
            model: Some("client-model".to_string()),
        });

        assert_eq!(transformed.result.object, ResponseObjectType::List);
        assert_eq!(transformed.result.data.len(), 2);
        assert_eq!(
            transformed.result.data[0].object,
            EmbeddingObjectType::Embedding
        );
        assert_eq!(transformed.result.model, "client-model");
        assert_eq!(transformed.result.usage.prompt_tokens, 4);
        assert_eq!(transformed.result.usage.total_tokens, 4);
    }
}
