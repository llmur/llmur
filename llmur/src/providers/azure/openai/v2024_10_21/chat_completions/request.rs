use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::providers::azure::openai::v2024_10_21::chat_completions::extension::AzureChatExtensionConfiguration;
use crate::providers::azure::openai::v2024_10_21::chat_completions::format::ResponseFormat;
use crate::providers::azure::openai::v2024_10_21::chat_completions::message::Message;
use crate::providers::azure::openai::v2024_10_21::chat_completions::stop::Stop;
use crate::providers::azure::openai::v2024_10_21::chat_completions::stream::StreamOptions;
use crate::providers::azure::openai::v2024_10_21::chat_completions::tool::{Tool, ToolChoice};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Request {
    pub messages: Vec<Message>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Up to 4 sequences where the API will stop generating further tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_sources: Option<Vec<AzureChatExtensionConfiguration>>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u64>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>, //

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,
}

pub mod from_openai_transform {
    use crate::providers::azure::openai::v2024_10_21::chat_completions::request::{Request as AzureRequest};
    use crate::providers::openai::chat_completions::request::{Request as OpenAiRequest};
    use crate::providers::{Transformation, TransformationContext, TransformationLoss, Transformer};
    use crate::providers::azure::openai::v2024_10_21::chat_completions::extension::AzureChatExtensionConfiguration;
    
    #[derive(Debug)]
    pub struct Loss {
        pub model: String,
    }
    
    #[derive(Debug)]
    pub struct Context {
        pub data_sources: Option<Vec<AzureChatExtensionConfiguration>>,
    }

    impl TransformationContext<OpenAiRequest, AzureRequest> for Context {}
    impl TransformationLoss<OpenAiRequest, AzureRequest> for Loss {}

    impl Transformer<AzureRequest, Context, Loss> for OpenAiRequest {
        fn transform(self, context: Context) -> Transformation<AzureRequest, Loss> {
            Transformation {
                result: AzureRequest {
                    messages: self.messages.into_iter().map(Into::into).collect(),
                    temperature: self.temperature,
                    top_p: self.top_p,
                    stream: self.stream,
                    max_tokens: self.max_tokens,
                    max_completion_tokens: self.max_completion_tokens,
                    presence_penalty: self.presence_penalty,
                    frequency_penalty: self.frequency_penalty,
                    logit_bias: self.logit_bias,
                    user: self.user,
                    n: self.n,
                    seed: self.seed,
                    response_format: self.response_format.map(Into::into),
                    tools: self.tools.map(|values| values
                        .into_iter()
                        .filter_map(|part| part.try_into().ok())
                        .collect()
                    ),
                    tool_choice: self.tool_choice.map(Into::into),
                    stop: self.stop.map(Into::into),
                    data_sources: context.data_sources,

                    // TODO
                    logprobs: self.logprobs,
                    top_logprobs: self.top_logprobs,
                    parallel_tool_calls: self.parallel_tool_calls,
                    stream_options: self.stream_options.map(Into::into),
                },
                loss: Loss { model: self.model },
            }
        }
    }
}