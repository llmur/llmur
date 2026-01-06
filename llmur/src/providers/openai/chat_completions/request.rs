use crate::providers::ExposesDeployment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// region: --- Request structs
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
// endregion: --- Request structs

// region: --- Message structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum Message {
    #[serde(rename = "system", alias = "system")]
    SystemMessage {
        content: SystemMessageContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },

    #[serde(rename = "developer", alias = "developer")]
    DeveloperMessage {
        content: DeveloperMessageContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },

    #[serde(rename = "user", alias = "user")]
    UserMessage {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        content: UserMessageContent,
    },

    #[serde(rename = "assistant", alias = "assistant")]
    AssistantMessage {
        #[serde(skip_serializing_if = "Option::is_none")]
        audio: Option<AssistantMessageAudio>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<AssistantMessageContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        refusal: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<AssistantToolCall>>,
    },

    #[serde(rename = "tool", alias = "tool")]
    ToolMessage {
        content: ToolMessageContent,
        tool_call_id: String
    },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserMessageContent {
    Text(String),
    Array(Vec<UserMessageContentPart>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeveloperMessageContent {
    Text(String),
    Array(Vec<DeveloperMessageContentPart>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemMessageContent {
    Text(String),
    Array(Vec<SystemMessageContentPart>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolMessageContent {
    Text(String),
    Array(Vec<ToolMessageContentPart>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AssistantMessageContent {
    Text(String),
    Array(Vec<AssistantMessageContentPart>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AssistantMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
    #[serde(rename = "refusal", alias = "refusal")]
    Refusal { text: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SystemMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DeveloperMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantMessageAudio {
    id: String
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UserMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
    #[serde(rename = "image_url", alias = "image_url")]
    ImageUrl { image_url: ImageUrlContentPart },
    #[serde(rename = "input_audio", alias = "input_audio")]
    InputAudio { input_audio: AudioContentPart },
    #[serde(rename = "file", alias = "file")]
    File { file: FileContentPart },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum FileContentPart {
    #[serde(rename = "file_data", alias = "file_data")]
    FileData(String),
    #[serde(rename = "file_id", alias = "file_id")]
    FileId(String),
    #[serde(rename = "file_name", alias = "file_name")]
    FileName(String),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ImageUrlContentPart {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AudioContentPart {
    pub data: String,
    pub format: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AssistantToolCall {
    #[serde(rename = "function", alias = "function")]
    Function {
        id: String,
        function: AssistantToolCallFunction,
    },
    #[serde(rename = "custom", alias = "custom")]
    Custom {
        id: String,
        custom: AssistantToolCallCustom,
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum AssistantToolCallType {
    #[serde(rename = "function")]
    FunctionType,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantToolCallCustom {
    pub input: String,
    pub name: String,
}
// endregion: --- Message structs

// region: --- Format structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseFormat {
    #[serde(rename = "text", alias = "text")]
    Text,
    #[serde(rename = "json_object", alias = "json_object")]
    JsonObject,
    #[serde(rename = "json_schema", alias = "json_schema")]
    JsonSchema {json_schema: ResponseJsonSchema},
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseJsonSchema {
    pub name: String,
    pub description: Option<String>,
    pub schema: Option<serde_json::Value>,
    pub strict: Option<bool>
}
// endregion: --- Format structs

// region: --- Tool structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Tool {
    #[serde(rename = "function", alias = "function")]
    Function { function: ToolFunction },
    #[serde(rename = "custom", alias = "custom")]
    Custom { custom: ToolCustom },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolCustom {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ToolCustomFormat>
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolCustomFormat {
    #[serde(rename = "text", alias = "text")]
    Text,
    #[serde(rename = "grammar", alias = "grammar")]
    Grammar { grammar: GrammarFormat },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct GrammarFormat {
    definition: String,
    syntax: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    String(String),
    Function(ToolChoiceFunction),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolChoiceFunction {
    #[serde(rename = "function", alias = "function")]
    FunctionTool { function: ToolChoiceFunctionDetails },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceFunctionDetails {
    pub name: String,
}
// endregion: --- Tool structs

// region: --- Stop structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Stop {
    String(String),
    Array(Vec<String>),
}
// endregion: --- Stop structs

// region: --- Stream structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_obfuscation: Option<bool>,
}
// endregion: --- Stream structs

// region: --- Audio structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Audio {
    pub format: String,
    pub voice: Voice
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Voice {
    BuiltIn(String),
    Custom { id: String },
}
// endregion: --- Audio structs

// region: --- Prediction structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Prediction {
    #[serde(rename = "content", alias = "content")]
    StaticContent { content: PredictionStaticContent },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PredictionStaticContent {
    Text(String),
    Array(Vec<PredictionStaticContentPart>)
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PredictionStaticContentPart {
    pub text: String,
    #[serde(rename = "type")]
    pub r#type: String,
}
// endregion: --- Prediction structs

// region: --- WebSearch structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct WebSearchOptions {
    pub search_context_size: Option<SearchContextSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<UserLocation>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum SearchContextSize {
    Low,
    Medium,
    High,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UserLocation {
    #[serde(rename = "approximate", alias = "approximate")]
    Approximate {
        city: Option<String>,
        country: Option<String>,
        region: Option<String>,
        timezone: Option<String>,
    }
}
// endregion: --- WebSearch structs

// region: --- Transform methods
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
// endregion: --- Transform methods
