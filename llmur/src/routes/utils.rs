use crate::data::limits::{BudgetLimits, RequestLimits, TokenLimits};
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
