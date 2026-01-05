use serde::{Deserialize, Serialize};
use crate::providers::ExposesUsage;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Response {
    pub object: String,
    pub data: Vec<EmbeddingObject>,
    pub model: String,
    pub usage: ResponseUsage
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct EmbeddingObject {
    pub embedding: Vec<f64>,
    pub index: u64,
    pub object: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseUsage {
    pub prompt_tokens: u64,
    pub total_tokens: u64,
}

impl ExposesUsage for Response {
    fn get_input_tokens(&self) -> u64 {
        self.usage.prompt_tokens
    }
    fn get_output_tokens(&self) -> u64 {
        0
    }
}

pub mod to_self {
    use crate::providers::openai::embeddings::response::Response;
    use crate::providers::{Transformation, TransformationContext, TransformationLoss, Transformer};

    #[derive(Debug)]
    pub struct Loss {}

    #[derive(Debug)]
    pub struct Context { pub model: Option<String> }

    impl TransformationContext<Response, Response> for Context {}
    impl TransformationLoss<Response, Response> for Loss {}

    impl Transformer<Response, Context, Loss> for Response {
        fn transform(self, context: Context) -> Transformation<Response, Loss> {
            Transformation {
                result: Response {
                    object: self.object,
                    data: self.data,
                    model: context.model.unwrap_or(self.model),
                    usage: self.usage,
                },
                loss: Loss {},
            }
        }
    }
}
