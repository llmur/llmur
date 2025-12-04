use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub(crate) struct RequestLimits {
    pub(crate) requests_per_minute: Option<i64>,
    pub(crate) requests_per_hour: Option<i64>,
    pub(crate) requests_per_day: Option<i64>,
    pub(crate) requests_per_week: Option<i64>,
    pub(crate) requests_per_month: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub(crate) struct BudgetLimits {
    pub(crate) cost_per_minute: Option<f64>,
    pub(crate) cost_per_hour: Option<f64>,
    pub(crate) cost_per_day: Option<f64>,
    pub(crate) cost_per_week: Option<f64>,
    pub(crate) cost_per_month: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub(crate) struct TokenLimits {
    pub(crate) tokens_per_minute: Option<i64>,
    pub(crate) tokens_per_hour: Option<i64>,
    pub(crate) tokens_per_day: Option<i64>,
    pub(crate) tokens_per_week: Option<i64>,
    pub(crate) tokens_per_month: Option<i64>,
}
