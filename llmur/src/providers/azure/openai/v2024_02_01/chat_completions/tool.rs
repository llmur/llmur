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


pub mod from_openai_transform {
    use crate::providers::azure::openai::v2024_02_01::chat_completions::tool::{
        Tool as AzureTool,
        ToolFunction as AzureToolFunction,
        ToolChoice as AzureToolChoice,
        ToolChoiceFunction as AzureToolChoiceFunction,
        ToolChoiceFunctionDetails as AzureToolChoiceFunctionDetails,
    };
    use crate::providers::openai::chat_completions::tool::{Tool as OpenAiTool, ToolFunction as OpenAiToolFunction, ToolChoice as OpenAiToolChoice, ToolChoiceFunction as OpenAiToolChoiceFunction, ToolChoiceFunctionDetails as OpenAiToolChoiceFunctionDetails};

    impl From<OpenAiTool> for AzureTool {
        fn from(value: OpenAiTool) -> Self {
            match value {
                OpenAiTool::Function { function } => AzureTool::Function{ function: function.into() }
            }
        }
    }
    impl From<OpenAiToolFunction> for AzureToolFunction {
        fn from(value: OpenAiToolFunction) -> Self {
            AzureToolFunction {
                name: value.name,
                description: value.description,
                parameters: value.parameters,
            }
        }
    }
    impl From<OpenAiToolChoice> for AzureToolChoice {
        fn from(value: OpenAiToolChoice) -> Self {
            match value {
                OpenAiToolChoice::String(s) => { AzureToolChoice::String(s) }
                OpenAiToolChoice::Function(f) => { AzureToolChoice::Function(f.into()) }
            }
        }
    }

    impl From<OpenAiToolChoiceFunction> for AzureToolChoiceFunction {
        fn from(value: OpenAiToolChoiceFunction) -> Self {
            match value {
                OpenAiToolChoiceFunction::FunctionTool { function } => { AzureToolChoiceFunction::FunctionTool { function: function.into() } }
            }
        }
    }
    impl From<OpenAiToolChoiceFunctionDetails> for AzureToolChoiceFunctionDetails {
        fn from(value: OpenAiToolChoiceFunctionDetails) -> Self {
            AzureToolChoiceFunctionDetails {
                name: value.name,
            }
        }
    }
}