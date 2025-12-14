use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::providers::ExposesDeployment;
use crate::providers::openai::chat_completions::message::Message;
use crate::providers::openai::chat_completions::stop::Stop;
use crate::providers::openai::chat_completions::stream::StreamOptions;
use crate::providers::openai::chat_completions::tool::{Tool, ToolChoice};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Request {
    pub model: String,
    pub messages: Vec<Message>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<serde_json::Value>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>, //
}

impl ExposesDeployment for Request {
    fn get_deployment_ref(&self) -> &str {
        &self.model
    }
}

pub mod to_self {
    use serde::Deserialize;
    use crate::providers::openai::chat_completions::request::Request;
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
                    messages: self.messages,
                    n: self.n,
                    frequency_penalty: self.frequency_penalty,
                    temperature: self.temperature,
                    logprobs: self.logprobs,
                    top_logprobs: self.top_logprobs,
                    max_tokens: self.max_tokens,
                    presence_penalty: self.presence_penalty,
                    top_p: self.top_p,
                    stream: self.stream,
                    stream_options: self.stream_options,
                    stop: self.stop,
                    user: self.user,
                    seed: self.seed,
                    response_format: self.response_format,
                    logit_bias: self.logit_bias,
                    tools: self.tools,
                    tool_choice: self.tool_choice,
                    service_tier: self.service_tier,
                },
                loss: Loss {},
            }
        }
    }
}