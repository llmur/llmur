use serde::{Deserialize, Serialize};

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

pub mod from_openai_transform {
    use crate::providers::azure::openai::v2024_10_21::chat_completions::format::{
        ResponseFormat as AzureResponseFormat,
        ResponseJsonSchema as AzureResponseJsonSchema,
    };
    use crate::providers::openai::chat_completions::format::{
        ResponseFormat as OpenAiResponseFormat,
        ResponseJsonSchema as OpenAiResponseJsonSchema,
    };

    impl From<OpenAiResponseFormat> for AzureResponseFormat {
        fn from(value: OpenAiResponseFormat) -> Self {
            match value {
                OpenAiResponseFormat::Text => AzureResponseFormat::Text,
                OpenAiResponseFormat::JsonObject => AzureResponseFormat::JsonObject,
                OpenAiResponseFormat::JsonSchema { json_schema } => AzureResponseFormat::JsonSchema { json_schema: json_schema.into() },
            }
        }
    }

    impl From<OpenAiResponseJsonSchema> for AzureResponseJsonSchema {
        fn from(value: OpenAiResponseJsonSchema) -> Self {
            AzureResponseJsonSchema {
                name: value.name,
                description: value.description,
                schema: value.schema,
                strict: value.strict,
            }
        }
    }
}