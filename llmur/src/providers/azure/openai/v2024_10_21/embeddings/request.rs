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

// region: --- Transform methods
pub mod from_openai_transform {
    use super::*;
    use crate::providers::openai::embeddings::request::{
        EmbeddingsInput as OpenAiEmbeddingsInput, EncodingFormat as OpenAiEncodingFormat,
        Request as OpenAiRequest,
    };
    use crate::providers::{Transformation, TransformationContext, TransformationLoss, Transformer};

    #[derive(Debug)]
    pub struct Loss {
        pub model: String,
    }

    #[derive(Debug)]
    pub struct Context {
        pub input_type: Option<String>,
    }

    impl TransformationContext<OpenAiRequest, Request> for Context {}
    impl TransformationLoss<OpenAiRequest, Request> for Loss {}

    impl Transformer<Request, Context, Loss> for OpenAiRequest {
        fn transform(self, context: Context) -> Transformation<Request, Loss> {
            Transformation {
                result: Request {
                    input: transform_input(self.input),
                    user: self.user,
                    input_type: context.input_type,
                    encoding_format: self.encoding_format.map(transform_encoding_format),
                    dimensions: self.dimensions.map(|value| value.get()),
                },
                loss: Loss { model: self.model },
            }
        }
    }

    fn transform_input(input: OpenAiEmbeddingsInput) -> EmbeddingsInput {
        match input {
            OpenAiEmbeddingsInput::Text(text) => EmbeddingsInput::Text(text),
            OpenAiEmbeddingsInput::Array(values) => EmbeddingsInput::Array(values),
            OpenAiEmbeddingsInput::TokenArray(_) => EmbeddingsInput::Null(()),
            OpenAiEmbeddingsInput::TokenArrayBatch(_) => EmbeddingsInput::Null(()),
        }
    }

    fn transform_encoding_format(format: OpenAiEncodingFormat) -> String {
        match format {
            OpenAiEncodingFormat::Float => "float".to_string(),
            OpenAiEncodingFormat::Base64 => "base64".to_string(),
        }
    }
}
// endregion: --- Transform methods

#[cfg(test)]
mod tests {
    use super::from_openai_transform;
    use crate::providers::azure::openai::v2024_10_21::embeddings::request::EmbeddingsInput;
    use crate::providers::openai::embeddings::request::{
        EmbeddingsInput as OpenAiEmbeddingsInput, EncodingFormat, Request as OpenAiRequest,
    };
    use crate::providers::Transformer;
    use std::num::NonZeroU64;

    #[test]
    fn transforms_openai_request_to_azure() {
        let request = OpenAiRequest {
            model: "text-embedding-3-small".to_string(),
            input: OpenAiEmbeddingsInput::Text("hello".to_string()),
            dimensions: NonZeroU64::new(256),
            encoding_format: Some(EncodingFormat::Float),
            user: Some("user-1".to_string()),
        };

        let transformed = request.transform(from_openai_transform::Context { input_type: None });
        assert_eq!(transformed.loss.model, "text-embedding-3-small");
        assert_eq!(
            transformed.result.input,
            EmbeddingsInput::Text("hello".to_string())
        );
        assert_eq!(transformed.result.dimensions, Some(256));
        assert_eq!(
            transformed.result.encoding_format,
            Some("float".to_string())
        );
        assert_eq!(transformed.result.user, Some("user-1".to_string()));
    }
}
