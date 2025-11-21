use serde::{Deserialize, Serialize};

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
