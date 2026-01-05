use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>
}


pub mod from_openai_transform {
    use crate::providers::azure::openai::v2024_10_21::chat_completions::stream::{
        StreamOptions as AzureStreamOptions,
    };
    use crate::providers::openai::chat_completions::stream::{
        StreamOptions as OpenAiStreamOptions,
    };

    impl From<OpenAiStreamOptions> for AzureStreamOptions {
        fn from(value: OpenAiStreamOptions) -> Self {
            AzureStreamOptions {
                include_usage: value.include_usage,
            }
        }
    }
}