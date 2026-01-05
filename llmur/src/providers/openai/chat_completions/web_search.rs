use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct WebSearchOptions {
    pub search_context_size: Option<SearchContextSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<UserLocation>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum SearchContextSize {
    Low,
    Medium,
    High,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UserLocation {
    #[serde(rename = "approximate", alias = "approximate")]
    Approximate {
        city: Option<String>,
        country: Option<String>,
        region: Option<String>,
        timezone: Option<String>,
    }
}
