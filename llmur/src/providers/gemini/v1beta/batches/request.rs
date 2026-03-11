use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchCreateRequest {
    pub batch: BatchCreateBody,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchCreateBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub input_config: BatchInputConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchInputConfig {
    pub file_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_create_request_serializes_with_camel_case_input_config() {
        let request = BatchCreateRequest {
            batch: BatchCreateBody {
                display_name: Some("daily-batch".to_string()),
                input_config: BatchInputConfig {
                    file_name: "files/input-1".to_string(),
                },
            },
        };

        let value = serde_json::to_value(&request).unwrap();
        let batch = value.get("batch").unwrap();
        assert_eq!(
            batch.get("displayName").and_then(|v| v.as_str()),
            Some("daily-batch")
        );
        assert_eq!(
            batch
                .get("inputConfig")
                .and_then(|v| v.get("fileName"))
                .and_then(|v| v.as_str()),
            Some("files/input-1")
        );
    }
}
