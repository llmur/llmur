use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChatCompletionRequest {
    /// ID of the model to use. See the [model endpoint compatibility](https://platform.openai.com/docs/models/model-endpoint-compatibility) table for details on which models work with the Chat API.
    pub model: String,

    /// A list of messages comprising the conversation so far. [Example Python code](https://cookbook.openai.com/examples/how_to_format_inputs_to_chatgpt_models).
    pub messages: Vec<ChatCompletionMessage>,

    /// How many chat completion choices to generate for each input message. Note that you will be charged based on the number of generated tokens across all of the choices. Keep n as 1 to minimize costs.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub n: Option<u64>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim.
    /// [See more information about frequency and presence penalties](https://platform.openai.com/docs/guides/text-generation/frequency-and-presence-penalties).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub frequency_penalty: Option<f64>,
    
    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic.
    /// We generally recommend altering this or top_p but not both.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub temperature: Option<f64>,
    
    /// Whether to return log probabilities of the output tokens or not. If true, returns the log probabilities of each output token returned in the content of message
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub logprobs: Option<bool>,
    
    /// An integer between 0 and 20 specifying the number of most likely tokens to return at each token position, each with an associated log probability. logprobs must be set to true if this parameter is used.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub top_logprobs: Option<i64>,
    
    /// The maximum number of [tokens](https://platform.openai.com/tokenizer) that can be generated in the chat completion.
    /// The total length of input tokens and generated tokens is limited by the model's context length. [Example Python code for counting tokens](https://cookbook.openai.com/examples/how_to_count_tokens_with_tiktoken).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub max_tokens: Option<u64>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.
    /// [See more information about frequency and presence penalties](https://platform.openai.com/docs/guides/text-generation/frequency-and-presence-penalties).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub presence_penalty: Option<f64>,
    
    /// An alternative to sampling with temperature, called nucleus sampling, where the model considers the results of the tokens with top_p probability mass. So 0.1 means only the tokens comprising the top 10% probability mass are considered.
    /// We generally recommend altering this or temperature but not both.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub top_p: Option<f64>,
    
    /// If set, partial message deltas will be sent, like in ChatGPT. Tokens will be sent as data-only [server-sent events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#event_stream_format) as they become available, with the stream terminated by a data: [DONE] message. [Example Python code](https://cookbook.openai.com/examples/how_to_stream_completions).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub stream: Option<bool>,
    
    /// Up to 4 sequences where the API will stop generating further tokens. The returned text will not contain the stop sequence.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub stop: Option<ChatCompletionStop>,

    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse. [Learn more](https://platform.openai.com/docs/guides/safety-best-practices).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub user: Option<String>,
    
    /// If specified, our system will make a best effort to sample deterministically, such that repeated requests with the same seed and parameters should return the same result.
    /// Determinism is not guaranteed, and you should refer to the system_fingerprint response parameter to monitor changes in the backend.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub seed: Option<i64>,

    /// Specifies the format that the model must output. Compatible with [GPT-4o](https://platform.openai.com/docs/models/gpt-4o), [GPT-4 Turbo](https://platform.openai.com/docs/models/gpt-4-turbo-and-gpt-4), and all GPT-3.5 Turbo models since `gpt-3.5-turbo-1106`.
    /// Setting to `{ "type": "json_object" }` enables JSON mode, which guarantees the message the model generates is valid JSON.
    /// **Important**: when using JSON mode, you **must** also instruct the model to produce JSON yourself via a system or user message. Without this, the model may generate an unending stream of whitespace until the generation reaches the token limit, resulting in a long-running and seemingly "stuck" request. Also note that the message content may be partially cut off if `finish_reason="length"`, which indicates the generation exceeded `max_tokens` or the conversation exceeded the max context length.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub response_format: Option<serde_json::Value>,
    
    /// Modify the likelihood of specified tokens appearing in the completion.
    /// Accepts a JSON object that maps tokens (specified by their token ID in the GPT tokenizer) to an associated bias value from -100 to 100. You can use this [tokenizer tool](https://platform.openai.com/tokenizer?view=bpe) to convert text to token IDs. Mathematically, the bias is added to the logits generated by the model prior to sampling. The exact effect will vary per model, but values between -1 and 1 should decrease or increase likelihood of selection; values like -100 or 100 should result in a ban or exclusive selection of the relevant token.
    /// As an example, you can pass `{"50256": -100}` to prevent the <|endoftext|> token from being generated.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub logit_bias: Option<HashMap<String, i32>>,
    
    
    /// A list of tools the model may call. Currently, only functions are supported as a tool. Use this to provide a list of functions the model may generate JSON inputs for. A max of 128 functions are supported.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub tools: Option<Vec<ChatCompletionTool>>,
    
    /// Controls which (if any) tool is called by the model. `none` means the model will not call any tool and instead generates a message. `auto` means the model can pick between generating a message or calling one or more tools. `required` means the model must call one or more tools. Specifying a particular tool via `{"type": "function", "function": {"name": "my_function"}}` forces the model to call that tool.
    /// `none` is the default when no tools are present. `auto` is the default if tools are present.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub tool_choice: Option<ChatCompletionToolChoice>,
}

// region:    --- ChatCompletionStop

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(untagged))]
pub enum ChatCompletionStop {
    StringStop(String),
    ArrayStop(Vec<String>),
}

// endregion: --- ChatCompletionStop
// region:    --- ChatCompletionMessage
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(tag = "role"))]
pub enum ChatCompletionMessage {
    #[cfg_attr(feature = "serde", serde(alias = "system"))]
    SystemMessage {
        content: String,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        name: Option<String>,
    },
    #[cfg_attr(feature = "serde", serde(alias = "user"))]
    UserMessage {
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        name: Option<String>,
        content: UserMessageContent,
    },
    #[cfg_attr(feature = "serde", serde(alias = "assistant"))]
    AssistantMessage {
        //TODO Confirm if content is optional
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        content: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        name: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        tool_calls: Option<Vec<AssistantToolCall>>,
        // TODO: Decide if should implement deprecated parameters (function_call)
    },
    #[cfg_attr(feature = "serde", serde(alias = "tool"))]
    ToolMessage {
        content: String,
        tool_call_id: String,
    },
    // TODO: Decide if should implement deprecated parameters (FunctionMessage)
}

// region:    --- Chat Completion Message Content
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(untagged))]
pub enum UserMessageContent {
    TextContent(String),
    ArrayContentParts(Vec<UserMessageContentPart>),
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(tag="type"))]
pub enum UserMessageContentPart {
    #[cfg_attr(feature = "serde", serde(alias = "text"))]
    TextContentPart {
        text: String,
    },
    #[cfg_attr(feature = "serde", serde(alias = "image_url"))]
    ImageContentPart {
        image_url: ImageUrlContentPart,
    },
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImageUrlContentPart {
    pub url: String,
    #[cfg_attr(feature = "serde", serde(alias = "tool"))]
    pub detail: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssistantToolCall {
    pub id: String,
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub r#type: AssistantToolCallType,
    pub function: AssistantToolCallFunction,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AssistantToolCallType {
    #[cfg_attr(feature = "serde", serde(rename = "function"))]
    FunctionType
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssistantToolCallFunction {
    pub name: String,
    pub arguments: String,
}

// endregion: --- Chat Completion Message Content
// endregion: --- ChatCompletionMessage

// region:    --- Tools
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(tag="type"))]
pub enum ChatCompletionTool {
    #[cfg_attr(feature = "serde", serde(alias = "function"))]
    FunctionTool {
        function: ChatCompletionToolFunction
    }
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChatCompletionToolFunction {
    pub name: String,
    #[cfg_attr(feature = "serde", serde(alias = "tool"))]
    pub description: Option<String>,
    #[cfg_attr(feature = "serde", serde(alias = "tool"))]
    pub parameters: Option<serde_json::Value>
}


#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(untagged))]
pub enum ChatCompletionToolChoice {
    StringChoice(String),
    FunctionChoice(ChatCompletionToolChoiceObject)
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(tag="type"))]
pub enum ChatCompletionToolChoiceObject {
    #[cfg_attr(feature = "serde", serde(alias = "function"))]
    FunctionTool {
        function: ChatCompletionToolChoiceFunction
    }
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChatCompletionToolChoiceFunction {
    pub name: String,
}

// endregion: --- Tools
// region:    --- Tests


#[cfg(test)]
mod tests {
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>; // For early tests.

    use serde_json::json;
    use super::*;

    #[test]
    fn test_chat_completions_01_decode_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_user_json = json!({
            "role": "user",
            "content": "hello"
        }).to_string();
        let fx_system_json = json!({
            "role": "system",
            "content": "world"
        }).to_string();
        {
            let data: ChatCompletionMessage = serde_json::from_str(&fx_user_json).unwrap();
            assert_eq!(data, ChatCompletionMessage::UserMessage { content: UserMessageContent::TextContent("hello".to_string()), name: None })
        }
        {
            let data: ChatCompletionMessage = serde_json::from_str(&fx_system_json).unwrap();
            assert_eq!(data, ChatCompletionMessage::SystemMessage { content: "world".to_string(), name: None })
        }

        Ok(())
    }

    #[test]
    fn test_assistants_message_tool_call_01_decode_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_tool_call = json!({
            "id": "123",
            "type": "function",
            "function": {
                "name": "my_name",
                "arguments": "args"
            }
        }).to_string();
        let data: AssistantToolCall = serde_json::from_str(&fx_tool_call).unwrap();

        assert_eq!(data, AssistantToolCall {
            id: "123".to_string(),
            r#type: AssistantToolCallType::FunctionType,
            function: AssistantToolCallFunction {
                name: "my_name".to_string(),
                arguments: "args".to_string(),
            },
        });

        Ok(())
    }

    #[test]
    fn test_assistants_message_tool_call_01_decode_fail() -> Result<()> {
        // -- Setup & Fixtures
        let fx_tool_call = json!({
            "id": "123",
            "type": "invalid",
            "function": {
                "name": "my_name",
                "arguments": "args"
            }
        }).to_string();
        let data: serde_json::error::Result<AssistantToolCall> = serde_json::from_str(&fx_tool_call);

        assert!(data.is_err());

        Ok(())
    }

    #[test]
    fn test_request_response_format_01_decode_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_request = json!({
            "model": "gpt-4o",
            "messages": [],
            "response_format": {
                "hello": "world"
            }
          }).to_string();
    
        let data: ChatCompletionRequest = serde_json::from_str(&fx_request).unwrap();
        
        assert_eq!(data.model, "gpt-4o".to_string());
        assert_eq!(data.response_format, Some(json!({"hello": "world"})));
        
        Ok(())
    }

    #[test]
    fn test_request_stop_01_decode_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_request = json!({
            "model": "gpt-4o",
            "messages": [],
            "stop": ["hello", "world"]
          }).to_string();
    
        let _: ChatCompletionRequest = serde_json::from_str(&fx_request).unwrap();
        
        Ok(())
    }

    #[test]
    fn test_request_stop_02_decode_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_request = json!({
            "model": "gpt-4o",
            "messages": [],
            "stop": "hello"
          }).to_string();
    
        let _: ChatCompletionRequest = serde_json::from_str(&fx_request).unwrap();
        
        Ok(())
    }

    #[test]
    fn test_openai_example_01_decode_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_request = json!({
            "model": "gpt-4o",
            "messages": [
              {
                "role": "system",
                "content": "You are a helpful assistant."
              },
              {
                "role": "user",
                "content": "Hello!"
              }
            ]
          }).to_string();
    
        let _: ChatCompletionRequest = serde_json::from_str(&fx_request).unwrap();
        
        Ok(())
    }

    #[test]
    fn test_openai_example_02_decode_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_request = json!({
            "model": "gpt-4-turbo",
            "messages": [
            {
                "role": "user",
                "content": [
                {
                    "type": "text",
                    "text": "What's in this image?"
                },
                {
                    "type": "image_url",
                    "image_url": {
                        "url": "https://upload.wikimedia.org/wikipedia/commons/thumb/d/dd/Gfp-wisconsin-madison-the-nature-boardwalk.jpg/2560px-Gfp-wisconsin-madison-the-nature-boardwalk.jpg"
                    }
                }
                ]
            }
            ],
            "max_tokens": 300
        }).to_string();
    
        let _: ChatCompletionRequest = serde_json::from_str(&fx_request).unwrap();
        
        Ok(())
    }

    #[test]
    fn test_openai_example_03_decode_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_request = json!({
            "model": "gpt-4o",
            "messages": [
              {
                "role": "system",
                "content": "You are a helpful assistant."
              },
              {
                "role": "user",
                "content": "Hello!"
              }
            ],
            "stream": true
          }).to_string();
    
        let _data: ChatCompletionRequest = serde_json::from_str(&fx_request).unwrap();
        
        Ok(())
    }

    #[test]
    fn test_openai_example_04_decode_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_request = json!({
            "model": "gpt-4-turbo",
            "messages": [
              {
                "role": "user",
                "content": "What'\''s the weather like in Boston today?"
              }
            ],
            "tools": [
              {
                "type": "function",
                "function": {
                  "name": "get_current_weather",
                  "description": "Get the current weather in a given location",
                  "parameters": {
                    "type": "object",
                    "properties": {
                      "location": {
                        "type": "string",
                        "description": "The city and state, e.g. San Francisco, CA"
                      },
                      "unit": {
                        "type": "string",
                        "enum": ["celsius", "fahrenheit"]
                      }
                    },
                    "required": ["location"]
                  }
                }
              }
            ],
            "tool_choice": "auto"
          }).to_string();
    
        let _: ChatCompletionRequest = serde_json::from_str(&fx_request).unwrap();
        
        Ok(())
    }

    #[test]
    fn test_openai_example_05_decode_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_request = json!({
            "model": "gpt-4o",
            "messages": [
              {
                "role": "user",
                "content": "Hello!"
              }
            ],
            "logprobs": true,
            "top_logprobs": 2
          }).to_string();
    
        let _: ChatCompletionRequest = serde_json::from_str(&fx_request).unwrap();
        
        Ok(())
    }

    #[test]
    fn test_tool_schema_01_decode_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_request = json!({
            "model": "gpt-4-turbo",
            "messages": [
              {
                "role": "user",
                "content": "What'\''s the weather like in Boston today?"
              }
            ],
            "tools": [
              {
                "type": "function",
                "function": {
                  "name": "get_current_weather",
                  "description": "Get the current weather in a given location",
                  "parameters": {
                    "type": "object",
                    "properties": {
                      "location": {
                        "type": "string",
                        "description": "The city and state, e.g. San Francisco, CA"
                      },
                      "unit": {
                        "type": "string",
                        "enum": ["celsius", "fahrenheit"]
                      }
                    },
                    "required": ["location"]
                  }
                }
              }
            ],
            "tool_choice": {
                "type": "function",
                "function": {
                    "name": "get_current_weather"
                }
            }
          }).to_string();
    
        let _: ChatCompletionRequest = serde_json::from_str(&fx_request).unwrap();
        
        Ok(())
    }
}

// endregion:    --- Tests