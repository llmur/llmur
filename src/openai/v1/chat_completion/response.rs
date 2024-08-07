// region:    --- Object Response
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChatCompletionObjectResponse {
	/// A unique identifier for the chat completion.
	pub id: String,
	/// A list of chat completion choices. Can be more than one if n is greater than 1.
	pub choices: Vec<ChatCompletionObjectResponseChoice>,
	/// The Unix timestamp (in seconds) of when the chat completion
	pub created: u64,
	/// The model used for the chat completion.
	pub model: String,
	/// This fingerprint represents the backend configuration that the model runs with.
	/// Can be used in conjunction with the seed request parameter to understand when backend
	/// changes have been made that might impact determinism.
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub system_fingerprint: Option<String>,
	/// The object type, which is always chat.completion.
	pub object: String,
	/// Usage statistics for the completion request.
	pub usage: ChatCompletionResponseUsage,
	/// The service tier used for processing the request. This field is only included if the
	/// service_tier parameter is specified in the request.
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub service_tier: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChatCompletionObjectResponseChoice {
	/// The reason the model stopped generating tokens. This will be stop if the model hit a
	/// natural stop point or a provided stop sequence, length if the maximum number of tokens
	/// specified in the request was reached, content_filter if content was omitted due to a flag
	/// from our content filters, tool_calls if the model called a tool, or function_call
	/// (deprecated) if the model called a function.
	pub finish_reason: String,

	/// The index of the choice in the list of choices.
	pub index: u64,

	// A chat completion message generated by the model.
	pub message: ChatCompletionObjectResponseChoiceMessage,

	/// Log probability information for the choice.
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub logprobs: Option<ChatCompletionResponseChoiceLogprob>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChatCompletionObjectResponseChoiceMessage {
	/// The contents of the message
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub content: Option<String>,
	/// The role of the author of the message
	pub role: String,
	/// The tool calls generated by the model, such as function calls.
	pub tool_calls: Option<Vec<ChatCompletionObjectResponseChoiceToolCall>>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(tag = "type"))]
pub enum ChatCompletionObjectResponseChoiceToolCall {
	#[cfg_attr(feature = "serde", serde(rename = "function", alias = "function"))]
	FunctionTool { id: String, function: ChatCompletionResponseChoiceFunctionToolCall },
}

// endregion: --- Object Response

// region:    --- Chunk Response
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChatCompletionChunkResponse {
	/// A unique identifier for the chat completion.
	pub id: String,
	/// A list of chat completion choices. Can be more than one if n is greater than 1.
	pub choices: Vec<ChatCompletionChunkResponseChoice>,
	/// The Unix timestamp (in seconds) of when the chat completion
	pub created: u64,
	/// The model used for the chat completion.
	pub model: String,
	/// This fingerprint represents the backend configuration that the model runs with.
	/// Can be used in conjunction with the seed request parameter to understand when backend
	/// changes have been made that might impact determinism.
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub system_fingerprint: Option<String>,
	/// The object type, which is always chat.completion.
	pub object: String,
	/// An optional field that will only be present when you set stream_options: {"include_usage":
	/// true} in your request. When present, it contains a null value except for the last chunk
	/// which contains the token usage statistics for the entire request.
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub usage: Option<ChatCompletionResponseUsage>,
	/// The service tier used for processing the request. This field is only included if the
	/// service_tier parameter is specified in the request.
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub service_tier: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChatCompletionChunkResponseChoice {
	/// The reason the model stopped generating tokens. This will be stop if the model hit a
	/// natural stop point or a provided stop sequence, length if the maximum number of tokens
	/// specified in the request was reached, content_filter if content was omitted due to a flag
	/// from our content filters, tool_calls if the model called a tool, or function_call
	/// (deprecated) if the model called a function.
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub finish_reason: Option<String>,

	/// The index of the choice in the list of choices.
	pub index: u64,

	// A chat completion delta generated by streamed model responses.
	pub delta: ChatCompletionChunkResponseChoiceDelta,

	/// Log probability information for the choice.
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub logprobs: Option<ChatCompletionResponseChoiceLogprob>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChatCompletionChunkResponseChoiceDelta {
	/// The contents of the message
	pub content: Option<String>,
	/// The role of the author of the message
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub role: Option<String>,
	/// The tool calls generated by the model, such as function calls.
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub tool_calls: Option<Vec<ChatCompletionChunkResponseChoiceToolCall>>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(tag = "type"))]
pub enum ChatCompletionChunkResponseChoiceToolCall {
	#[cfg_attr(feature = "serde", serde(rename = "function", alias = "function"))]
	FunctionTool { index: u64, id: String, function: ChatCompletionResponseChoiceFunctionToolCall },
}
// endregion: --- Chunk Response

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChatCompletionResponseUsage {
	/// Number of tokens in the generated completion.
	completion_tokens: u64,
	/// Number of tokens in the prompt.
	prompt_tokens: u64,
	/// Total number of tokens used in the request (prompt + completion).
	total_tokens: u64,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChatCompletionResponseChoiceLogprob {
	/// The contents of the message
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub content: Option<Vec<ChatCompletionResponseChoiceLogprobContent>>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChatCompletionResponseChoiceLogprobContent {
	/// The token
	pub token: String,
	/// The log probability of this token, if it is within the top 20 most likely tokens.
	/// Otherwise, the value -9999.0 is used to signify that the token is very unlikely.
	pub logprob: f64,
	/// A list of integers representing the UTF-8 bytes representation of the token. Useful in
	/// instances where characters are represented by multiple tokens and their byte
	/// representations must be combined to generate the correct text representation. Can be null
	/// if there is no bytes representation for the token
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub bytes: Option<Vec<u8>>,
	/// List of the most likely tokens and their log probability, at this token position. In rare
	/// cases, there may be fewer than the number of requested top_logprobs returned.
	pub top_logprobs: Vec<ChatCompletionResponseChoiceTopLogprobContent>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChatCompletionResponseChoiceTopLogprobContent {
	/// The token
	pub token: String,
	/// The log probability of this token, if it is within the top 20 most likely tokens.
	/// Otherwise, the value -9999.0 is used to signify that the token is very unlikely.
	pub logprob: f64,
	/// A list of integers representing the UTF-8 bytes representation of the token. Useful in
	/// instances where characters are represented by multiple tokens and their byte
	/// representations must be combined to generate the correct text representation. Can be null
	/// if there is no bytes representation for the token
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub bytes: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChatCompletionResponseChoiceFunctionToolCall {
	name: String,
	arguments: String,
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>; // For early tests.

	use super::*;
	use serde_json::json;

	#[test]
	fn test_response_object_example_01_decode_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_request = json!({
		  "id": "chatcmpl-123",
		  "object": "chat.completion",
		  "created": 1677652288,
		  "model": "gpt-3.5-turbo-0125",
		  "system_fingerprint": "fp_44709d6fcb",
		  "choices": [{
			"index": 0,
			"message": {
			  "role": "assistant",
			  "content": "\n\nHello there, how may I assist you today?",
			},
			"logprobs": null,
			"finish_reason": "stop"
		  }],
		  "usage": {
			"prompt_tokens": 9,
			"completion_tokens": 12,
			"total_tokens": 21
		  }
		}
		)
		.to_string();

		let _: ChatCompletionObjectResponse = serde_json::from_str(&fx_request).unwrap();

		Ok(())
	}

	#[test]
	fn test_response_object_example_02_decode_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_request = json!({
		  "id": "chatcmpl-123",
		  "object": "chat.completion",
		  "created": 1677652288,
		  "model": "gpt-3.5-turbo-0125",
		  "system_fingerprint": "fp_44709d6fcb",
		  "choices": [{
			"index": 0,
			"message": {
			  "role": "assistant",
			  "content": "\n\nThis image shows a wooden boardwalk extending through a lush green marshland.",
			},
			"logprobs": null,
			"finish_reason": "stop"
		  }],
		  "usage": {
			"prompt_tokens": 9,
			"completion_tokens": 12,
			"total_tokens": 21
		  }
		}
		)
		.to_string();

		let _: ChatCompletionObjectResponse = serde_json::from_str(&fx_request).unwrap();

		Ok(())
	}

	#[test]
	fn test_response_object_example_03_decode_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_request = json!({
		  "id": "chatcmpl-abc123",
		  "object": "chat.completion",
		  "created": 1699896916,
		  "model": "gpt-3.5-turbo-0125",
		  "choices": [
			{
			  "index": 0,
			  "message": {
				"role": "assistant",
				"content": null,
				"tool_calls": [
				  {
					"id": "call_abc123",
					"type": "function",
					"function": {
					  "name": "get_current_weather",
					  "arguments": "{\n\"location\": \"Boston, MA\"\n}"
					}
				  }
				]
			  },
			  "logprobs": null,
			  "finish_reason": "tool_calls"
			}
		  ],
		  "usage": {
			"prompt_tokens": 82,
			"completion_tokens": 17,
			"total_tokens": 99
		  }
		}
		)
		.to_string();

		let _: ChatCompletionObjectResponse = serde_json::from_str(&fx_request).unwrap();

		Ok(())
	}

	#[test]
	fn test_response_object_example_04_decode_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_request = json!({
		  "id": "chatcmpl-123",
		  "object": "chat.completion",
		  "created": 1702685778,
		  "model": "gpt-3.5-turbo-0125",
		  "choices": [
			{
			  "index": 0,
			  "message": {
				"role": "assistant",
				"content": "Hello! How can I assist you today?"
			  },
			  "logprobs": {
				"content": [
				  {
					"token": "Hello",
					"logprob": -0.31725305,
					"bytes": [72, 101, 108, 108, 111],
					"top_logprobs": [
					  {
						"token": "Hello",
						"logprob": -0.31725305,
						"bytes": [72, 101, 108, 108, 111]
					  },
					  {
						"token": "Hi",
						"logprob": -1.3190403,
						"bytes": [72, 105]
					  }
					]
				  },
				  {
					"token": "!",
					"logprob": -0.02380986,
					"bytes": [
					  33
					],
					"top_logprobs": [
					  {
						"token": "!",
						"logprob": -0.02380986,
						"bytes": [33]
					  },
					  {
						"token": " there",
						"logprob": -3.787621,
						"bytes": [32, 116, 104, 101, 114, 101]
					  }
					]
				  },
				  {
					"token": " How",
					"logprob": -0.000054669687,
					"bytes": [32, 72, 111, 119],
					"top_logprobs": [
					  {
						"token": " How",
						"logprob": -0.000054669687,
						"bytes": [32, 72, 111, 119]
					  },
					  {
						"token": "<|end|>",
						"logprob": -10.953937,
						"bytes": null
					  }
					]
				  },
				  {
					"token": " can",
					"logprob": -0.015801601,
					"bytes": [32, 99, 97, 110],
					"top_logprobs": [
					  {
						"token": " can",
						"logprob": -0.015801601,
						"bytes": [32, 99, 97, 110]
					  },
					  {
						"token": " may",
						"logprob": -4.161023,
						"bytes": [32, 109, 97, 121]
					  }
					]
				  },
				  {
					"token": " I",
					"logprob": -3.7697225e-6,
					"bytes": [
					  32,
					  73
					],
					"top_logprobs": [
					  {
						"token": " I",
						"logprob": -3.7697225e-6,
						"bytes": [32, 73]
					  },
					  {
						"token": " assist",
						"logprob": -13.596657,
						"bytes": [32, 97, 115, 115, 105, 115, 116]
					  }
					]
				  },
				  {
					"token": " assist",
					"logprob": -0.04571125,
					"bytes": [32, 97, 115, 115, 105, 115, 116],
					"top_logprobs": [
					  {
						"token": " assist",
						"logprob": -0.04571125,
						"bytes": [32, 97, 115, 115, 105, 115, 116]
					  },
					  {
						"token": " help",
						"logprob": -3.1089056,
						"bytes": [32, 104, 101, 108, 112]
					  }
					]
				  },
				  {
					"token": " you",
					"logprob": -5.4385737e-6,
					"bytes": [32, 121, 111, 117],
					"top_logprobs": [
					  {
						"token": " you",
						"logprob": -5.4385737e-6,
						"bytes": [32, 121, 111, 117]
					  },
					  {
						"token": " today",
						"logprob": -12.807695,
						"bytes": [32, 116, 111, 100, 97, 121]
					  }
					]
				  },
				  {
					"token": " today",
					"logprob": -0.0040071653,
					"bytes": [32, 116, 111, 100, 97, 121],
					"top_logprobs": [
					  {
						"token": " today",
						"logprob": -0.0040071653,
						"bytes": [32, 116, 111, 100, 97, 121]
					  },
					  {
						"token": "?",
						"logprob": -5.5247097,
						"bytes": [63]
					  }
					]
				  },
				  {
					"token": "?",
					"logprob": -0.0008108172,
					"bytes": [63],
					"top_logprobs": [
					  {
						"token": "?",
						"logprob": -0.0008108172,
						"bytes": [63]
					  },
					  {
						"token": "?\n",
						"logprob": -7.184561,
						"bytes": [63, 10]
					  }
					]
				  }
				]
			  },
			  "finish_reason": "stop"
			}
		  ],
		  "usage": {
			"prompt_tokens": 9,
			"completion_tokens": 9,
			"total_tokens": 18
		  },
		  "system_fingerprint": null
		}
		)
		.to_string();

		let _: ChatCompletionObjectResponse = serde_json::from_str(&fx_request).unwrap();

		Ok(())
	}

	#[test]
	fn test_response_chunk_example_01_decode_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_request = json!(
		  {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1694268190,"model":"gpt-3.5-turbo-0125", "system_fingerprint": "fp_44709d6fcb", "choices":[{"index":0,"delta":{"role":"assistant","content":""},"logprobs":null,"finish_reason":null}]}
		)
            .to_string();

		let _: ChatCompletionChunkResponse = serde_json::from_str(&fx_request).unwrap();

		Ok(())
	}

	#[test]
	fn test_response_chunk_example_02_decode_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_request = json!(
		  {
			  "id":"chatcmpl-123",
			  "object":"chat.completion.chunk",
			  "created":1694268190,
			  "model":"gpt-3.5-turbo-0125",
			  "system_fingerprint": "fp_44709d6fcb",
			  "choices":[
				  {
					  "index":0,
					  "delta": {"content":"Hello"},
					  "logprobs":null,
					  "finish_reason":null
				  }
			  ]
		  }
		)
		.to_string();

		let _: ChatCompletionChunkResponse = serde_json::from_str(&fx_request).unwrap();

		Ok(())
	}

	#[test]
	fn test_response_chunk_example_03_decode_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_request = json!(
			{"id":"chatcmpl-123","object":"chat.completion.chunk","created":1694268190,"model":"gpt-3.5-turbo-0125", "system_fingerprint": "fp_44709d6fcb", "choices":[{"index":0,"delta":{},"logprobs":null,"finish_reason":"stop"}]}
		)
            .to_string();

		let _: ChatCompletionChunkResponse = serde_json::from_str(&fx_request).unwrap();

		Ok(())
	}
}

// endregion:    --- Tests
