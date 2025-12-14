use crate::data::graph::local_store::GraphData;
use crate::data::usage::DbUsageStatsRecord;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::BTreeMap;

// ---------- Enums instead of type-level tags ----------
#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) enum Metric {
    Budget,
    Requests,
    Tokens,
}

impl Metric {
    fn as_str(&self) -> &'static str {
        match self {
            Metric::Budget => "budget",
            Metric::Requests => "requests",
            Metric::Tokens => "tokens",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) enum Period {
    CurrentMinute,
    CurrentHour,
    CurrentDay,
    CurrentMonth,
}

impl Period {
    fn format_timestamp(&self, ts: &DateTime<Utc>) -> String {
        match self {
            Period::CurrentMinute => ts.format("%Y%m%d%H%M").to_string(),
            Period::CurrentHour => ts.format("%Y%m%d%H").to_string(),
            Period::CurrentDay => ts.format("%Y%m%d").to_string(),
            Period::CurrentMonth => ts.format("%Y%m").to_string(),
        }
    }

    fn period_id(&self) -> &'static str {
        match self {
            Period::CurrentMinute => "M",
            Period::CurrentHour => "H",
            Period::CurrentDay => "d",
            Period::CurrentMonth => "m",
        }
    }

    fn all() -> [Period; 4] {
        [
            Period::CurrentMinute,
            Period::CurrentHour,
            Period::CurrentDay,
            Period::CurrentMonth,
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Resource {
    VirtualKey,
    Deployment,
    Connection,
    Project,
}

impl Resource {
    fn as_str(&self) -> &'static str {
        match self {
            Resource::VirtualKey => "virtualkey",
            Resource::Deployment => "deployment",
            Resource::Connection => "connection",
            Resource::Project => "project",
        }
    }
}

// ---------- Stat Value ----------
#[derive(Clone, Debug, Serialize)]
pub(crate) enum StatValue {
    Int(i64),
    Float(f64),
    NotSet
}

impl Default for StatValue {
    fn default() -> Self {
        StatValue::Int(0)
    }
}

impl StatValue {
    pub(crate) fn as_i64(&self) -> i64 {
        match self {
            StatValue::Int(v) => *v,
            StatValue::Float(v) => *v as i64,
            StatValue::NotSet => 0,
        }
    }

    pub(crate) fn as_f64(&self) -> f64 {
        match self {
            StatValue::Int(v) => *v as f64,
            StatValue::Float(v) => *v,
            StatValue::NotSet => 0.0,
        }
    }
}

// ---------- Usage Stat ----------
#[derive(Clone, Debug, Serialize)]
pub(crate) struct UsageStat {
    key: String,
    value: StatValue,
}

impl UsageStat {
    fn generate_key(
        resource: Resource,
        id: &impl std::fmt::Display,
        metric: Metric,
        period: Period,
        now_utc: &DateTime<Utc>,
    ) -> String {
        format!(
            "stats:{}:{}:{}:{}:{}",
            resource.as_str(),
            id,
            metric.as_str(),
            period.period_id(),
            period.format_timestamp(now_utc)
        )
    }

    fn extract_i64(
        resource: Resource,
        id: &impl std::fmt::Display,
        metric: Metric,
        period: Period,
        now_utc: &DateTime<Utc>,
        data: &BTreeMap<String, Option<String>>,
    ) -> Self {
        let key = Self::generate_key(resource, id, metric, period, now_utc);
        match data
            .get(&key)
            .and_then(|opt| opt.as_ref())
            .and_then(|s| s.parse::<i64>().ok()) {
            None => {
                UsageStat {
                    key,
                    value: StatValue::NotSet,
                }
            }
            Some(value) => {
                UsageStat {
                    key,
                    value: StatValue::Int(value),
                }
            }
        }
    }

    fn extract_f64(
        resource: Resource,
        id: &impl std::fmt::Display,
        metric: Metric,
        period: Period,
        now_utc: &DateTime<Utc>,
        data: &BTreeMap<String, Option<String>>,
    ) -> Self {
        let key = Self::generate_key(resource, id, metric, period, now_utc);
        match data
            .get(&key)
            .and_then(|opt| opt.as_ref())
            .and_then(|s| s.parse::<f64>().ok()) {
            None => {
                UsageStat {
                    key,
                    value: StatValue::NotSet,
                }
            }
            Some(value) => {
                UsageStat {
                    key,
                    value: StatValue::Float(value),
                }
            }
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &StatValue {
        &self.value
    }

    pub fn into_parts(self) -> (String, StatValue) {
        (self.key, self.value)
    }
    
    pub fn is_value_missing(&self) -> bool {
        match &self.value {
            StatValue::NotSet => true,
            _ => false
        }
    }
}

// ---------- Period Stats ----------
#[derive(Clone, Debug, Serialize)]
pub(crate) struct PeriodStats {
    pub(crate) current_minute: UsageStat,
    pub(crate) current_hour: UsageStat,
    pub(crate) current_day: UsageStat,
    pub(crate) current_month: UsageStat,
}

impl PeriodStats {
    fn has_value_missing(&self) -> bool {
        self.current_minute.is_value_missing() ||   
            self.current_hour.is_value_missing() ||
            self.current_day.is_value_missing() ||
            self.current_month.is_value_missing()
    }
    fn extract_i64(
        resource: Resource,
        id: &impl std::fmt::Display,
        metric: Metric,
        now_utc: &DateTime<Utc>,
        data: &BTreeMap<String, Option<String>>,
    ) -> Self {
        PeriodStats {
            current_minute: UsageStat::extract_i64(resource, id, metric, Period::CurrentMinute, now_utc, data),
            current_hour: UsageStat::extract_i64(resource, id, metric, Period::CurrentHour, now_utc, data),
            current_day: UsageStat::extract_i64(resource, id, metric, Period::CurrentDay, now_utc, data),
            current_month: UsageStat::extract_i64(resource, id, metric, Period::CurrentMonth, now_utc, data),
        }
    }

    fn extract_f64(
        resource: Resource,
        id: &impl std::fmt::Display,
        metric: Metric,
        now_utc: &DateTime<Utc>,
        data: &BTreeMap<String, Option<String>>,
    ) -> Self {
        PeriodStats {
            current_minute: UsageStat::extract_f64(resource, id, metric, Period::CurrentMinute, now_utc, data),
            current_hour: UsageStat::extract_f64(resource, id, metric, Period::CurrentHour, now_utc, data),
            current_day: UsageStat::extract_f64(resource, id, metric, Period::CurrentDay, now_utc, data),
            current_month: UsageStat::extract_f64(resource, id, metric, Period::CurrentMonth, now_utc, data),
        }
    }

    fn generate_keys(
        resource: Resource,
        id: &impl std::fmt::Display,
        metric: Metric,
        now_utc: &DateTime<Utc>,
    ) -> Vec<String> {
        Period::all()
            .iter()
            .map(|period| UsageStat::generate_key(resource, id, metric, *period, now_utc))
            .collect()
    }
    
    fn into_usage_stat_map(self) -> BTreeMap<String, StatValue> {
        [
            self.current_minute.into_parts(),
            self.current_hour.into_parts(),
            self.current_day.into_parts(),
            self.current_month.into_parts()
        ].into_iter().collect()
    }
}

// ---------- Metrics Usage Stats ----------
#[derive(Clone, Debug, Serialize)]
pub(crate) struct MetricsUsageStats {
    pub(crate) requests: PeriodStats,
    pub(crate) budget: PeriodStats,
    pub(crate) tokens: PeriodStats,
}

impl MetricsUsageStats {
    pub(crate) fn into_usage_stat_map(self) -> BTreeMap<String, StatValue> {
        self.requests.into_usage_stat_map().into_iter()
            .chain(self.budget.into_usage_stat_map().into_iter())
            .chain(self.tokens.into_usage_stat_map().into_iter())
            .collect()
    }
    
    pub fn has_value_missing(&self) -> bool {
        self.budget.has_value_missing() ||
            self.tokens.has_value_missing() ||
            self.requests.has_value_missing()
    }
    
    pub fn extract_from_map(
        resource: Resource,
        id: &impl std::fmt::Display,
        now_utc: &DateTime<Utc>,
        data: &BTreeMap<String, Option<String>>,
    ) -> Self {
        MetricsUsageStats {
            requests: PeriodStats::extract_i64(resource, id, Metric::Requests, now_utc, data),
            budget: PeriodStats::extract_f64(resource, id, Metric::Budget, now_utc, data),
            tokens: PeriodStats::extract_i64(resource, id, Metric::Tokens, now_utc, data),
        }
    }

    pub fn generate_all_keys(
        resource: Resource,
        id: &impl std::fmt::Display,
        now_utc: &DateTime<Utc>,
    ) -> Vec<String> {
        let mut keys = Vec::with_capacity(12);
        keys.extend(PeriodStats::generate_keys(resource, id, Metric::Requests, now_utc));
        keys.extend(PeriodStats::generate_keys(resource, id, Metric::Budget, now_utc));
        keys.extend(PeriodStats::generate_keys(resource, id, Metric::Tokens, now_utc));
        keys
    }

    pub fn generate_all_keys_with_values(
        resource: Resource,
        id: &impl std::fmt::Display,
        now_utc: &DateTime<Utc>,
        requests: i64,
        cost: f64,
        tokens: i64,
    ) -> BTreeMap<String, String> {
        let mut result = BTreeMap::new();

        for key in PeriodStats::generate_keys(resource, id, Metric::Requests, now_utc) {
            result.insert(key, requests.to_string());
        }
        for key in PeriodStats::generate_keys(resource, id, Metric::Budget, now_utc) {
            result.insert(key, cost.to_string());
        }
        for key in PeriodStats::generate_keys(resource, id, Metric::Tokens, now_utc) {
            result.insert(key, tokens.to_string());
        }

        result
    }

    pub fn generate_request_keys_with_values(
        resource: Resource,
        id: &impl std::fmt::Display,
        now_utc: &DateTime<Utc>,
        requests: i64,
    ) -> BTreeMap<String, i64> {
        PeriodStats::generate_keys(resource, id, Metric::Requests, now_utc)
            .into_iter()
            .map(|k| (k, requests))
            .collect()
    }

    pub fn generate_budget_keys_with_values(
        resource: Resource,
        id: &impl std::fmt::Display,
        now_utc: &DateTime<Utc>,
        cost: f64,
    ) -> BTreeMap<String, f64> {
        PeriodStats::generate_keys(resource, id, Metric::Budget, now_utc)
            .into_iter()
            .map(|k| (k, cost))
            .collect()
    }

    pub fn generate_token_keys_with_values(
        resource: Resource,
        id: &impl std::fmt::Display,
        now_utc: &DateTime<Utc>,
        tokens: i64,
    ) -> BTreeMap<String, i64> {
        PeriodStats::generate_keys(resource, id, Metric::Tokens, now_utc)
            .into_iter()
            .map(|k| (k, tokens))
            .collect()
    }

    fn generate_keys_for_ids(
        resource: Resource,
        ids: &[impl std::fmt::Display],
        now_utc: &DateTime<Utc>,
    ) -> Vec<String> {
        let mut keys = Vec::with_capacity(ids.len() * 12);
        for id in ids {
            keys.extend(Self::generate_all_keys(resource, id, now_utc));
        }
        keys
    }
}

// ---------- Resource-specific wrapper types ----------
macro_rules! impl_resource_usage_stats {
    ($name:ident, $resource:expr) => {
        #[derive(Clone, Debug, Serialize)]
        pub(crate) struct $name(pub MetricsUsageStats);

        impl $name {
            pub fn has_value_missing(&self) -> bool {
                self.0.has_value_missing()
            }
            
            pub fn extract_from_map(
                id: &impl std::fmt::Display,
                now_utc: &DateTime<Utc>,
                data: &BTreeMap<String, Option<String>>,
            ) -> Self {
                $name(MetricsUsageStats::extract_from_map($resource, id, now_utc, data))
            }

            pub fn generate_all_keys(
                id: &impl std::fmt::Display,
                now_utc: &DateTime<Utc>,
            ) -> Vec<String> {
                MetricsUsageStats::generate_all_keys($resource, id, now_utc)
            }

            pub fn generate_all_keys_with_values(
                id: &impl std::fmt::Display,
                now_utc: &DateTime<Utc>,
                requests: i64,
                cost: f64,
                tokens: i64,
            ) -> BTreeMap<String, String> {
                MetricsUsageStats::generate_all_keys_with_values($resource, id, now_utc, requests, cost, tokens)
            }

            pub fn generate_request_keys_with_values(
                id: &impl std::fmt::Display,
                now_utc: &DateTime<Utc>,
                requests: i64,
            ) -> BTreeMap<String, i64> {
                MetricsUsageStats::generate_request_keys_with_values($resource, id, now_utc, requests)
            }

            pub fn generate_budget_keys_with_values(
                id: &impl std::fmt::Display,
                now_utc: &DateTime<Utc>,
                cost: f64,
            ) -> BTreeMap<String, f64> {
                MetricsUsageStats::generate_budget_keys_with_values($resource, id, now_utc, cost)
            }

            pub fn generate_token_keys_with_values(
                id: &impl std::fmt::Display,
                now_utc: &DateTime<Utc>,
                tokens: i64,
            ) -> BTreeMap<String, i64> {
                MetricsUsageStats::generate_token_keys_with_values($resource, id, now_utc, tokens)
            }

            pub fn requests(&self) -> &PeriodStats {
                &self.0.requests
            }

            pub fn budget(&self) -> &PeriodStats {
                &self.0.budget
            }

            pub fn tokens(&self) -> &PeriodStats {
                &self.0.tokens
            }
            
            pub fn from_db_record(
                id: &impl std::fmt::Display,
                now_utc: &DateTime<Utc>,
                value: DbUsageStatsRecord,
            ) -> Self {
                $name(MetricsUsageStats {
                    requests: PeriodStats {
                        current_minute: UsageStat {
                            key: UsageStat::generate_key($resource, id, Metric::Requests, Period::CurrentMinute, now_utc),
                            value: StatValue::Int(value.current_minute_requests)
                        },
                        current_hour: UsageStat {
                            key: UsageStat::generate_key($resource, id, Metric::Requests, Period::CurrentHour, now_utc),
                            value: StatValue::Int(value.current_hour_requests)
                        },
                        current_day: UsageStat {
                            key: UsageStat::generate_key($resource, id, Metric::Requests, Period::CurrentDay, now_utc),
                            value: StatValue::Int(value.current_day_requests)
                        },
                        current_month: UsageStat {
                            key: UsageStat::generate_key($resource, id, Metric::Requests, Period::CurrentMonth, now_utc),
                            value: StatValue::Int(value.current_month_requests)
                        },
                    },
                    budget: PeriodStats {
                        current_minute: UsageStat {
                            key: UsageStat::generate_key($resource, id, Metric::Budget, Period::CurrentMinute, now_utc),
                            value: StatValue::Float(value.current_minute_cost)
                        },
                        current_hour: UsageStat {
                            key: UsageStat::generate_key($resource, id, Metric::Budget, Period::CurrentHour, now_utc),
                            value: StatValue::Float(value.current_hour_cost)
                        },
                        current_day: UsageStat {
                            key: UsageStat::generate_key($resource, id, Metric::Budget, Period::CurrentDay, now_utc),
                            value: StatValue::Float(value.current_day_cost)
                        },
                        current_month: UsageStat {
                            key: UsageStat::generate_key($resource, id, Metric::Budget, Period::CurrentMonth, now_utc),
                            value: StatValue::Float(value.current_month_cost)
                        },
                    },
                    tokens: PeriodStats {
                        current_minute: UsageStat {
                            key: UsageStat::generate_key($resource, id, Metric::Tokens, Period::CurrentMinute, now_utc),
                            value: StatValue::Int(value.current_minute_tokens)
                        },
                        current_hour: UsageStat {
                            key: UsageStat::generate_key($resource, id, Metric::Tokens, Period::CurrentHour, now_utc),
                            value: StatValue::Int(value.current_hour_tokens)
                        },
                        current_day: UsageStat {
                            key: UsageStat::generate_key($resource, id, Metric::Tokens, Period::CurrentDay, now_utc),
                            value: StatValue::Int(value.current_day_tokens)
                        },
                        current_month: UsageStat {
                            key: UsageStat::generate_key($resource, id, Metric::Tokens, Period::CurrentMonth, now_utc),
                            value: StatValue::Int(value.current_month_tokens)
                        },
                    },
                })
            }
        }
    };
}

impl_resource_usage_stats!(VirtualKeyUsageStats, Resource::VirtualKey);
impl_resource_usage_stats!(DeploymentUsageStats, Resource::Deployment);
impl_resource_usage_stats!(ProjectUsageStats, Resource::Project);
impl_resource_usage_stats!(ConnectionUsageStats, Resource::Connection);



// ---------- Graph Model Usage Stats Keys Generation ----------
impl GraphData {
    pub fn generate_all_usage_stats_keys(&self, now_utc: &DateTime<Utc>) -> Vec<String> {
        let mut keys = Vec::with_capacity(self.connections.len() * 12 + 36);

        keys.extend(MetricsUsageStats::generate_all_keys(
            Resource::VirtualKey,
            &self.virtual_key.id,
            now_utc,
        ));
        keys.extend(MetricsUsageStats::generate_all_keys(
            Resource::Deployment,
            &self.deployment.id,
            now_utc,
        ));
        keys.extend(MetricsUsageStats::generate_all_keys(
            Resource::Project,
            &self.project.id,
            now_utc,
        ));

        let connection_ids: Vec<_> = self.connections.iter().map(|c| &c.id).collect();
        keys.extend(MetricsUsageStats::generate_keys_for_ids(
            Resource::Connection,
            &connection_ids,
            now_utc,
        ));

        keys
    }
}