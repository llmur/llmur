use serde::{Deserialize, Serialize};
use serde_json::Value;

// region: --- Request structs
/// GenerateContent request payload for Gemini.
///
/// Defines the current conversation contents plus optional tools, safety settings,
/// system instruction, generation config, and cached context.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub contents: Vec<Content>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<ToolConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<Vec<SafetySetting>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_content: Option<String>,
}
// endregion: --- Request structs

// region: --- Content structs
/// Conversation content sent to Gemini.
///
/// For single-turn prompts, this is one content item. For multi-turn prompts,
/// provide the history and latest request as repeated entries.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub parts: Vec<Part>,
}

/// Content part (text, media, or tool/function artifacts).
///
/// Each part may contain text, inline media, file references, or tool/function
/// call artifacts depending on the request mode.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "inline_data")]
    pub inline_data: Option<InlineData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "file_data")]
    pub file_data: Option<FileData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "function_call")]
    pub function_call: Option<FunctionCall>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "function_response")]
    pub function_response: Option<FunctionResponse>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "executable_code")]
    pub executable_code: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "code_execution_result")]
    pub code_execution_result: Option<Value>,
}

/// Inline media payload (base64 data + MIME type).
///
/// Use for directly embedding binary data in the request.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineData {
    #[serde(alias = "mime_type")]
    pub mime_type: String,
    pub data: String,
}

/// Reference to uploaded media.
///
/// Points at an uploaded file URI plus its MIME type.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileData {
    #[serde(alias = "mime_type")]
    pub mime_type: String,
    #[serde(alias = "file_uri")]
    pub file_uri: String,
}

/// Function call emitted by the model.
///
/// Contains the function name and structured arguments.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCall {
    pub name: String,
    pub args: Value,
}

/// Function response payload returned to the model.
///
/// Returns tool execution results back to the model.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionResponse {
    pub name: String,
    pub response: Value,
}
// endregion: --- Content structs

// region: --- Tool structs
/// Tool definitions for the model.
///
/// Tools enable the model to call external systems or run code.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_declarations: Option<Vec<FunctionDeclaration>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_execution: Option<CodeExecutionTool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search_retrieval: Option<Value>,
}

/// Code execution tool toggle (empty object when enabled).
///
/// Presence enables code execution tool support.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CodeExecutionTool {}

/// Function declaration exposed to the model.
///
/// Declares a callable function name, description, and JSON schema parameters.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionDeclaration {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
}

/// Configuration for tool calling behavior.
///
/// Controls how the model chooses and invokes tools.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_calling_config: Option<FunctionCallingConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_execution_config: Option<Value>,
}

/// Function calling mode and allowlist.
///
/// Mode selects the calling strategy and allowed functions restrict availability.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCallingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_function_names: Option<Vec<String>>,
}
// endregion: --- Tool structs

// region: --- Safety structs
/// Safety setting overrides for a harm category.
///
/// Overrides default safety thresholds for the specified category.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetySetting {
    pub category: HarmCategory,
    pub threshold: HarmBlockThreshold,
}

/// Harm category used in safety settings.
///
/// Categories cover various kinds of harms that can be blocked or allowed.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmCategory {
    HarmCategoryUnspecified,
    HarmCategoryDerogatory,
    HarmCategoryToxicity,
    HarmCategoryViolence,
    HarmCategorySexual,
    HarmCategoryMedical,
    HarmCategoryDangerous,
    HarmCategoryHarassment,
    HarmCategoryHateSpeech,
    HarmCategorySexuallyExplicit,
    HarmCategoryDangerousContent,
    HarmCategoryCivicIntegrity,
}

/// Blocking threshold for a safety category.
///
/// Controls at which probability a category is blocked.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmBlockThreshold {
    HarmBlockThresholdUnspecified,
    BlockLowAndAbove,
    BlockMediumAndAbove,
    BlockOnlyHigh,
    BlockNone,
    Off,
}
// endregion: --- Safety structs

// region: --- Generation config
/// Generation options for the model output.
///
/// Controls output modalities, decoding parameters, schema/mime type, and
/// optional speech/thinking/image configuration.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mime_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<Value>,

    #[serde(rename = "_responseJsonSchema")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_json_schema_proto: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_json_schema: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_modalities: Option<Vec<ResponseModality>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_count: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_logprobs: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_enhanced_civic_answers: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech_config: Option<SpeechConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_config: Option<ThinkingConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_config: Option<ImageConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_resolution: Option<MediaResolution>,
}

/// Requested response modalities.
///
/// Empty list is equivalent to requesting only text.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResponseModality {
    ModalityUnspecified,
    Text,
    Image,
    Audio,
}

/// Speech synthesis configuration.
///
/// Configure single-voice or multi-speaker speech output and language.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_config: Option<VoiceConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_speaker_voice_config: Option<MultiSpeakerVoiceConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

/// Voice configuration for speech output.
///
/// Union wrapper for voice configuration types.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prebuilt_voice_config: Option<PrebuiltVoiceConfig>,
}

/// Prebuilt voice selection.
///
/// Selects a preset voice by name.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrebuiltVoiceConfig {
    pub voice_name: String,
}

/// Multi-speaker configuration for speech output.
///
/// Provides a list of named speakers with voice configs.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSpeakerVoiceConfig {
    pub speaker_voice_configs: Vec<SpeakerVoiceConfig>,
}

/// Voice config for a named speaker.
///
/// Speaker name must match the prompt's speaker identifiers.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerVoiceConfig {
    pub speaker: String,
    pub voice_config: VoiceConfig,
}

/// Thinking feature configuration.
///
/// Controls whether thoughts are included, budget, or a fixed level.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThinkingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_thoughts: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_budget: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<ThinkingLevel>,
}

/// Thinking intensity level.
///
/// Higher levels allow deeper internal reasoning before responding.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThinkingLevel {
    ThinkingLevelUnspecified,
    Minimal,
    Low,
    Medium,
    High,
}

/// Image generation configuration.
///
/// Controls aspect ratio and output size for generated images.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_size: Option<String>,
}

/// Media resolution for input media.
///
/// Indicates how input media should be processed (low/medium/high).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaResolution {
    MediaResolutionUnspecified,
    MediaResolutionLow,
    MediaResolutionMedium,
    MediaResolutionHigh,
}
// endregion: --- Generation config

// region: --- Transform methods
pub mod from_openai_transform {
    use super::*;
    use crate::providers::openai::chat_completions::request::{
        Request as OpenAiRequest,
        Message as OpenAiMessage,
        UserMessageContent as OpenAiUserMessageContent,
        UserMessageContentPart as OpenAiUserMessageContentPart,
        ImageUrlContentPart as OpenAiImageUrlContentPart,
        SystemMessageContent as OpenAiSystemMessageContent,
        SystemMessageContentPart as OpenAiSystemMessageContentPart,
        DeveloperMessageContent as OpenAiDeveloperMessageContent,
        DeveloperMessageContentPart as OpenAiDeveloperMessageContentPart,
        AssistantMessageContent as OpenAiAssistantMessageContent,
        AssistantMessageContentPart as OpenAiAssistantMessageContentPart,
        ToolMessageContent as OpenAiToolMessageContent,
        ToolMessageContentPart as OpenAiToolMessageContentPart,
        AssistantToolCall as OpenAiAssistantToolCall,
        AssistantToolCallFunction as OpenAiAssistantToolCallFunction,
        AssistantFunctionCall as OpenAiAssistantFunctionCall,
        ResponseFormat as OpenAiResponseFormat,
        ResponseJsonSchema as OpenAiResponseJsonSchema,
        Tool as OpenAiTool,
        ToolFunction as OpenAiToolFunction,
        ToolChoice as OpenAiToolChoice,
        ToolChoiceMode as OpenAiToolChoiceMode,
        ToolChoiceFunction as OpenAiToolChoiceFunction,
        Stop as OpenAiStop,
        FunctionCall as OpenAiFunctionCall,
        FunctionCallMode as OpenAiFunctionCallMode,
        FunctionCallOption as OpenAiFunctionCallOption,
        FunctionDefinition as OpenAiFunctionDefinition,
        ChatCompletionModality as OpenAiChatCompletionModality,
    };
    use crate::providers::{Transformation, TransformationContext, TransformationLoss, Transformer};
    use serde_json::Value;

    #[derive(Debug)]
    pub struct Loss {
        pub model: String,
    }

    #[derive(Debug)]
    pub struct Context {
        pub model: Option<String>,
    }

    impl TransformationContext<OpenAiRequest, Request> for Context {}
    impl TransformationLoss<OpenAiRequest, Request> for Loss {}

    impl Transformer<Request, Context, Loss> for OpenAiRequest {
        fn transform(self, context: Context) -> Transformation<Request, Loss> {
            let generation_config = transform_generation_config(&self);
            let (system_instruction, contents) = transform_messages(self.messages);
            let (tools, tool_config) =
                transform_tools_and_config(self.tools, self.tool_choice, self.function_call, self.functions);

            Transformation {
                result: Request {
                    contents,
                    tools,
                    tool_config,
                    safety_settings: None,
                    system_instruction,
                    generation_config,
                    cached_content: None,
                },
                loss: Loss {
                    model: context.model.unwrap_or(self.model),
                },
            }
        }
    }

    fn transform_messages(messages: Vec<OpenAiMessage>) -> (Option<Content>, Vec<Content>) {
        let mut system_parts: Vec<Part> = Vec::new();
        let mut contents: Vec<Content> = Vec::new();

        for message in messages {
            match message {
                OpenAiMessage::SystemMessage { content, .. } => {
                    system_parts.extend(transform_system_message_content(content));
                }
                OpenAiMessage::DeveloperMessage { content, .. } => {
                    system_parts.extend(transform_developer_message_content(content));
                }
                OpenAiMessage::UserMessage { content, .. } => {
                    contents.push(Content {
                        role: Some("user".to_string()),
                        parts: transform_user_message_content(content),
                    });
                }
                OpenAiMessage::AssistantMessage {
                    content,
                    tool_calls,
                    function_call,
                    ..
                } => {
                    let mut parts: Vec<Part> = Vec::new();
                    if let Some(content) = content {
                        parts.extend(transform_assistant_message_content(content));
                    }
                    if let Some(tool_calls) = tool_calls {
                        parts.extend(
                            tool_calls
                                .into_iter()
                                .map(transform_assistant_tool_call),
                        );
                    }
                    if let Some(function_call) = function_call {
                        parts.push(transform_assistant_function_call_deprecated(function_call));
                    }
                    contents.push(Content {
                        role: Some("model".to_string()),
                        parts,
                    });
                }
                OpenAiMessage::ToolMessage { content, tool_call_id } => {
                    contents.push(Content {
                        role: Some("user".to_string()),
                        parts: vec![transform_tool_message_content(content, tool_call_id)],
                    });
                }
                OpenAiMessage::FunctionMessage { content, name } => {
                    contents.push(Content {
                        role: Some("user".to_string()),
                        parts: vec![transform_function_message_content(content, name)],
                    });
                }
            }
        }

        let system_instruction = if system_parts.is_empty() {
            None
        } else {
            Some(Content {
                role: Some("system".to_string()),
                parts: system_parts,
            })
        };

        (system_instruction, contents)
    }

    fn transform_system_message_content(content: OpenAiSystemMessageContent) -> Vec<Part> {
        match content {
            OpenAiSystemMessageContent::Text(text) => vec![text_part(text)],
            OpenAiSystemMessageContent::Array(parts) => parts
                .into_iter()
                .map(transform_system_message_content_part)
                .collect(),
        }
    }

    fn transform_system_message_content_part(part: OpenAiSystemMessageContentPart) -> Part {
        match part {
            OpenAiSystemMessageContentPart::Text { text } => text_part(text),
        }
    }

    fn transform_developer_message_content(content: OpenAiDeveloperMessageContent) -> Vec<Part> {
        match content {
            OpenAiDeveloperMessageContent::Text(text) => vec![text_part(text)],
            OpenAiDeveloperMessageContent::Array(parts) => parts
                .into_iter()
                .map(transform_developer_message_content_part)
                .collect(),
        }
    }

    fn transform_developer_message_content_part(part: OpenAiDeveloperMessageContentPart) -> Part {
        match part {
            OpenAiDeveloperMessageContentPart::Text { text } => text_part(text),
        }
    }

    fn transform_user_message_content(content: OpenAiUserMessageContent) -> Vec<Part> {
        match content {
            OpenAiUserMessageContent::Text(text) => vec![text_part(text)],
            OpenAiUserMessageContent::Array(parts) => parts
                .into_iter()
                .filter_map(transform_user_message_content_part)
                .collect(),
        }
    }

    fn transform_user_message_content_part(part: OpenAiUserMessageContentPart) -> Option<Part> {
        match part {
            OpenAiUserMessageContentPart::Text { text } => Some(text_part(text)),
            OpenAiUserMessageContentPart::ImageUrl { image_url } => transform_image_url_part(image_url),
            OpenAiUserMessageContentPart::InputAudio { .. } => None,
            OpenAiUserMessageContentPart::File { .. } => None,
        }
    }

    fn transform_image_url_part(image_url: OpenAiImageUrlContentPart) -> Option<Part> {
        if let Some(inline_data) = parse_data_url(&image_url.url) {
            Some(Part {
                text: None,
                inline_data: Some(inline_data),
                file_data: None,
                function_call: None,
                function_response: None,
                executable_code: None,
                code_execution_result: None,
            })
        } else if let Some(mime_type) = guess_mime_type(&image_url.url) {
            Some(Part {
                text: None,
                inline_data: None,
                file_data: Some(FileData {
                    mime_type,
                    file_uri: image_url.url,
                }),
                function_call: None,
                function_response: None,
                executable_code: None,
                code_execution_result: None,
            })
        } else {
            None
        }
    }

    fn transform_assistant_message_content(content: OpenAiAssistantMessageContent) -> Vec<Part> {
        match content {
            OpenAiAssistantMessageContent::Text(text) => vec![text_part(text)],
            OpenAiAssistantMessageContent::Array(parts) => parts
                .into_iter()
                .filter_map(transform_assistant_message_content_part)
                .collect(),
        }
    }

    fn transform_assistant_message_content_part(part: OpenAiAssistantMessageContentPart) -> Option<Part> {
        match part {
            OpenAiAssistantMessageContentPart::Text { text } => Some(text_part(text)),
            OpenAiAssistantMessageContentPart::Refusal { .. } => None,
        }
    }

    fn transform_assistant_tool_call(tool_call: OpenAiAssistantToolCall) -> Part {
        match tool_call {
            OpenAiAssistantToolCall::Function { function, .. } => {
                transform_assistant_function_call(function)
            }
        }
    }

    fn transform_assistant_function_call(function_call: OpenAiAssistantToolCallFunction) -> Part {
        let args = serde_json::from_str::<Value>(&function_call.arguments)
            .unwrap_or_else(|_| Value::String(function_call.arguments));
        Part {
            text: None,
            inline_data: None,
            file_data: None,
            function_call: Some(FunctionCall {
                name: function_call.name,
                args,
            }),
            function_response: None,
            executable_code: None,
            code_execution_result: None,
        }
    }

    fn transform_assistant_function_call_deprecated(function_call: OpenAiAssistantFunctionCall) -> Part {
        let args = serde_json::from_str::<Value>(&function_call.arguments)
            .unwrap_or_else(|_| Value::String(function_call.arguments));
        Part {
            text: None,
            inline_data: None,
            file_data: None,
            function_call: Some(FunctionCall {
                name: function_call.name,
                args,
            }),
            function_response: None,
            executable_code: None,
            code_execution_result: None,
        }
    }

    fn transform_tool_message_content(content: OpenAiToolMessageContent, tool_call_id: String) -> Part {
        let text = match content {
            OpenAiToolMessageContent::Text(text) => text,
            OpenAiToolMessageContent::Array(parts) => parts
                .into_iter()
                .map(transform_tool_message_content_part)
                .collect::<Vec<String>>()
                .join(""),
        };
        let response = serde_json::from_str::<Value>(&text).unwrap_or(Value::String(text));
        Part {
            text: None,
            inline_data: None,
            file_data: None,
            function_call: None,
            function_response: Some(FunctionResponse {
                name: tool_call_id,
                response,
            }),
            executable_code: None,
            code_execution_result: None,
        }
    }

    fn transform_tool_message_content_part(part: OpenAiToolMessageContentPart) -> String {
        match part {
            OpenAiToolMessageContentPart::Text { text } => text,
        }
    }

    fn transform_function_message_content(content: Option<String>, name: String) -> Part {
        let text = content.unwrap_or_default();
        let response = serde_json::from_str::<Value>(&text).unwrap_or(Value::String(text));
        Part {
            text: None,
            inline_data: None,
            file_data: None,
            function_call: None,
            function_response: Some(FunctionResponse { name, response }),
            executable_code: None,
            code_execution_result: None,
        }
    }

    fn transform_tools_and_config(
        tools: Option<Vec<OpenAiTool>>,
        tool_choice: Option<OpenAiToolChoice>,
        function_call: Option<OpenAiFunctionCall>,
        functions: Option<Vec<OpenAiFunctionDefinition>>,
    ) -> (Option<Vec<Tool>>, Option<ToolConfig>) {
        let mut declarations: Vec<FunctionDeclaration> = Vec::new();

        if let Some(tools) = tools {
            for tool in tools {
                let OpenAiTool::Function { function } = tool;
                declarations.push(transform_tool_function(function));
            }
        }

        if let Some(functions) = functions {
            for function in functions {
                declarations.push(transform_function_definition(function));
            }
        }

        let tools = if declarations.is_empty() {
            None
        } else {
            Some(vec![Tool {
                function_declarations: Some(declarations),
                code_execution: None,
                google_search: None,
                google_search_retrieval: None,
            }])
        };

        let tool_config = tool_choice
            .map(transform_tool_choice)
            .or_else(|| function_call.map(transform_function_call))
            .map(|config| ToolConfig {
                function_calling_config: Some(config),
                code_execution_config: None,
            });

        (tools, tool_config)
    }

    fn transform_tool_function(tool: OpenAiToolFunction) -> FunctionDeclaration {
        FunctionDeclaration {
            name: tool.name,
            description: tool.description,
            parameters: tool.parameters,
        }
    }

    fn transform_function_definition(definition: OpenAiFunctionDefinition) -> FunctionDeclaration {
        FunctionDeclaration {
            name: definition.name,
            description: definition.description,
            parameters: definition.parameters,
        }
    }

    fn transform_tool_choice(tool_choice: OpenAiToolChoice) -> FunctionCallingConfig {
        match tool_choice {
            OpenAiToolChoice::Mode(mode) => FunctionCallingConfig {
                mode: Some(match mode {
                    OpenAiToolChoiceMode::None => "NONE".to_string(),
                    OpenAiToolChoiceMode::Auto => "AUTO".to_string(),
                    OpenAiToolChoiceMode::Required => "ANY".to_string(),
                }),
                allowed_function_names: None,
            },
            OpenAiToolChoice::Function(function) => match function {
                OpenAiToolChoiceFunction::FunctionTool { function } => FunctionCallingConfig {
                    mode: Some("ANY".to_string()),
                    allowed_function_names: Some(vec![function.name]),
                },
            },
        }
    }

    fn transform_function_call(function_call: OpenAiFunctionCall) -> FunctionCallingConfig {
        match function_call {
            OpenAiFunctionCall::Mode(mode) => FunctionCallingConfig {
                mode: Some(match mode {
                    OpenAiFunctionCallMode::None => "NONE".to_string(),
                    OpenAiFunctionCallMode::Auto => "AUTO".to_string(),
                }),
                allowed_function_names: None,
            },
            OpenAiFunctionCall::Function(OpenAiFunctionCallOption { name }) => FunctionCallingConfig {
                mode: Some("ANY".to_string()),
                allowed_function_names: Some(vec![name]),
            },
        }
    }

    fn transform_generation_config(request: &OpenAiRequest) -> Option<GenerationConfig> {
        let stop_sequences = request.stop.clone().map(transform_stop);
        let response_config = request.response_format.clone().map(transform_response_format);
        let max_output_tokens = request
            .max_completion_tokens
            .or(request.max_tokens)
            .map(|value| value as u64);
        let response_modalities = request.modalities.as_ref().map(transform_modalities);

        if stop_sequences.is_none()
            && response_config.is_none()
            && request.n.is_none()
            && max_output_tokens.is_none()
            && request.temperature.is_none()
            && request.top_p.is_none()
            && request.seed.is_none()
            && request.presence_penalty.is_none()
            && request.frequency_penalty.is_none()
            && request.logprobs.is_none()
            && request.top_logprobs.is_none()
            && response_modalities.is_none()
        {
            return None;
        }

        let (response_mime_type, response_schema) = response_config.unwrap_or((None, None));

        Some(GenerationConfig {
            stop_sequences,
            response_mime_type,
            response_schema,
            response_json_schema_proto: None,
            response_json_schema: None,
            response_modalities,
            candidate_count: request.n.map(|value| value as u64),
            max_output_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            top_k: None,
            seed: request.seed,
            presence_penalty: request.presence_penalty,
            frequency_penalty: request.frequency_penalty,
            response_logprobs: request.logprobs,
            logprobs: request.top_logprobs.map(|value| value as u64),
            enable_enhanced_civic_answers: None,
            speech_config: None,
            thinking_config: None,
            image_config: None,
            media_resolution: None,
        })
    }

    fn transform_response_format(format: OpenAiResponseFormat) -> (Option<String>, Option<Value>) {
        match format {
            OpenAiResponseFormat::Text => (None, None),
            OpenAiResponseFormat::JsonObject => (Some("application/json".to_string()), None),
            OpenAiResponseFormat::JsonSchema { json_schema } => (
                Some("application/json".to_string()),
                transform_response_json_schema(json_schema),
            ),
        }
    }

    fn transform_response_json_schema(schema: OpenAiResponseJsonSchema) -> Option<Value> {
        schema.schema
    }

    fn transform_stop(stop: OpenAiStop) -> Vec<String> {
        match stop {
            OpenAiStop::String(value) => vec![value],
            OpenAiStop::Array(values) => values,
        }
    }

    fn transform_modalities(modalities: &Vec<OpenAiChatCompletionModality>) -> Vec<ResponseModality> {
        modalities
            .iter()
            .map(|modality| match modality {
                OpenAiChatCompletionModality::Text => ResponseModality::Text,
                OpenAiChatCompletionModality::Audio => ResponseModality::Audio,
            })
            .collect()
    }

    fn text_part(text: String) -> Part {
        Part {
            text: Some(text),
            inline_data: None,
            file_data: None,
            function_call: None,
            function_response: None,
            executable_code: None,
            code_execution_result: None,
        }
    }

    fn parse_data_url(url: &str) -> Option<InlineData> {
        if !url.starts_with("data:") {
            return None;
        }

        let data = &url["data:".len()..];
        let mut segments = data.splitn(2, ',');
        let metadata = segments.next()?;
        let payload = segments.next()?;

        let mut metadata_iter = metadata.split(';');
        let mime_type = metadata_iter.next().filter(|value| !value.is_empty());
        let is_base64 = metadata_iter.any(|value| value == "base64");

        if !is_base64 {
            return None;
        }

        Some(InlineData {
            mime_type: mime_type.unwrap_or("application/octet-stream").to_string(),
            data: payload.to_string(),
        })
    }

    fn guess_mime_type(url: &str) -> Option<String> {
        let extension = url
            .split('?')
            .next()
            .and_then(|value| value.rsplit('.').next())
            .map(|value| value.to_ascii_lowercase())?;

        let mime_type = match extension.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "bmp" => "image/bmp",
            "svg" => "image/svg+xml",
            _ => return None,
        };

        Some(mime_type.to_string())
    }
}
// endregion: --- Transform methods

#[cfg(test)]
mod tests {
    use super::{from_openai_transform, Request};
    use crate::providers::openai::chat_completions::request::Request as OpenAiRequest;
    use crate::providers::Transformer;

    #[test]
    fn request_text_generation_example_roundtrip() {
        let json = r#"{
            "contents": [{
                "parts": [{"text": "Write a story about a magic backpack."}]
            }]
        }"#;

        let request: Request = serde_json::from_str(json).expect("parse request");
        assert_eq!(request.contents.len(), 1);
        assert_eq!(request.contents[0].parts.len(), 1);
        assert_eq!(
            request.contents[0].parts[0].text.as_deref(),
            Some("Write a story about a magic backpack.")
        );

        let value = serde_json::to_value(&request).expect("serialize request");
        assert_eq!(
            value["contents"][0]["parts"][0]["text"],
            "Write a story about a magic backpack."
        );
    }

    #[test]
    fn request_inline_data_example_roundtrip() {
        let json = r#"{
            "contents": [{
                "parts": [
                    {"text": "Tell me about this instrument."},
                    {"inline_data": {"mime_type": "image/jpeg", "data": "BASE64DATA"}}
                ]
            }]
        }"#;

        let request: Request = serde_json::from_str(json).expect("parse request");
        assert_eq!(request.contents.len(), 1);
        assert_eq!(request.contents[0].parts.len(), 2);
        assert_eq!(
            request.contents[0]
                .parts[1]
                .inline_data
                .as_ref()
                .map(|data| data.mime_type.as_str()),
            Some("image/jpeg")
        );

        let value = serde_json::to_value(&request).expect("serialize request");
        assert_eq!(
            value["contents"][0]["parts"][1]["inlineData"]["mimeType"],
            "image/jpeg"
        );
        assert_eq!(
            value["contents"][0]["parts"][1]["inlineData"]["data"],
            "BASE64DATA"
        );
    }

    #[test]
    fn request_safety_settings_example_roundtrip() {
        let json = r#"{
            "safetySettings": [
                {"category": "HARM_CATEGORY_HARASSMENT", "threshold": "BLOCK_ONLY_HIGH"},
                {"category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "BLOCK_MEDIUM_AND_ABOVE"}
            ],
            "contents": [{
                "parts": [{
                    "text": "I support Martians Soccer Club and I think Jupiterians Football Club sucks!"
                }]
            }]
        }"#;

        let request: Request = serde_json::from_str(json).expect("parse request");
        assert_eq!(request.safety_settings.as_ref().map(Vec::len), Some(2));
        assert_eq!(request.contents.len(), 1);

        let value = serde_json::to_value(&request).expect("serialize request");
        assert_eq!(
            value["safetySettings"][0]["category"],
            "HARM_CATEGORY_HARASSMENT"
        );
        assert_eq!(
            value["safetySettings"][0]["threshold"],
            "BLOCK_ONLY_HIGH"
        );
    }

    #[test]
    fn transform_openai_request_to_gemini() {
        let json = r#"{
            "model": "gpt-4o",
            "messages": [
                { "role": "system", "content": "You are a helper." },
                {
                    "role": "user",
                    "content": [
                        { "type": "text", "text": "What's the weather?" },
                        { "type": "image_url", "image_url": { "url": "data:image/png;base64,AAA=" } }
                    ]
                },
                {
                    "role": "assistant",
                    "content": "Calling tool.",
                    "tool_calls": [{
                        "type": "function",
                        "id": "call_1",
                        "function": { "name": "getWeather", "arguments": "{\"city\":\"Paris\"}" }
                    }]
                },
                { "role": "tool", "tool_call_id": "call_1", "content": "{\"temp\":21}" }
            ],
            "tools": [{
                "type": "function",
                "function": {
                    "name": "getWeather",
                    "description": "Returns weather",
                    "parameters": { "type": "object" }
                }
            }],
            "tool_choice": "auto",
            "n": 2,
            "max_tokens": 50,
            "response_format": { "type": "json_object" }
        }"#;

        let openai_request: OpenAiRequest = serde_json::from_str(json).expect("parse openai request");
        let transformed = openai_request.transform(from_openai_transform::Context { model: None });

        let request = transformed.result;
        let system_instruction = request.system_instruction.expect("system instruction");
        assert_eq!(system_instruction.parts[0].text.as_deref(), Some("You are a helper."));

        assert_eq!(request.contents.len(), 3);
        assert_eq!(request.contents[0].role.as_deref(), Some("user"));
        assert_eq!(request.contents[1].role.as_deref(), Some("model"));
        assert_eq!(request.contents[2].role.as_deref(), Some("user"));

        let user_parts = &request.contents[0].parts;
        assert_eq!(user_parts.len(), 2);
        assert_eq!(user_parts[1].inline_data.as_ref().map(|d| d.mime_type.as_str()), Some("image/png"));

        let tool_config = request.tool_config.expect("tool config");
        assert_eq!(
            tool_config
                .function_calling_config
                .as_ref()
                .and_then(|config| config.mode.as_deref()),
            Some("AUTO")
        );

        let tools = request.tools.expect("tools");
        let declarations = tools[0].function_declarations.as_ref().expect("function declarations");
        assert_eq!(declarations[0].name, "getWeather");

        let generation_config = request.generation_config.expect("generation config");
        assert_eq!(generation_config.candidate_count, Some(2));
        assert_eq!(generation_config.max_output_tokens, Some(50));
        assert_eq!(generation_config.response_mime_type.as_deref(), Some("application/json"));

        let tool_response = &request.contents[2].parts[0].function_response;
        assert_eq!(tool_response.as_ref().map(|resp| resp.name.as_str()), Some("call_1"));
    }
}
