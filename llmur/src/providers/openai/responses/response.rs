use crate::providers::ExposesUsage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::providers::openai::responses::types::{
    OutputItem, Reasoning, ServiceTier, TextConfig, Tool, ToolChoice, Truncation,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub object: ResponseObject,
    pub created_at: u64,
    pub status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incomplete_details: Option<ResponseIncompleteDetails>,
    pub output: Vec<OutputItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ResponseUsage>,
    pub parallel_tool_calls: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextConfig>,
    pub tools: Vec<Tool>,
    pub tool_choice: ToolChoice,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<Truncation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseObject {
    Response,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Completed,
    Failed,
    InProgress,
    Incomplete,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseIncompleteDetails {
    pub reason: ResponseIncompleteReason,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ResponseIncompleteReason {
    #[serde(rename = "max_output_tokens", alias = "max_tokens")]
    MaxOutputTokens,
    #[serde(rename = "content_filter")]
    ContentFilter,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseUsage {
    pub input_tokens: u64,
    pub input_tokens_details: ResponseInputTokensDetails,
    pub output_tokens: u64,
    pub output_tokens_details: ResponseOutputTokensDetails,
    pub total_tokens: u64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseInputTokensDetails {
    pub cached_tokens: u64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseOutputTokensDetails {
    pub reasoning_tokens: u64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: ResponseErrorCode,
    pub message: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseErrorCode {
    ServerError,
    RateLimitExceeded,
    InvalidPrompt,
    VectorStoreTimeout,
    InvalidImage,
    InvalidImageFormat,
    InvalidBase64Image,
    InvalidImageUrl,
    ImageTooLarge,
    ImageTooSmall,
    ImageParseError,
    ImageContentPolicyViolation,
    InvalidImageMode,
    ImageFileTooLarge,
    UnsupportedImageMediaType,
    EmptyImageFile,
    FailedToDownloadImage,
    ImageFileNotFound,
}

impl ExposesUsage for Response {
    fn get_input_tokens(&self) -> u64 {
        self.usage
            .as_ref()
            .map(|usage| usage.input_tokens)
            .unwrap_or(0)
    }
    fn get_output_tokens(&self) -> u64 {
        self.usage
            .as_ref()
            .map(|usage| usage.output_tokens)
            .unwrap_or(0)
    }
}

pub mod to_self {
    use crate::providers::openai::responses::response::Response;
    use crate::providers::{
        Transformation, TransformationContext, TransformationLoss, Transformer,
    };

    #[derive(Debug)]
    pub struct Loss {}

    #[derive(Debug)]
    pub struct Context {
        pub model: Option<String>,
    }

    impl TransformationContext<Response, Response> for Context {}
    impl TransformationLoss<Response, Response> for Loss {}

    impl Transformer<Response, Context, Loss> for Response {
        fn transform(self, context: Context) -> Transformation<Response, Loss> {
            Transformation {
                result: Response {
                    model: context.model.unwrap_or(self.model),
                    ..self
                },
                loss: Loss {},
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Response;

    #[test]
    fn response_deserializes_with_optional_usage() {
        let json = r#"{
            "id": "resp_123",
            "object": "response",
            "created_at": 1741476542,
            "status": "completed",
            "error": null,
            "incomplete_details": null,
            "model": "gpt-4.1-2025-04-14",
            "output": [
              {
                "type": "message",
                "id": "msg_123",
                "status": "completed",
                "role": "assistant",
                "content": [
                  {
                    "type": "output_text",
                    "text": "Hello",
                    "annotations": []
                  }
                ]
              }
            ],
            "parallel_tool_calls": true,
            "previous_response_id": null,
            "reasoning": {
              "effort": null,
              "summary": null,
              "generate_summary": null
            },
            "max_output_tokens": null,
            "instructions": null,
            "text": {
              "format": {
                "type": "text"
              }
            },
            "tool_choice": "auto",
            "tools": [],
            "top_p": 1.0,
            "temperature": 1.0,
            "truncation": "disabled",
            "usage": {
              "input_tokens": 1,
              "input_tokens_details": { "cached_tokens": 0 },
              "output_tokens": 2,
              "output_tokens_details": { "reasoning_tokens": 0 },
              "total_tokens": 3
            },
            "user": null,
            "metadata": {}
          }"#;

        let response: Response = serde_json::from_str(json).expect("parse response");
        assert_eq!(response.output.len(), 1);
        assert!(response.usage.is_some());
    }
}
