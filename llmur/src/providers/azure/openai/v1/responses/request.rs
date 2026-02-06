use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::providers::azure::openai::v1::responses::types::{
    ConversationParam, Includable, Input, Prompt, PromptCacheRetention, Reasoning,
    ResponseStreamOptions, TextConfig, Tool, ToolChoice, Truncation,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Request {
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
    pub previous_response_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<Prompt>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<Truncation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Input>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<Includable>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<ResponseStreamOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<ConversationParam>,
}
