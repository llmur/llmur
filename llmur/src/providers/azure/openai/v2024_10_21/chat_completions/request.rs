use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// region: --- Request structs
/// Azure OpenAI chat completion request payload.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Request {
    pub messages: Vec<Message>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Up to 4 sequences where the API will stop generating further tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_sources: Option<Vec<AzureChatExtensionConfiguration>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<FunctionDefinition>>,
}
// endregion: --- Request structs

// region: --- Message structs
/// Union of supported chat message roles for Azure chat completions.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum Message {
    #[serde(rename = "system", alias = "system")]
    SystemMessage {
        content: SystemMessageContent,
        name: Option<String>,
    },

    #[serde(rename = "user", alias = "user")]
    UserMessage {
        content: UserMessageContent,
        name: Option<String>,
    },

    #[serde(rename = "assistant", alias = "assistant")]
    AssistantMessage {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<AssistantMessageContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        refusal: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<AssistantToolCall>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        function_call: Option<AssistantFunctionCall>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
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

/// User message content as text or structured content parts.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserMessageContent {
    Text(String),
    Array(Vec<UserMessageContentPart>),
}

/// User message content part (text or image_url).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UserMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
    #[serde(rename = "image_url", alias = "image_url")]
    ImageUrl { image_url: ImageUrlContentPart },
}

/// Image URL content part object.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ImageUrlContentPart {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<ImageDetail>,
}

/// Detail level for image input processing.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    Auto,
    Low,
    High,
}

/// System message content as text or text content parts.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemMessageContent {
    Text(String),
    Array(Vec<SystemMessageContentPart>),
}

/// System message content part (text only).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SystemMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
}

/// Assistant message content as text or structured content parts.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AssistantMessageContent {
    Text(String),
    Array(Vec<AssistantMessageContentPart>),
}

/// Assistant message content part (text or refusal).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AssistantMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
    #[serde(rename = "refusal", alias = "refusal")]
    Refusal { text: String },
}

/// Tool message content as text or text content parts.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolMessageContent {
    Text(String),
    Array(Vec<ToolMessageContentPart>),
}

/// Tool message content part (text only).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
}

/// Tool call emitted by the model.
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
pub enum AssistantToolCallType {
    #[serde(rename = "function", alias = "function")]
    FunctionType,
}

/// Tool call function payload.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantToolCallFunction {
    pub name: String,
    pub arguments: String,
}

/// Deprecated function call payload in assistant messages.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantFunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Azure extension message context returned with assistant responses.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantMessageContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<AssistantContextCitation>>,
}

/// Citation entry in Azure extension message context.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantContextCitation {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filepath: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_id: Option<String>,
}
// endregion: --- Message structs

// region: --- Format structs
/// Response format configuration (text/json_object/json_schema).
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

/// JSON schema details for structured outputs.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseJsonSchema {
    pub name: String,
    pub description: Option<String>,
    pub schema: Option<serde_json::Value>,
    pub strict: Option<bool>
}
// endregion: --- Format structs

// region: --- Tool structs
/// Tool definition (Azure supports function tools).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Tool {
    #[serde(rename = "function", alias = "function")]
    Function { function: ToolFunction },
}

/// Function tool definition.
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

/// Tool choice configuration (mode or named tool).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    Mode(ToolChoiceMode),
    Function(ToolChoiceFunction),
}

/// Tool choice mode (none/auto/required).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceMode {
    None,
    Auto,
    Required,
}

/// Named tool selection.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolChoiceFunction {
    #[serde(rename = "function", alias = "function")]
    FunctionTool { function: ToolChoiceFunctionDetails },
}

/// Named function for tool choice.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceFunctionDetails {
    pub name: String,
}
// endregion: --- Tool structs

// region: --- Stop structs
/// Stop sequence configuration (string or array).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Stop {
    String(String),
    Array(Vec<String>),
}
// endregion: --- Stop structs

// region: --- Extension structs
/// Azure chat extension configuration (search/cosmos).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AzureChatExtensionConfiguration {
    #[serde(rename = "azure_cosmos_db", alias = "azure_cosmos_db")]
    AzureCosmosDB { parameters: serde_json::Value },

    #[serde(rename = "azure_search", alias = "azure_search")]
    AzureSearch { parameters: serde_json::Value },
}
// endregion: --- Extension structs

// region: --- Stream structs
/// Stream response options.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>
}
// endregion: --- Stream structs

// region: --- Function structs
/// Deprecated function_call request field (mode or named function).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionCall {
    Mode(FunctionCallMode),
    Function(FunctionCallOption),
}

/// Deprecated function_call mode.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FunctionCallMode {
    None,
    Auto,
}

/// Deprecated named function selection.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FunctionCallOption {
    pub name: String,
}

/// Deprecated function definition (use tools instead).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}
// endregion: --- Function structs

// region: --- Transform methods
pub mod from_openai_transform {
    use super::*;
    use crate::providers::openai::chat_completions::request::{
        Request as OpenAiRequest,
        Message as OpenAiMessage,
        UserMessageContent as OpenAiUserMessageContent,
        UserMessageContentPart as OpenAiUserMessageContentPart,
        ImageDetail as OpenAiImageDetail,
        SystemMessageContent as OpenAiSystemMessageContent,
        SystemMessageContentPart as OpenAiSystemMessageContentPart,
        DeveloperMessageContent as OpenAiDeveloperMessageContent,
        DeveloperMessageContentPart as OpenAiDeveloperMessageContentPart,
        AssistantMessageContent as OpenAiAssistantMessageContent,
        AssistantMessageContentPart as OpenAiAssistantMessageContentPart,
        ToolMessageContent as OpenAiToolMessageContent,
        ToolMessageContentPart as OpenAiToolMessageContentPart,
        AssistantToolCall as OpenAiAssistantToolCall,
        ResponseFormat as OpenAiResponseFormat,
        Tool as OpenAiTool,
        ToolChoice as OpenAiToolChoice,
        ToolChoiceMode as OpenAiToolChoiceMode,
        ToolChoiceFunction as OpenAiToolChoiceFunction,
        Stop as OpenAiStop,
        StreamOptions as OpenAiStreamOptions,
        FunctionCall as OpenAiFunctionCall,
        FunctionCallMode as OpenAiFunctionCallMode,
        FunctionCallOption as OpenAiFunctionCallOption,
        FunctionDefinition as OpenAiFunctionDefinition,
    };
    use crate::providers::{Transformation, TransformationContext, TransformationLoss, Transformer};
    
    #[derive(Debug)]
    pub struct Loss {
        pub model: String,
    }
    
    #[derive(Debug)]
    pub struct Context {
        pub data_sources: Option<Vec<AzureChatExtensionConfiguration>>,
        pub stream_include_usage: bool,
    }

    impl TransformationContext<OpenAiRequest, Request> for Context {}
    impl TransformationLoss<OpenAiRequest, Request> for Loss {}

    impl Transformer<Request, Context, Loss> for OpenAiRequest {
        fn transform(self, context: Context) -> Transformation<Request, Loss> {
            Transformation {
                result: Request {
                    messages: self.messages.into_iter().map(|msg| transform_message(msg)).collect(),
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
                    response_format: self.response_format.map(|rf| transform_response_format(rf)),
                    tools: self.tools.map(|values| {
                        values
                            .into_iter()
                            .filter_map(|tool| transform_tool(tool))
                            .collect()
                    }),
                    tool_choice: self.tool_choice.map(|tc| transform_tool_choice(tc)),
                    stop: self.stop.map(|s| transform_stop(s)),
                    data_sources: context.data_sources,
                    logprobs: self.logprobs,
                    top_logprobs: self.top_logprobs,
                    parallel_tool_calls: self.parallel_tool_calls,
                    stream_options: {
                        if self.stream.unwrap_or(false) && context.stream_include_usage {
                            let mut options = self.stream_options.unwrap_or(OpenAiStreamOptions {
                                include_usage: None,
                            });
                            options.include_usage = Some(true);
                            Some(transform_stream_options(options))
                        } else {
                            self.stream_options.map(|so| transform_stream_options(so))
                        }
                    },
                    function_call: self.function_call.map(transform_function_call),
                    functions: self.functions.map(|functions| {
                        functions
                            .into_iter()
                            .map(|definition| transform_function_definition(definition))
                            .collect()
                    }),
                },
                loss: Loss { model: self.model },
            }
        }
    }

    fn transform_message(msg: OpenAiMessage) -> Message {
        match msg {
            OpenAiMessage::SystemMessage { content, name } => Message::SystemMessage {
                content: transform_system_message_content(content),
                name,
            },
            OpenAiMessage::UserMessage { name, content } => Message::UserMessage {
                content: transform_user_message_content(content),
                name,
            },
            OpenAiMessage::AssistantMessage {
                content,
                name,
                refusal,
                tool_calls,
                function_call,
                ..
            } => Message::AssistantMessage {
                content: content.map(|c| transform_assistant_message_content(c)),
                refusal,
                tool_calls: tool_calls.map(|tc| {
                    tc.into_iter()
                        .filter_map(|t| transform_assistant_tool_call(t))
                        .collect()
                }),
                function_call: function_call.map(|call| AssistantFunctionCall {
                    name: call.name,
                    arguments: call.arguments,
                }),
                name,
            },
            OpenAiMessage::ToolMessage {
                content,
                tool_call_id,
            } => Message::ToolMessage {
                content: transform_tool_message_content(content),
                tool_call_id,
            },
            OpenAiMessage::FunctionMessage { content, name } => Message::FunctionMessage {
                content,
                name,
            },
            OpenAiMessage::DeveloperMessage { content, name } => {
                // Convert developer message into System Message
                Message::SystemMessage {
                    content: transform_developer_message_content(content),
                    name,
                }
            }
        }
    }

    fn transform_user_message_content(content: OpenAiUserMessageContent) -> UserMessageContent {
        match content {
            OpenAiUserMessageContent::Text(s) => UserMessageContent::Text(s),
            OpenAiUserMessageContent::Array(vec) => UserMessageContent::Array(
                vec.into_iter()
                    .filter_map(|part| transform_user_message_content_part(part))
                    .collect(),
            ),
        }
    }

    fn transform_user_message_content_part(part: OpenAiUserMessageContentPart) -> Option<UserMessageContentPart> {
        match part {
            OpenAiUserMessageContentPart::Text { text } => {
                Some(UserMessageContentPart::Text { text })
            }
            OpenAiUserMessageContentPart::ImageUrl { image_url } => {
                Some(UserMessageContentPart::ImageUrl {
                    image_url: ImageUrlContentPart {
                        url: image_url.url,
                        detail: image_url.detail.map(|detail| match detail {
                            OpenAiImageDetail::Auto => ImageDetail::Auto,
                            OpenAiImageDetail::Low => ImageDetail::Low,
                            OpenAiImageDetail::High => ImageDetail::High,
                        }),
                    },
                })
            }
            _ => None,
        }
    }

    fn transform_assistant_tool_call(tool_call: OpenAiAssistantToolCall) -> Option<AssistantToolCall> {
        match tool_call {
            OpenAiAssistantToolCall::Function { id, function } => {
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

    fn transform_tool_message_content(content: OpenAiToolMessageContent) -> ToolMessageContent {
        match content {
            OpenAiToolMessageContent::Text(s) => ToolMessageContent::Text(s),
            OpenAiToolMessageContent::Array(parts) => ToolMessageContent::Array(
                parts.into_iter().map(|part| transform_tool_message_content_part(part)).collect()
            ),
        }
    }

    fn transform_tool_message_content_part(part: OpenAiToolMessageContentPart) -> ToolMessageContentPart {
        match part {
            OpenAiToolMessageContentPart::Text { text } => ToolMessageContentPart::Text { text },
        }
    }

    fn transform_system_message_content(content: OpenAiSystemMessageContent) -> SystemMessageContent {
        match content {
            OpenAiSystemMessageContent::Text(s) => SystemMessageContent::Text(s),
            OpenAiSystemMessageContent::Array(vec) => SystemMessageContent::Array(
                vec.into_iter().map(|part| transform_system_message_content_part(part)).collect()
            ),
        }
    }

    fn transform_system_message_content_part(part: OpenAiSystemMessageContentPart) -> SystemMessageContentPart {
        match part {
            OpenAiSystemMessageContentPart::Text { text } => SystemMessageContentPart::Text { text },
        }
    }

    fn transform_developer_message_content(content: OpenAiDeveloperMessageContent) -> SystemMessageContent {
        match content {
            OpenAiDeveloperMessageContent::Text(s) => SystemMessageContent::Text(s),
            OpenAiDeveloperMessageContent::Array(vec) => SystemMessageContent::Array(
                vec.into_iter().map(|part| transform_developer_message_content_part(part)).collect()
            ),
        }
    }

    fn transform_developer_message_content_part(part: OpenAiDeveloperMessageContentPart) -> SystemMessageContentPart {
        match part {
            OpenAiDeveloperMessageContentPart::Text { text } => SystemMessageContentPart::Text { text },
        }
    }

    fn transform_assistant_message_content(content: OpenAiAssistantMessageContent) -> AssistantMessageContent {
        match content {
            OpenAiAssistantMessageContent::Text(s) => AssistantMessageContent::Text(s),
            OpenAiAssistantMessageContent::Array(vec) => AssistantMessageContent::Array(
                vec.into_iter().map(|part| transform_assistant_message_content_part(part)).collect()
            ),
        }
    }

    fn transform_assistant_message_content_part(part: OpenAiAssistantMessageContentPart) -> AssistantMessageContentPart {
        match part {
            OpenAiAssistantMessageContentPart::Text { text } => AssistantMessageContentPart::Text { text },
            OpenAiAssistantMessageContentPart::Refusal { text } => AssistantMessageContentPart::Refusal { text },
        }
    }

    fn transform_response_format(format: OpenAiResponseFormat) -> ResponseFormat {
        match format {
            OpenAiResponseFormat::Text => ResponseFormat::Text,
            OpenAiResponseFormat::JsonObject => ResponseFormat::JsonObject,
            OpenAiResponseFormat::JsonSchema { json_schema } => ResponseFormat::JsonSchema {
                json_schema: ResponseJsonSchema {
                    name: json_schema.name,
                    description: json_schema.description,
                    schema: json_schema.schema,
                    strict: json_schema.strict,
                },
            },
        }
    }

    fn transform_tool(tool: OpenAiTool) -> Option<Tool> {
        match tool {
            OpenAiTool::Function { function } => Some(Tool::Function {
                function: ToolFunction {
                    name: function.name,
                    description: function.description,
                    parameters: function.parameters,
                    strict: function.strict,
                },
            }),
        }
    }

    fn transform_tool_choice(tool_choice: OpenAiToolChoice) -> ToolChoice {
        match tool_choice {
            OpenAiToolChoice::Mode(mode) => ToolChoice::Mode(match mode {
                OpenAiToolChoiceMode::None => ToolChoiceMode::None,
                OpenAiToolChoiceMode::Auto => ToolChoiceMode::Auto,
                OpenAiToolChoiceMode::Required => ToolChoiceMode::Required,
            }),
            OpenAiToolChoice::Function(f) => ToolChoice::Function(transform_tool_choice_function(f)),
        }
    }

    fn transform_tool_choice_function(tool_choice_function: OpenAiToolChoiceFunction) -> ToolChoiceFunction {
        match tool_choice_function {
            OpenAiToolChoiceFunction::FunctionTool { function } => ToolChoiceFunction::FunctionTool {
                function: ToolChoiceFunctionDetails {
                    name: function.name,
                },
            },
        }
    }

    fn transform_stop(stop: OpenAiStop) -> Stop {
        match stop {
            OpenAiStop::String(s) => Stop::String(s),
            OpenAiStop::Array(v) => Stop::Array(v),
        }
    }

    fn transform_stream_options(stream_options: OpenAiStreamOptions) -> StreamOptions {
        StreamOptions {
            include_usage: stream_options.include_usage,
        }
    }

    fn transform_function_call(call: OpenAiFunctionCall) -> FunctionCall {
        match call {
            OpenAiFunctionCall::Mode(mode) => FunctionCall::Mode(match mode {
                OpenAiFunctionCallMode::None => FunctionCallMode::None,
                OpenAiFunctionCallMode::Auto => FunctionCallMode::Auto,
            }),
            OpenAiFunctionCall::Function(OpenAiFunctionCallOption { name }) => {
                FunctionCall::Function(FunctionCallOption { name })
            }
        }
    }

    fn transform_function_definition(definition: OpenAiFunctionDefinition) -> FunctionDefinition {
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
    use super::{
        FunctionCallMode, Message, Request, ToolChoice, ToolChoiceMode,
    };

    #[test]
    fn request_image_url_shape_and_tool_choice_mode() {
        let json = r#"{
            "messages": [{
                "role": "user",
                "content": [{
                    "type": "image_url",
                    "image_url": { "url": "https://example.com/image.png", "detail": "low" }
                }]
            }],
            "tool_choice": "required"
        }"#;

        let request: Request = serde_json::from_str(json).expect("parse request");
        match &request.messages[0] {
            Message::UserMessage { content, .. } => match content {
                super::UserMessageContent::Array(parts) => match &parts[0] {
                    super::UserMessageContentPart::ImageUrl { image_url } => {
                        assert_eq!(image_url.url, "https://example.com/image.png");
                    }
                    _ => panic!("expected image_url part"),
                },
                _ => panic!("expected array content"),
            },
            _ => panic!("expected user message"),
        }

        assert!(matches!(request.tool_choice, Some(ToolChoice::Mode(ToolChoiceMode::Required))));
    }

    #[test]
    fn request_function_call_and_functions_parse() {
        let json = r#"{
            "messages": [{
                "role": "assistant",
                "content": null,
                "function_call": { "name": "do_thing", "arguments": "{}" }
            }],
            "function_call": "auto",
            "functions": [{
                "name": "do_thing",
                "description": "Do the thing.",
                "parameters": { "type": "object", "properties": {} }
            }]
        }"#;

        let request: Request = serde_json::from_str(json).expect("parse request");
        assert!(matches!(
            request.function_call,
            Some(super::FunctionCall::Mode(FunctionCallMode::Auto))
        ));
        assert_eq!(request.functions.as_ref().map(Vec::len), Some(1));
        match &request.messages[0] {
            Message::AssistantMessage { function_call, .. } => {
                assert_eq!(
                    function_call.as_ref().map(|call| call.name.as_str()),
                    Some("do_thing")
                );
            }
            _ => panic!("expected assistant message"),
        }
    }
}
