use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Stop {
    String(String),
    Array(Vec<String>),
}

pub mod from_openai_transform {
    use crate::providers::azure::openai::v2024_02_01::chat_completions::stop::{Stop as AzureStop};
    use crate::providers::openai::chat_completions::stop::{Stop as OpenAiStop};

    impl From<OpenAiStop> for AzureStop {
        fn from(value: OpenAiStop) -> Self {
            match value {
                OpenAiStop::String(s) => { AzureStop::String(s) }
                OpenAiStop::Array(v) => { AzureStop::Array(v) }
            }
        }
    }
}