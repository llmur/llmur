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
    pub service_tier: Option<ServiceTier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<ChatCompletionModality>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<Prediction>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_options: Option<WebSearchOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<FunctionDefinition>>,
}

impl ExposesDeployment for Request {
    fn get_deployment_ref(&self) -> &str {
        &self.model
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatCompletionModality {
    Text,
    Audio,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceTier {
    Auto,
    Default,
    Flex,
}

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
        tool_call_id: String
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
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantToolCallFunction {
    pub name: String,
    pub arguments: String,
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
#[serde(untagged)]
pub enum ToolChoice {
    Mode(ToolChoiceMode),
    Function(ToolChoiceFunction),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceMode {
    None,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<WebSearchContextSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<UserLocation>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebSearchContextSize {
    Low,
    Medium,
    High,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct UserLocation {
    #[serde(rename = "type")]
    pub kind: UserLocationType,
    pub approximate: WebSearchLocation,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserLocationType {
    Approximate,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct WebSearchLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
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
                    reasoning_effort: self.reasoning_effort,
                    store: self.store,
                    max_completion_tokens: self.max_completion_tokens,
                    web_search_options: self.web_search_options,
                    max_tokens: self.max_tokens,
                    user: self.user,
                    function_call: self.function_call,
                    functions: self.functions,
                },
                loss: Loss {},
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ChatCompletionModality, Request, UserLocationType, WebSearchContextSize,
    };

    #[test]
    fn request_file_and_user_location_shapes() {
        let json = r#"{
            "model": "gpt-4o",
            "messages": [{
                "role": "user",
                "content": [{
                    "type": "file",
                    "file": {
                        "filename": "note.txt",
                        "file_data": "ZGF0YQ=="
                    }
                }]
            }],
            "web_search_options": {
                "search_context_size": "medium",
                "user_location": {
                    "type": "approximate",
                    "approximate": {
                        "country": "US"
                    }
                }
            }
        }"#;

        let request: Request = serde_json::from_str(json).expect("parse request");
        let file_part = match &request.messages[0] {
            super::Message::UserMessage { content, .. } => match content {
                super::UserMessageContent::Array(parts) => match &parts[0] {
                    super::UserMessageContentPart::File { file } => file,
                    _ => panic!("expected file part"),
                },
                _ => panic!("expected array content"),
            },
            _ => panic!("expected user message"),
        };
        assert_eq!(file_part.filename.as_deref(), Some("note.txt"));
        assert_eq!(file_part.file_data.as_deref(), Some("ZGF0YQ=="));

        let user_location = request
            .web_search_options
            .as_ref()
            .and_then(|options| options.user_location.as_ref())
            .expect("user_location");
        assert!(matches!(user_location.kind, UserLocationType::Approximate));
        assert_eq!(user_location.approximate.country.as_deref(), Some("US"));
        assert!(matches!(
            request.web_search_options.as_ref().unwrap().search_context_size,
            Some(WebSearchContextSize::Medium)
        ));
    }

    #[test]
    fn request_modalities_serialize_lowercase() {
        let request = Request {
            model: "gpt-4o".to_string(),
            messages: vec![super::Message::UserMessage {
                name: None,
                content: super::UserMessageContent::Text("hi".to_string()),
            }],
            audio: None,
            n: None,
            frequency_penalty: None,
            temperature: None,
            logprobs: None,
            top_logprobs: None,
            max_completion_tokens: None,
            max_tokens: None,
            presence_penalty: None,
            top_p: None,
            stream: None,
            stream_options: None,
            stop: None,
            seed: None,
            response_format: None,
            logit_bias: None,
            tools: None,
            tool_choice: None,
            service_tier: None,
            metadata: None,
            modalities: Some(vec![ChatCompletionModality::Text, ChatCompletionModality::Audio]),
            parallel_tool_calls: None,
            prediction: None,
            reasoning_effort: None,
            store: None,
            user: None,
            web_search_options: None,
            function_call: None,
            functions: None,
        };

        let value = serde_json::to_value(&request).expect("serialize request");
        assert_eq!(value["modalities"][0], "text");
        assert_eq!(value["modalities"][1], "audio");
    }
}
// endregion: --- Transform methods
