use crate::openai::v1::chat_completion::request::ChatCompletionRequest as OpenAIChatCompletionRequest;
use crate::openai::v1::chat_completion::request::ChatCompletionMessage as OpenAIChatCompletionMessage;
use crate::openai::v1::chat_completion::request::UserMessageContent as OpenAIUserMessageContent;
use crate::openai::v1::chat_completion::request::UserMessageContentPart as OpenAIUserMessageContentPart;
use crate::openai::v1::chat_completion::request::AssistantToolCallType as OpenAIAssistantToolCallType;
use crate::openai::v1::chat_completion::request::ChatCompletionTool as OpenAIChatCompletionTool;
use crate::openai::v1::chat_completion::request::ChatCompletionToolChoice as OpenAIChatCompletionToolChoice;
use crate::openai::v1::chat_completion::request::ChatCompletionToolChoiceObject as OpenAIChatCompletionToolChoiceObject;
use crate::openai::v1::chat_completion::request::ChatCompletionStop as OpenAIChatCompletionStop;


use crate::azure::v2024_02_01::chat_completion::request::ChatCompletionRequest as AzureChatCompletionRequest;
use crate::azure::v2024_02_01::chat_completion::request::ChatCompletionMessage as AzureChatCompletionMessage;
use crate::azure::v2024_02_01::chat_completion::request::UserMessageContent as AzureUserMessageContent;
use crate::azure::v2024_02_01::chat_completion::request::UserMessageContentPart as AzureUserMessageContentPart;
use crate::azure::v2024_02_01::chat_completion::request::AssistantToolCall as AzureAssistantToolCall;
use crate::azure::v2024_02_01::chat_completion::request::AssistantToolCallType as AzureAssistantToolCallType;
use crate::azure::v2024_02_01::chat_completion::request::AssistantToolCallFunction as AzureAssistantToolCallFunction;
use crate::azure::v2024_02_01::chat_completion::request::ChatCompletionTool as AzureChatCompletionTool;
use crate::azure::v2024_02_01::chat_completion::request::ChatCompletionToolFunction as AzureChatCompletionToolFunction;
use crate::azure::v2024_02_01::chat_completion::request::ChatCompletionToolChoice as AzureChatCompletionToolChoice;
use crate::azure::v2024_02_01::chat_completion::request::ChatCompletionToolChoiceFunction as AzureChatCompletionToolChoiceFunction;
use crate::azure::v2024_02_01::chat_completion::request::ChatCompletionToolChoiceObject as AzureChatCompletionToolChoiceObject;
use crate::azure::v2024_02_01::chat_completion::request::ChatCompletionStop as AzureChatCompletionStop;
use crate::azure::v2024_02_01::chat_completion::request::AzureChatExtensionConfiguration as AzureChatExtensionConfiguration;




impl OpenAIChatCompletionRequest {
    pub fn to_azure_v2024_02_01(&self, context: TransformationContext) -> Transformation {
        let _ = context;
        Transformation {
            request: AzureChatCompletionRequest {
                messages: self.messages.clone()
                    .into_iter()
                    .map(|message| 
                        match message {
                            OpenAIChatCompletionMessage::SystemMessage { content, ..  } => 
                                AzureChatCompletionMessage::SystemMessage { 
                                    content 
                                },
                            OpenAIChatCompletionMessage::UserMessage { content, .. } => 
                                AzureChatCompletionMessage::UserMessage { 
                                    content: match content {
                                        OpenAIUserMessageContent::TextContent(value) => 
                                            AzureUserMessageContent::TextContent(value),
                                        OpenAIUserMessageContent::ArrayContentParts(parts) => 
                                            AzureUserMessageContent::ArrayContentParts(
                                                parts.into_iter()
                                                    .map(|part| match part {
                                                        OpenAIUserMessageContentPart::TextContentPart { text } => AzureUserMessageContentPart::TextContentPart { text },
                                                        OpenAIUserMessageContentPart::ImageContentPart { image_url } => AzureUserMessageContentPart::ImageContentPart { image_url: image_url.url },
                                                    })
                                                    .collect()
                                            ),
                                    }
                                },
                            OpenAIChatCompletionMessage::AssistantMessage { content, tool_calls, .. } => 
                                AzureChatCompletionMessage::AssistantMessage { 
                                    content, 
                                    tool_calls: tool_calls.map(|calls|
                                        calls.into_iter()
                                            .map(|call|
                                                AzureAssistantToolCall {
                                                    id: call.id,
                                                    r#type: match call.r#type {
                                                        OpenAIAssistantToolCallType::FunctionType => AzureAssistantToolCallType::FunctionType,
                                                    },
                                                    function: AzureAssistantToolCallFunction {
                                                        name: call.function.name,
                                                        arguments: call.function.arguments,
                                                    },
                                                }
                                            ).collect()
                                    ), 
                                    context: None
                                },
                            OpenAIChatCompletionMessage::ToolMessage { content, tool_call_id } => 
                                AzureChatCompletionMessage::ToolMessage { 
                                    content, 
                                    tool_call_id
                                },
                        }
                    )
                    .collect(),
                temperature: self.temperature,
                top_p: self.top_p,
                stream: self.stream,
                max_tokens: self.max_tokens,
                presence_penalty: self.presence_penalty,
                frequency_penalty: self.frequency_penalty,
                logit_bias: self.logit_bias.clone(),
                n: self.n,
                seed: self.seed,
                user: self.user.clone(),
                response_format: self.response_format.clone(),
                tools: self.tools.clone().map(|tls| tls
                    .into_iter()
                    .map(|tool| match tool {
                        OpenAIChatCompletionTool::FunctionTool { function } => AzureChatCompletionTool::FunctionTool{
                            function: AzureChatCompletionToolFunction {
                                name: function.name,
                                description: function.description,
                                parameters: function.parameters,
                            },
                        },
                    })
                    .collect()
                ),
                tool_choice: self.tool_choice.clone().map(|choice| match choice {
                    OpenAIChatCompletionToolChoice::StringChoice(v) => AzureChatCompletionToolChoice::StringChoice(v),
                    OpenAIChatCompletionToolChoice::FunctionChoice(v) => AzureChatCompletionToolChoice::FunctionChoice(
                        match v {
                            OpenAIChatCompletionToolChoiceObject::FunctionTool { function } => AzureChatCompletionToolChoiceObject::FunctionTool {
                                function: AzureChatCompletionToolChoiceFunction {
                                    name: function.name,
                                },
                            }
                        }
                    ),
                } ),
                stop: self.stop.clone().map(|stop| match stop {
                    OpenAIChatCompletionStop::StringStop(v) => AzureChatCompletionStop::StringStop(v),
                    OpenAIChatCompletionStop::ArrayStop(v) => AzureChatCompletionStop::ArrayStop(v),
                }),
                data_sources: context.data_sources,
            },
            loss: TransformationLoss {
                model: self.model.clone()
            },
        }
    }
}

pub struct TransformationLoss {
    pub model: String
}

pub struct TransformationContext {
    pub data_sources: Option<Vec<AzureChatExtensionConfiguration>>
}

pub struct Transformation {
    pub request: AzureChatCompletionRequest,
    pub loss: TransformationLoss
}

// region:    --- Tests
#[cfg(test)]
mod tests {
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>; // For early tests.

    use crate::openai::v1::chat_completion::request::ImageUrlContentPart;

    use super::*;

    #[test]
    fn test_basic_request_transform_ok() -> Result<()> {
        let fx_messages = Vec::<OpenAIChatCompletionMessage>::new();
        let fx_request = OpenAIChatCompletionRequest {
            model: "my-model".to_string(),
            messages: fx_messages,
            n: Some(5),
            frequency_penalty: Some(0.5),
            temperature: None,
            logprobs: None,
            top_logprobs: None,
            max_tokens: None,
            presence_penalty: None,
            top_p: None,
            stream: None,
            stop: None,
            user: Some("user-1234".to_string()),
            seed: None,
            response_format: None,
            logit_bias: None,
            tools: None,
            tool_choice: None,
        };

        let data = fx_request.to_azure_v2024_02_01(TransformationContext {data_sources: None});

        // Check if the model was passed to the loss object.
        assert_eq!(data.loss.model, fx_request.model);

        // Check if all the other fields were copied correctly. Excludes messages, as it's a complex type and will be tested separately.
        assert_eq!(data.request.n, Some(5));
        assert_eq!(data.request.frequency_penalty, Some(0.5));
        assert_eq!(data.request.temperature, None);
        assert_eq!(data.request.user, Some("user-1234".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_system_message_request_transform_ok() -> Result<()> {
        let mut fx_messages = Vec::<OpenAIChatCompletionMessage>::new();
        fx_messages.push(OpenAIChatCompletionMessage::SystemMessage { content: "hello".to_string(), name: Some("my-name".to_string()) });

        let fx_request = OpenAIChatCompletionRequest {
            model: "my-model".to_string(),
            messages: fx_messages,
            n: None,
            frequency_penalty: None,
            temperature: None,
            logprobs: None,
            top_logprobs: None,
            max_tokens: None,
            presence_penalty: None,
            top_p: None,
            stream: None,
            stop: None,
            user: None,
            seed: None,
            response_format: None,
            logit_bias: None,
            tools: None,
            tool_choice: None,
        };

        let data = fx_request.to_azure_v2024_02_01(TransformationContext {data_sources: None});

        // Check if the model was passed to the loss object.
        assert_eq!(data.loss.model, fx_request.model);

        // Check if system message was transformed correctly.
        assert_eq!(data.request.messages.len(), 1);
        let el: &AzureChatCompletionMessage = data.request.messages.first().unwrap();

        match el {
            AzureChatCompletionMessage::SystemMessage { content } => {
                assert_eq!(content, "hello");
            },
            _ => panic!("Expected a SystemMessage"),
        }
        
        Ok(())
    }

    #[test]
    fn test_user_message_request_transform_ok() -> Result<()> {
    
        let mut fx_messages = Vec::<OpenAIChatCompletionMessage>::new();
        fx_messages.push(OpenAIChatCompletionMessage::UserMessage {name: None, content: OpenAIUserMessageContent::TextContent("hello".to_string()) });
        fx_messages.push(OpenAIChatCompletionMessage::UserMessage {name: None, content: OpenAIUserMessageContent::ArrayContentParts(vec![
            OpenAIUserMessageContentPart::TextContentPart { text: "part".to_string() },
            OpenAIUserMessageContentPart::ImageContentPart { image_url: ImageUrlContentPart { url: "http://example.com".to_string(), detail: Some("detail".to_string()) } },
            ]) 
        });

        let fx_request = OpenAIChatCompletionRequest {
            model: "my-model".to_string(),
            messages: fx_messages,
            n: None,
            frequency_penalty: None,
            temperature: None,
            logprobs: None,
            top_logprobs: None,
            max_tokens: None,
            presence_penalty: None,
            top_p: None,
            stream: None,
            stop: None,
            user: None,
            seed: None,
            response_format: None,
            logit_bias: None,
            tools: None,
            tool_choice: None,
        };

        let data = fx_request.to_azure_v2024_02_01(TransformationContext {data_sources: None});

        // Check if the model was passed to the loss object.
        assert_eq!(data.loss.model, fx_request.model);

        // Check if system message was transformed correctly.
        assert_eq!(data.request.messages.len(), 2);

        let el0: &AzureChatCompletionMessage = &data.request.messages[0];
        let el1: &AzureChatCompletionMessage = &data.request.messages[1];

        // Check first message - Text Content
        match el0 {
            AzureChatCompletionMessage::UserMessage { content } => {
                match content {
                    AzureUserMessageContent::TextContent(v) => {
                        assert_eq!(v, "hello");
                    },
                    _ => panic!("Expected a TextContent"),
                }
            },
            _ => panic!("Expected a UserMessage"),
        }

        // Check second message - Array Content
        match el1 {
            AzureChatCompletionMessage::UserMessage { content } => {
                match content {
                    AzureUserMessageContent::ArrayContentParts(parts) => {
                        assert_eq!(parts.len(), 2);

                        match &parts[0] {
                            AzureUserMessageContentPart::TextContentPart { text } => {
                                assert_eq!(text, "part");
                            },
                            _ => panic!("Expected a TextContentPart"),
                        }

                        match &parts[1] {
                            AzureUserMessageContentPart::ImageContentPart { image_url } => {
                                assert_eq!(image_url, "http://example.com");
                            },
                            _ => panic!("Expected a ImageContentPart"),
                        }
                    },
                    _ => panic!("Expected a ArrayContentParts"),
                }
            },
            _ => panic!("Expected a UserMessage"),
        }
        
        Ok(())
    }

}

// endregion:    --- Tests