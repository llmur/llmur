use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AzureChatExtensionConfiguration {
    #[serde(rename = "azure_cosmos_db", alias = "azure_cosmos_db")]
    AzureCosmosDB { parameters: serde_json::Value },

    #[serde(rename = "azure_search", alias = "azure_search")]
    AzureSearch { parameters: serde_json::Value },
}