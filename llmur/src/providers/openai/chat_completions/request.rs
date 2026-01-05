use crate::providers::openai::chat_completions::message::Message;
use crate::providers::openai::chat_completions::stop::Stop;
use crate::providers::openai::chat_completions::stream::StreamOptions;
use crate::providers::openai::chat_completions::tool::{Tool, ToolChoice};
use crate::providers::ExposesDeployment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::providers::openai::chat_completions::audio::Audio;
use crate::providers::openai::chat_completions::format::ResponseFormat;
use crate::providers::openai::chat_completions::prediction::Prediction;
use crate::providers::openai::chat_completions::web_search::WebSearchOptions;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Request {
    pub model: String,
    pub messages: Vec<Message>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<Audio>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<Prediction>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_options: Option<WebSearchOptions>,
}

impl ExposesDeployment for Request {
    fn get_deployment_ref(&self) -> &str {
        &self.model
    }
}

pub mod to_self {
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
                    audio: self.audio,
                    n: self.n,
                    frequency_penalty: self.frequency_penalty,
                    temperature: self.temperature,
                    logprobs: self.logprobs,
                    top_logprobs: self.top_logprobs,
                    presence_penalty: self.presence_penalty,
                    top_p: self.top_p,
                    stream: self.stream,
                    stream_options: self.stream_options,
                    stop: self.stop,
                    seed: self.seed,
                    response_format: self.response_format,
                    logit_bias: self.logit_bias,
                    tools: self.tools,
                    tool_choice: self.tool_choice,
                    service_tier: self.service_tier,
                    metadata: self.metadata,
                    modalities: self.modalities,
                    parallel_tool_calls: self.parallel_tool_calls,
                    prediction: self.prediction,
                    prompt_cache_key: self.prompt_cache_key,
                    reasoning_effort: self.reasoning_effort,
                    safety_identifier: self.safety_identifier,
                    store: self.store,
                    verbosity: self.verbosity,
                    max_completion_tokens: self.max_completion_tokens,
                    web_search_options: self.web_search_options,
                    max_tokens: self.max_tokens,
                    user: self.user,
                },
                loss: Loss {},
            }
        }
    }
}