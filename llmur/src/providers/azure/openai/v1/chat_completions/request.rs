use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::providers::azure::openai::v1::common::{
    PromptCacheRetention, ReasoningEffort, Verbosity,
};

// region: --- Request structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Request {
    pub messages: Vec<Message>,
    pub model: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<PromptCacheRetention>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<ChatCompletionModality>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<Verbosity>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<Audio>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<Prediction>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<FunctionDefinition>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_security_context: Option<UserSecurityContext>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatCompletionModality {
    Text,
    Audio,
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
        #[serde(skip_serializing_if = "Option::is_none")]
        function_call: Option<AssistantFunctionCall>,
    },

    #[serde(rename = "tool", alias = "tool")]
    ToolMessage {
        content: ToolMessageContent,
        tool_call_id: String,
    },

    #[serde(rename = "function", alias = "function")]
    FunctionMessage {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        name: String,
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
    pub id: String,
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
pub struct FileContentPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "file_name")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ImageUrlContentPart {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<ImageDetail>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    Auto,
    Low,
    High,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AudioContentPart {
    pub data: String,
    pub format: InputAudioFormat,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputAudioFormat {
    Wav,
    Mp3,
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
    },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantToolCallCustom {
    pub name: String,
    pub input: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantFunctionCall {
    pub name: String,
    pub arguments: String,
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
    JsonSchema { json_schema: ResponseJsonSchema },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseJsonSchema {
    pub name: String,
    pub description: Option<String>,
    pub schema: Option<serde_json::Value>,
    pub strict: Option<bool>,
}
// endregion: --- Format structs

// region: --- Tool structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Tool {
    Function(FunctionTool),
    Custom(CustomTool),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FunctionTool {
    #[serde(rename = "function", alias = "function")]
    Function { function: ToolFunction },
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
#[serde(tag = "type")]
pub enum CustomTool {
    #[serde(rename = "custom", alias = "custom")]
    Custom { custom: CustomToolDetails },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CustomToolDetails {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<CustomToolFormat>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CustomToolFormat {
    #[serde(rename = "text", alias = "text")]
    Text,
    #[serde(rename = "grammar", alias = "grammar")]
    Grammar { grammar: CustomToolGrammar },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CustomToolGrammar {
    pub definition: String,
    pub syntax: CustomToolGrammarSyntax,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CustomToolGrammarSyntax {
    Lark,
    Regex,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    Mode(ToolChoiceMode),
    AllowedTools(ToolChoiceAllowedTools),
    Function(ToolChoiceFunction),
    Custom(ToolChoiceCustom),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceMode {
    None,
    Auto,
    Required,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceAllowedTools {
    #[serde(rename = "type")]
    pub choice_type: ToolChoiceAllowedToolsType,
    pub allowed_tools: AllowedTools,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoiceAllowedToolsType {
    AllowedTools,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AllowedTools {
    pub mode: AllowedToolsMode,
    pub tools: Vec<Tool>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AllowedToolsMode {
    Auto,
    Required,
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolChoiceCustom {
    #[serde(rename = "custom", alias = "custom")]
    CustomTool { custom: ToolChoiceCustomDetails },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceCustomDetails {
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
    pub format: AudioFormat,
    pub voice: VoiceId,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Wav,
    Aac,
    Mp3,
    Flac,
    Opus,
    Pcm16,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VoiceId {
    Alloy,
    Ash,
    Ballad,
    Coral,
    Echo,
    Fable,
    Nova,
    Onyx,
    Sage,
    Shimmer,
    Verse,
    Marin,
    Cedar,
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
    Array(Vec<PredictionStaticContentPart>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PredictionStaticContentPart {
    pub text: String,
    #[serde(rename = "type")]
    pub r#type: String,
}
// endregion: --- Prediction structs

// region: --- Function structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionCall {
    Mode(FunctionCallMode),
    Function(FunctionCallOption),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FunctionCallMode {
    None,
    Auto,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FunctionCallOption {
    pub name: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}
// endregion: --- Function structs

// region: --- User security context
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct UserSecurityContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_user_tenant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ip: Option<String>,
}
// endregion: --- User security context

// region: --- Transform methods
pub mod from_openai_transform {
    use super::*;
    use crate::providers::openai::chat_completions::request as openai;
    use crate::providers::{
        Transformation, TransformationContext, TransformationLoss, Transformer,
    };

    #[derive(Debug)]
    pub struct Loss {}

    #[derive(Debug)]
    pub struct Context {
        pub model: Option<String>,
        pub safety_identifier: Option<String>,
        pub prompt_cache_key: Option<String>,
        pub prompt_cache_retention: Option<PromptCacheRetention>,
        pub user_security_context: Option<UserSecurityContext>,
        pub stream_include_usage: bool,
    }

    impl TransformationContext<openai::Request, Request> for Context {}
    impl TransformationLoss<openai::Request, Request> for Loss {}

    impl Transformer<Request, Context, Loss> for openai::Request {
        fn transform(self, context: Context) -> Transformation<Request, Loss> {
            Transformation {
                result: Request {
                    messages: self.messages.into_iter().map(transform_message).collect(),
                    model: context.model.unwrap_or(self.model),
                    metadata: self.metadata,
                    top_logprobs: self.top_logprobs,
                    temperature: self.temperature,
                    top_p: self.top_p,
                    user: self.user,
                    safety_identifier: context.safety_identifier,
                    prompt_cache_key: context.prompt_cache_key,
                    prompt_cache_retention: context.prompt_cache_retention,
                    modalities: self.modalities.map(|values| {
                        values
                            .into_iter()
                            .map(|modality| match modality {
                                openai::ChatCompletionModality::Text => {
                                    ChatCompletionModality::Text
                                }
                                openai::ChatCompletionModality::Audio => {
                                    ChatCompletionModality::Audio
                                }
                            })
                            .collect()
                    }),
                    verbosity: None,
                    reasoning_effort: self.reasoning_effort.map(|effort| match effort {
                        openai::ReasoningEffort::Low => ReasoningEffort::Low,
                        openai::ReasoningEffort::Medium => ReasoningEffort::Medium,
                        openai::ReasoningEffort::High => ReasoningEffort::High,
                    }),
                    max_completion_tokens: self.max_completion_tokens,
                    frequency_penalty: self.frequency_penalty,
                    presence_penalty: self.presence_penalty,
                    response_format: self.response_format.map(transform_response_format),
                    audio: self.audio.map(transform_audio),
                    store: self.store,
                    stream: self.stream,
                    stop: self.stop.map(transform_stop),
                    logit_bias: self.logit_bias,
                    logprobs: self.logprobs,
                    max_tokens: self.max_tokens,
                    n: self.n,
                    prediction: self.prediction.map(transform_prediction),
                    seed: self.seed,
                    stream_options: {
                        if self.stream.unwrap_or(false) && context.stream_include_usage {
                            let mut options =
                                self.stream_options.unwrap_or(openai::StreamOptions {
                                    include_usage: None,
                                });
                            options.include_usage = Some(true);
                            Some(transform_stream_options(options))
                        } else {
                            self.stream_options.map(transform_stream_options)
                        }
                    },
                    tools: self
                        .tools
                        .map(|tools| tools.into_iter().filter_map(transform_tool).collect()),
                    tool_choice: self.tool_choice.map(transform_tool_choice),
                    parallel_tool_calls: self.parallel_tool_calls,
                    function_call: self.function_call.map(transform_function_call),
                    functions: self.functions.map(|definitions| {
                        definitions
                            .into_iter()
                            .map(transform_function_definition)
                            .collect()
                    }),
                    user_security_context: context.user_security_context,
                },
                loss: Loss {},
            }
        }
    }

    fn transform_message(msg: openai::Message) -> Message {
        match msg {
            openai::Message::SystemMessage { content, name } => Message::SystemMessage {
                content: transform_system_message_content(content),
                name,
            },
            openai::Message::DeveloperMessage { content, name } => Message::DeveloperMessage {
                content: transform_developer_message_content(content),
                name,
            },
            openai::Message::UserMessage { name, content } => Message::UserMessage {
                content: transform_user_message_content(content),
                name,
            },
            openai::Message::AssistantMessage {
                audio,
                content,
                name,
                refusal,
                tool_calls,
                function_call,
            } => Message::AssistantMessage {
                audio: audio.and_then(transform_assistant_message_audio),
                content: content.map(transform_assistant_message_content),
                name,
                refusal,
                tool_calls: tool_calls.map(|tool_calls| {
                    tool_calls
                        .into_iter()
                        .filter_map(transform_assistant_tool_call)
                        .collect()
                }),
                function_call: function_call.map(|call| AssistantFunctionCall {
                    name: call.name,
                    arguments: call.arguments,
                }),
            },
            openai::Message::ToolMessage {
                content,
                tool_call_id,
            } => Message::ToolMessage {
                content: transform_tool_message_content(content),
                tool_call_id,
            },
            openai::Message::FunctionMessage { content, name } => {
                Message::FunctionMessage { content, name }
            }
        }
    }

    fn transform_user_message_content(content: openai::UserMessageContent) -> UserMessageContent {
        match content {
            openai::UserMessageContent::Text(s) => UserMessageContent::Text(s),
            openai::UserMessageContent::Array(parts) => UserMessageContent::Array(
                parts
                    .into_iter()
                    .filter_map(transform_user_message_content_part)
                    .collect(),
            ),
        }
    }

    fn transform_user_message_content_part(
        part: openai::UserMessageContentPart,
    ) -> Option<UserMessageContentPart> {
        match part {
            openai::UserMessageContentPart::Text { text } => {
                Some(UserMessageContentPart::Text { text })
            }
            openai::UserMessageContentPart::ImageUrl { image_url } => {
                Some(UserMessageContentPart::ImageUrl {
                    image_url: ImageUrlContentPart {
                        url: image_url.url,
                        detail: image_url.detail.map(|detail| match detail {
                            openai::ImageDetail::Auto => ImageDetail::Auto,
                            openai::ImageDetail::Low => ImageDetail::Low,
                            openai::ImageDetail::High => ImageDetail::High,
                        }),
                    },
                })
            }
            openai::UserMessageContentPart::InputAudio { input_audio } => {
                Some(UserMessageContentPart::InputAudio {
                    input_audio: AudioContentPart {
                        data: input_audio.data,
                        format: match input_audio.format {
                            openai::InputAudioFormat::Wav => InputAudioFormat::Wav,
                            openai::InputAudioFormat::Mp3 => InputAudioFormat::Mp3,
                        },
                    },
                })
            }
            openai::UserMessageContentPart::File { file } => Some(UserMessageContentPart::File {
                file: FileContentPart {
                    filename: file.filename,
                    file_data: file.file_data,
                    file_id: file.file_id,
                },
            }),
        }
    }

    fn transform_system_message_content(
        content: openai::SystemMessageContent,
    ) -> SystemMessageContent {
        match content {
            openai::SystemMessageContent::Text(s) => SystemMessageContent::Text(s),
            openai::SystemMessageContent::Array(parts) => SystemMessageContent::Array(
                parts
                    .into_iter()
                    .map(transform_system_message_content_part)
                    .collect(),
            ),
        }
    }

    fn transform_system_message_content_part(
        part: openai::SystemMessageContentPart,
    ) -> SystemMessageContentPart {
        match part {
            openai::SystemMessageContentPart::Text { text } => {
                SystemMessageContentPart::Text { text }
            }
        }
    }

    fn transform_developer_message_content(
        content: openai::DeveloperMessageContent,
    ) -> DeveloperMessageContent {
        match content {
            openai::DeveloperMessageContent::Text(s) => DeveloperMessageContent::Text(s),
            openai::DeveloperMessageContent::Array(parts) => DeveloperMessageContent::Array(
                parts
                    .into_iter()
                    .map(transform_developer_message_content_part)
                    .collect(),
            ),
        }
    }

    fn transform_developer_message_content_part(
        part: openai::DeveloperMessageContentPart,
    ) -> DeveloperMessageContentPart {
        match part {
            openai::DeveloperMessageContentPart::Text { text } => {
                DeveloperMessageContentPart::Text { text }
            }
        }
    }

    fn transform_assistant_message_content(
        content: openai::AssistantMessageContent,
    ) -> AssistantMessageContent {
        match content {
            openai::AssistantMessageContent::Text(s) => AssistantMessageContent::Text(s),
            openai::AssistantMessageContent::Array(parts) => AssistantMessageContent::Array(
                parts
                    .into_iter()
                    .map(transform_assistant_message_content_part)
                    .collect(),
            ),
        }
    }

    fn transform_assistant_message_content_part(
        part: openai::AssistantMessageContentPart,
    ) -> AssistantMessageContentPart {
        match part {
            openai::AssistantMessageContentPart::Text { text } => {
                AssistantMessageContentPart::Text { text }
            }
            openai::AssistantMessageContentPart::Refusal { text } => {
                AssistantMessageContentPart::Refusal { text }
            }
        }
    }

    fn transform_assistant_message_audio(
        audio: openai::AssistantMessageAudio,
    ) -> Option<AssistantMessageAudio> {
        serde_json::to_value(audio)
            .ok()
            .and_then(|value| serde_json::from_value(value).ok())
    }

    fn transform_assistant_tool_call(
        tool_call: openai::AssistantToolCall,
    ) -> Option<AssistantToolCall> {
        match tool_call {
            openai::AssistantToolCall::Function { id, function } => {
                Some(AssistantToolCall::Function {
                    id,
                    function: AssistantToolCallFunction {
                        name: function.name,
                        arguments: function.arguments,
                    },
                })
            }
        }
    }

    fn transform_tool_message_content(content: openai::ToolMessageContent) -> ToolMessageContent {
        match content {
            openai::ToolMessageContent::Text(s) => ToolMessageContent::Text(s),
            openai::ToolMessageContent::Array(parts) => ToolMessageContent::Array(
                parts
                    .into_iter()
                    .map(transform_tool_message_content_part)
                    .collect(),
            ),
        }
    }

    fn transform_tool_message_content_part(
        part: openai::ToolMessageContentPart,
    ) -> ToolMessageContentPart {
        match part {
            openai::ToolMessageContentPart::Text { text } => ToolMessageContentPart::Text { text },
        }
    }

    fn transform_response_format(format: openai::ResponseFormat) -> ResponseFormat {
        match format {
            openai::ResponseFormat::Text => ResponseFormat::Text,
            openai::ResponseFormat::JsonObject => ResponseFormat::JsonObject,
            openai::ResponseFormat::JsonSchema { json_schema } => ResponseFormat::JsonSchema {
                json_schema: ResponseJsonSchema {
                    name: json_schema.name,
                    description: json_schema.description,
                    schema: json_schema.schema,
                    strict: json_schema.strict,
                },
            },
        }
    }

    fn transform_audio(audio: openai::Audio) -> Audio {
        Audio {
            format: match audio.format {
                openai::AudioFormat::Wav => AudioFormat::Wav,
                openai::AudioFormat::Aac => AudioFormat::Aac,
                openai::AudioFormat::Mp3 => AudioFormat::Mp3,
                openai::AudioFormat::Flac => AudioFormat::Flac,
                openai::AudioFormat::Opus => AudioFormat::Opus,
                openai::AudioFormat::Pcm16 => AudioFormat::Pcm16,
            },
            voice: match audio.voice {
                openai::VoiceId::Alloy => VoiceId::Alloy,
                openai::VoiceId::Ash => VoiceId::Ash,
                openai::VoiceId::Ballad => VoiceId::Ballad,
                openai::VoiceId::Coral => VoiceId::Coral,
                openai::VoiceId::Echo => VoiceId::Echo,
                openai::VoiceId::Fable => VoiceId::Fable,
                openai::VoiceId::Nova => VoiceId::Nova,
                openai::VoiceId::Onyx => VoiceId::Onyx,
                openai::VoiceId::Sage => VoiceId::Sage,
                openai::VoiceId::Shimmer => VoiceId::Shimmer,
            },
        }
    }

    fn transform_stop(stop: openai::Stop) -> Stop {
        match stop {
            openai::Stop::String(s) => Stop::String(s),
            openai::Stop::Array(values) => Stop::Array(values),
        }
    }

    fn transform_prediction(prediction: openai::Prediction) -> Prediction {
        match prediction {
            openai::Prediction::StaticContent { content } => Prediction::StaticContent {
                content: match content {
                    openai::PredictionStaticContent::Text(text) => {
                        PredictionStaticContent::Text(text)
                    }
                    openai::PredictionStaticContent::Array(parts) => {
                        PredictionStaticContent::Array(
                            parts
                                .into_iter()
                                .map(|part| PredictionStaticContentPart {
                                    text: part.text,
                                    r#type: part.r#type,
                                })
                                .collect(),
                        )
                    }
                },
            },
        }
    }

    fn transform_stream_options(options: openai::StreamOptions) -> StreamOptions {
        StreamOptions {
            include_usage: options.include_usage,
            include_obfuscation: None,
        }
    }

    fn transform_tool(tool: openai::Tool) -> Option<Tool> {
        match tool {
            openai::Tool::Function { function } => Some(Tool::Function(FunctionTool::Function {
                function: ToolFunction {
                    name: function.name,
                    description: function.description,
                    parameters: function.parameters,
                    strict: function.strict,
                },
            })),
        }
    }

    fn transform_tool_choice(tool_choice: openai::ToolChoice) -> ToolChoice {
        match tool_choice {
            openai::ToolChoice::Mode(mode) => ToolChoice::Mode(match mode {
                openai::ToolChoiceMode::None => ToolChoiceMode::None,
                openai::ToolChoiceMode::Auto => ToolChoiceMode::Auto,
                openai::ToolChoiceMode::Required => ToolChoiceMode::Required,
            }),
            openai::ToolChoice::Function(tool) => match tool {
                openai::ToolChoiceFunction::FunctionTool { function } => {
                    ToolChoice::Function(ToolChoiceFunction::FunctionTool {
                        function: ToolChoiceFunctionDetails {
                            name: function.name,
                        },
                    })
                }
            },
        }
    }

    fn transform_function_call(call: openai::FunctionCall) -> FunctionCall {
        match call {
            openai::FunctionCall::Mode(mode) => FunctionCall::Mode(match mode {
                openai::FunctionCallMode::None => FunctionCallMode::None,
                openai::FunctionCallMode::Auto => FunctionCallMode::Auto,
            }),
            openai::FunctionCall::Function(openai::FunctionCallOption { name }) => {
                FunctionCall::Function(FunctionCallOption { name })
            }
        }
    }

    fn transform_function_definition(definition: openai::FunctionDefinition) -> FunctionDefinition {
        FunctionDefinition {
            name: definition.name,
            description: definition.description,
            parameters: definition.parameters,
        }
    }
}
// endregion: --- Transform methods

#[cfg(test)]
mod tests {
    use super::{Request, ToolChoice};

    #[test]
    fn request_custom_tool_and_allowed_tools_parse() {
        let json = r#"{
            "model": "gpt-4o",
            "messages": [{
                "role": "user",
                "content": [{
                    "type": "file",
                    "file": { "filename": "note.txt", "file_data": "ZGF0YQ==" }
                }]
            }],
            "tools": [{
                "type": "custom",
                "custom": {
                    "name": "validator",
                    "format": {
                        "type": "grammar",
                        "grammar": { "definition": "start: /a/", "syntax": "lark" }
                    }
                }
            }],
            "tool_choice": {
                "type": "allowed_tools",
                "allowed_tools": {
                    "mode": "auto",
                    "tools": [{ "type": "function", "function": { "name": "get_time" } }]
                }
            },
            "user_security_context": { "application_name": "app" }
        }"#;

        let request: Request = serde_json::from_str(json).expect("parse request");
        match &request.tool_choice {
            Some(ToolChoice::AllowedTools(choice)) => {
                assert_eq!(choice.allowed_tools.tools.len(), 1);
            }
            _ => panic!("expected allowed tools"),
        }

        match &request.messages[0] {
            super::Message::UserMessage { content, .. } => match content {
                super::UserMessageContent::Array(parts) => match &parts[0] {
                    super::UserMessageContentPart::File { file } => {
                        assert_eq!(file.filename.as_deref(), Some("note.txt"));
                    }
                    _ => panic!("expected file part"),
                },
                _ => panic!("expected array content"),
            },
            _ => panic!("expected user message"),
        }
    }
}
