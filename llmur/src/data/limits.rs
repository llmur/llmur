use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct RequestLimits {
    pub requests_per_minute: Option<i64>,
    pub requests_per_hour: Option<i64>,
    pub requests_per_day: Option<i64>,
    pub requests_per_week: Option<i64>,
    pub requests_per_month: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct BudgetLimits {
    pub cost_per_minute: Option<f64>,
    pub cost_per_hour: Option<f64>,
    pub cost_per_day: Option<f64>,
    pub cost_per_week: Option<f64>,
    pub cost_per_month: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct TokenLimits {
    pub tokens_per_minute: Option<i64>,
    pub tokens_per_hour: Option<i64>,
    pub tokens_per_day: Option<i64>,
    pub tokens_per_week: Option<i64>,
    pub tokens_per_month: Option<i64>,
}
