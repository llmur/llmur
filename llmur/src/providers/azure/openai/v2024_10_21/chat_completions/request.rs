use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// region: --- Request structs
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
}
// endregion: --- Request structs

// region: --- Message structs
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
        name: Option<String>,
    },

    #[serde(rename = "tool", alias = "tool")]
    ToolMessage {
        content: ToolMessageContent,
        tool_call_id: String,
    },

    #[serde(rename = "function", alias = "function")]
    FunctionMessage { content: String, name: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserMessageContent {
    Text(String),
    Array(Vec<UserMessageContentPart>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UserMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
    #[serde(rename = "image_url", alias = "image_url")]
    ImageUrl { image_url: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemMessageContent {
    Text(String),
    Array(Vec<SystemMessageContentPart>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SystemMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
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
#[serde(untagged)]
pub enum ToolMessageContent {
    Text(String),
    Array(Vec<ToolMessageContentPart>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolMessageContentPart {
    #[serde(rename = "text", alias = "text")]
    Text { text: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AssistantToolCall {
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssistantMessageContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<AssistantContextCitation>>,
}

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

// region: --- Extension structs
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
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>
}
// endregion: --- Stream structs

// region: --- Transform methods
pub mod from_openai_transform {
    use super::*;
    use crate::providers::openai::chat_completions::request::{
        Request as OpenAiRequest,
        Message as OpenAiMessage,
        UserMessageContent as OpenAiUserMessageContent,
        UserMessageContentPart as OpenAiUserMessageContentPart,
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
        ResponseFormat as OpenAiResponseFormat,
        ResponseJsonSchema as OpenAiResponseJsonSchema,
        Tool as OpenAiTool,
        ToolFunction as OpenAiToolFunction,
        ToolChoice as OpenAiToolChoice,
        ToolChoiceFunction as OpenAiToolChoiceFunction,
        ToolChoiceFunctionDetails as OpenAiToolChoiceFunctionDetails,
        Stop as OpenAiStop,
        StreamOptions as OpenAiStreamOptions,
    };
    use crate::providers::{Transformation, TransformationContext, TransformationLoss, Transformer};
    
    #[derive(Debug)]
    pub struct Loss {
        pub model: String,
    }
    
    #[derive(Debug)]
    pub struct Context {
        pub data_sources: Option<Vec<AzureChatExtensionConfiguration>>,
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
                    stream_options: self.stream_options.map(|so| transform_stream_options(so)),
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
                ..
            } => Message::AssistantMessage {
                content: content.map(|c| transform_assistant_message_content(c)),
                refusal,
                tool_calls: tool_calls.map(|tc| {
                    tc.into_iter()
                        .filter_map(|t| transform_assistant_tool_call(t))
                        .collect()
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
                    image_url: image_url.url,
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
            OpenAiAssistantToolCall::Custom { .. } => None,
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
            OpenAiTool::Custom { .. } => None,
        }
    }

    fn transform_tool_choice(tool_choice: OpenAiToolChoice) -> ToolChoice {
        match tool_choice {
            OpenAiToolChoice::String(s) => ToolChoice::String(s),
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
}
// endregion: --- Transform methods
