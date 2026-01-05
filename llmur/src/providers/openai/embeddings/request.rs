use crate::providers::ExposesDeployment;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Request {
    pub model: String,
    pub input: EmbeddingsInput,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum EmbeddingsInput  {
    Text(String),
    Array(Vec<String>),
}

impl ExposesDeployment for Request {
    fn get_deployment_ref(&self) -> &str {
        &self.model
    }
}

pub mod to_self {
    use crate::providers::openai::embeddings::request::Request;
    use crate::providers::{Transformation, TransformationContext, TransformationLoss, Transformer};

    #[derive(Debug)]
    pub struct Loss {}

    #[derive(Debug)]
    pub struct Context { pub model: Option<String> }

    impl TransformationContext<Request, Request> for Context {}
    impl TransformationLoss<Request, Request> for Loss {}

    impl Transformer<Request, Context, Loss> for Request {
        fn transform(self, context: Context) -> Transformation<Request, Loss> {
            Transformation {
                result: Request {
                    model: context.model.unwrap_or(self.model),
                    input: self.input,
                    dimensions: self.dimensions,
                    encoding_format: self.encoding_format,
                    user: self.user,
                },
                loss: Loss {},
            }
        }
    }
}