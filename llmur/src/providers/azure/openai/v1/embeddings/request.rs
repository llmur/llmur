use serde::{Deserialize, Serialize};

// region: --- Request structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Request {
    pub input: EmbeddingsInput,
    pub model: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<EncodingFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}
// endregion: --- Request structs

// region: --- Input structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingsInput {
    Text(String),
    Array(Vec<String>),
    TokenArray(Vec<i64>),
    TokenArrayBatch(Vec<Vec<i64>>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EncodingFormat {
    Float,
    Base64,
}
// endregion: --- Input structs

// region: --- Transform methods
pub mod from_openai_transform {
    use super::*;
    use crate::providers::openai::embeddings::request::{
        EmbeddingsInput as OpenAiEmbeddingsInput, EncodingFormat as OpenAiEncodingFormat,
        Request as OpenAiRequest,
    };
    use crate::providers::{
        Transformation, TransformationContext, TransformationLoss, Transformer,
    };

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
            let model = context.model.unwrap_or(self.model);
            Transformation {
                result: Request {
                    input: transform_input(self.input),
                    model: model.clone(),
                    encoding_format: self.encoding_format.map(transform_encoding_format),
                    dimensions: self.dimensions.map(|value| value.get()),
                    user: self.user,
                },
                loss: Loss { model },
            }
        }
    }

    fn transform_input(input: OpenAiEmbeddingsInput) -> EmbeddingsInput {
        match input {
            OpenAiEmbeddingsInput::Text(text) => EmbeddingsInput::Text(text),
            OpenAiEmbeddingsInput::Array(values) => EmbeddingsInput::Array(values),
            OpenAiEmbeddingsInput::TokenArray(values) => EmbeddingsInput::TokenArray(values),
            OpenAiEmbeddingsInput::TokenArrayBatch(values) => {
                EmbeddingsInput::TokenArrayBatch(values)
            }
        }
    }

    fn transform_encoding_format(format: OpenAiEncodingFormat) -> EncodingFormat {
        match format {
            OpenAiEncodingFormat::Float => EncodingFormat::Float,
            OpenAiEncodingFormat::Base64 => EncodingFormat::Base64,
        }
    }
}
// endregion: --- Transform methods

#[cfg(test)]
mod tests {
    use super::from_openai_transform;
    use crate::providers::Transformer;
    use crate::providers::azure::openai::v1::embeddings::request::{
        EmbeddingsInput, EncodingFormat, Request,
    };
    use crate::providers::openai::embeddings::request::{
        EmbeddingsInput as OpenAiEmbeddingsInput, EncodingFormat as OpenAiEncodingFormat,
        Request as OpenAiRequest,
    };
    use std::num::NonZeroU64;

    #[test]
    fn request_token_array_deserializes() {
        let json = r#"{
            "model": "text-embedding-3-small",
            "input": [1, 2, 3],
            "encoding_format": "base64",
            "dimensions": 256
        }"#;

        let request: Request = serde_json::from_str(json).expect("parse request");
        assert!(matches!(request.input, EmbeddingsInput::TokenArray(_)));
        assert!(matches!(
            request.encoding_format,
            Some(EncodingFormat::Base64)
        ));
        assert_eq!(request.dimensions, Some(256));
    }

    #[test]
    fn transforms_openai_request_to_azure_v1() {
        let request = OpenAiRequest {
            model: "text-embedding-3-small".to_string(),
            input: OpenAiEmbeddingsInput::TokenArray(vec![1, 2, 3]),
            dimensions: NonZeroU64::new(256),
            encoding_format: Some(OpenAiEncodingFormat::Float),
            user: Some("user-1".to_string()),
        };

        let transformed = request.transform(from_openai_transform::Context { model: None });
        assert_eq!(transformed.loss.model, "text-embedding-3-small");
        assert!(matches!(
            transformed.result.input,
            EmbeddingsInput::TokenArray(_)
        ));
        assert_eq!(transformed.result.dimensions, Some(256));
        assert!(matches!(
            transformed.result.encoding_format,
            Some(EncodingFormat::Float)
        ));
        assert_eq!(transformed.result.user, Some("user-1".to_string()));
    }
}
