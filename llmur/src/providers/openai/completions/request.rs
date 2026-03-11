use crate::providers::ExposesDeployment;
use crate::providers::openai::chat_completions::request::{Stop, StreamOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Request {
    pub model: String,
    pub prompt: Prompt,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_of: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Prompt {
    Null,
    Text(String),
    TextArray(Vec<String>),
    TokenArray(Vec<i64>),
    TokenArrayBatch(Vec<Vec<i64>>),
}

impl ExposesDeployment for Request {
    fn get_deployment_ref(&self) -> &str {
        &self.model
    }
}

pub mod to_self {
    use crate::providers::openai::completions::request::Request;
    use crate::providers::{
        Transformation, TransformationContext, TransformationLoss, Transformer,
    };

    #[derive(Debug)]
    pub struct Loss {}

    #[derive(Debug)]
    pub struct Context {
        pub model: Option<String>,
    }

    impl TransformationContext<Request, Request> for Context {}
    impl TransformationLoss<Request, Request> for Loss {}

    impl Transformer<Request, Context, Loss> for Request {
        fn transform(self, context: Context) -> Transformation<Request, Loss> {
            Transformation {
                result: Request {
                    model: context.model.unwrap_or(self.model),
                    ..self
                },
                loss: Loss {},
            }
        }
    }
}
