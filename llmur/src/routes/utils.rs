use crate::data::limits::{BudgetLimits, RequestLimits, TokenLimits};
use crate::data::virtual_batch::VirtualBatchId;
use crate::data::virtual_file::VirtualFileId;
use crate::errors::LLMurError;
use crate::routes::middleware::auth::{AuthorizationHeader, AuthorizationHeaderExtractionResult};
use serde::Deserialize;
use serde::de::Deserializer;

pub(crate) enum NullableField<T> {
    Missing,
    Value(T),
    Null,
}

impl<T> Default for NullableField<T> {
    fn default() -> Self {
        NullableField::Missing
    }
}

impl<'de, T> Deserialize<'de> for NullableField<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Option::<T>::deserialize(deserializer)?;
        Ok(match value {
            Some(value) => NullableField::Value(value),
            None => NullableField::Null,
        })
    }
}

impl<T> NullableField<T> {
    pub(crate) fn into_option(self) -> Option<Option<T>> {
        match self {
            NullableField::Missing => None,
            NullableField::Value(value) => Some(Some(value)),
            NullableField::Null => Some(None),
        }
    }
}

pub(crate) trait LimitsEmpty {
    fn is_empty(&self) -> bool;
}

impl LimitsEmpty for BudgetLimits {
    fn is_empty(&self) -> bool {
        BudgetLimits::is_empty(self)
    }
}

impl LimitsEmpty for RequestLimits {
    fn is_empty(&self) -> bool {
        RequestLimits::is_empty(self)
    }
}

impl LimitsEmpty for TokenLimits {
    fn is_empty(&self) -> bool {
        TokenLimits::is_empty(self)
    }
}

pub(crate) fn normalize_nullable<T>(value: NullableField<T>) -> Option<Option<T>> {
    value.into_option()
}

pub(crate) fn normalize_limits<T: LimitsEmpty>(value: NullableField<T>) -> Option<Option<T>> {
    match value {
        NullableField::Missing => None,
        NullableField::Value(value) => {
            if value.is_empty() {
                Some(None)
            } else {
                Some(Some(value))
            }
        }
        NullableField::Null => Some(None),
    }
}

pub(crate) fn parse_virtual_batch_id(value: &str) -> Result<VirtualBatchId, LLMurError> {
    parse_bad_request(value, "Invalid batch id value.")
}

pub(crate) fn parse_virtual_file_id(value: &str) -> Result<VirtualFileId, LLMurError> {
    parse_bad_request(value, "Invalid file id value.")
}

pub(crate) fn extract_api_key(
    auth: AuthorizationHeaderExtractionResult,
) -> Result<String, LLMurError> {
    let auth_header = auth?;
    match auth_header {
        AuthorizationHeader::Bearer(api_key) => Ok(api_key),
    }
}

fn parse_bad_request<T: std::str::FromStr>(
    value: &str,
    invalid_value_message: &'static str,
) -> Result<T, LLMurError> {
    value
        .parse::<T>()
        .map_err(|_| LLMurError::BadRequest(invalid_value_message.to_string()))
}
