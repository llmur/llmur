use crate::providers::ExposesUsage;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilePurpose {
    Assistants,
    AssistantsOutput,
    Batch,
    BatchOutput,
    #[serde(rename = "fine-tune")]
    FineTune,
    #[serde(rename = "fine-tune-results")]
    FineTuneResults,
    Vision,
    UserData,
    Evals,
}

impl FromStr for FilePurpose {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "assistants" => Ok(Self::Assistants),
            "assistants_output" => Ok(Self::AssistantsOutput),
            "batch" => Ok(Self::Batch),
            "batch_output" => Ok(Self::BatchOutput),
            "fine-tune" => Ok(Self::FineTune),
            "fine-tune-results" => Ok(Self::FineTuneResults),
            "vision" => Ok(Self::Vision),
            "user_data" => Ok(Self::UserData),
            "evals" => Ok(Self::Evals),
            _ => Err(format!("Invalid file purpose '{}'.", value)),
        }
    }
}

impl AsRef<str> for FilePurpose {
    fn as_ref(&self) -> &str {
        match self {
            Self::Assistants => "assistants",
            Self::AssistantsOutput => "assistants_output",
            Self::Batch => "batch",
            Self::BatchOutput => "batch_output",
            Self::FineTune => "fine-tune",
            Self::FineTuneResults => "fine-tune-results",
            Self::Vision => "vision",
            Self::UserData => "user_data",
            Self::Evals => "evals",
        }
    }
}

impl Display for FilePurpose {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileObjectType {
    File,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Uploaded,
    Processed,
    Error,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FileObject {
    pub id: String,
    pub bytes: u64,
    pub created_at: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,

    pub filename: String,
    pub object: FileObjectType,
    pub purpose: FilePurpose,
    pub status: FileStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_details: Option<String>,
}

impl ExposesUsage for FileObject {
    fn get_input_tokens(&self) -> u64 {
        0
    }

    fn get_output_tokens(&self) -> u64 {
        0
    }
}
