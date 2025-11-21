use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub(crate) struct RequestLimits {
    requests_per_minute: Option<u64>,
    requests_per_hour: Option<u64>,
    requests_per_day: Option<u64>,
    requests_per_week: Option<u64>,
    requests_per_month: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub(crate) struct BudgetLimits {
    cost_per_minute: Option<f32>,
    cost_per_hour: Option<f32>,
    cost_per_day: Option<f32>,
    cost_per_week: Option<f32>,
    cost_per_month: Option<f32>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub(crate) struct TokenLimits {
    tokens_per_minute: Option<u64>,
    tokens_per_hour: Option<u64>,
    tokens_per_day: Option<u64>,
    tokens_per_week: Option<u64>,
    tokens_per_month: Option<u64>,
}
