use crate::data::connection::Connection;
use crate::data::deployment::Deployment;
use crate::data::graph::local_store::GraphData;
use crate::data::project::Project;
use crate::data::virtual_key::VirtualKey;
use crate::data::WithIdParameter;
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use serde::Serialize;
use std::collections::BTreeMap;
use std::marker::PhantomData;

// ---------- type-level tags ----------
#[derive(Clone, Debug, Default, Serialize)]
struct Budget;
#[derive(Clone, Debug, Default, Serialize)]
struct Requests;
#[derive(Clone, Debug, Default, Serialize)]
struct Tokens;


#[derive(Clone, Debug, Default, Serialize)]
struct CurrentMonth;
#[derive(Clone, Debug, Default, Serialize)]
struct CurrentDay;
#[derive(Clone, Debug, Default, Serialize)]
struct CurrentHour;
#[derive(Clone, Debug, Default, Serialize)]
struct CurrentMinute;

// ---------- traits ----------
trait MetricTag { const STR: &'static str; }
trait ResourceTag { const STR: &'static str; }
trait PeriodTag {
    fn floor(ts: &DateTime<Utc>) -> DateTime<Utc>;
    fn fmt(ts: &DateTime<Utc>) -> String { ts.timestamp().to_string() }
}

impl MetricTag for Budget   { const STR: &'static str = "budget"; }
impl MetricTag for Requests { const STR: &'static str = "requests"; }
impl MetricTag for Tokens   { const STR: &'static str = "tokens"; }

impl ResourceTag for VirtualKey { const STR: &'static str = "virtualkey"; }
impl ResourceTag for Deployment { const STR: &'static str = "deployment"; }
impl ResourceTag for Connection { const STR: &'static str = "connection"; }
impl ResourceTag for Project    { const STR: &'static str = "project"; }

impl PeriodTag for CurrentMonth {
    fn floor(ts: &DateTime<Utc>) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(ts.year(), ts.month(), 1, 0, 0, 0).unwrap()
    }
}
impl PeriodTag for CurrentDay {
    fn floor(ts: &DateTime<Utc>) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(ts.year(), ts.month(), ts.day(), 0, 0, 0).unwrap()
    }
}
impl PeriodTag for CurrentHour {
    fn floor(ts: &DateTime<Utc>) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(ts.year(), ts.month(), ts.day(), ts.hour(), 0, 0).unwrap()
    }
}
impl PeriodTag for CurrentMinute {
    fn floor(ts: &DateTime<Utc>) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(ts.year(), ts.month(), ts.day(), ts.hour(), ts.minute(), 0).unwrap()
    }
}

// ---------- Generic Stat ----------
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Serialize)]
struct UsageStat<Resource, Metric, Period, ValueType>(String, ValueType, PhantomData<(Metric, Resource, Period)>);

impl<R, M, P, V> UsageStat<R, M, P, V>
where
    M: MetricTag,
    P: PeriodTag,
    V: std::fmt::Display,
{
    /// "stats:<resource>:<id>:<metric>:<ts-rounded-down>"
    pub fn generate_key<I>(id: &I, now_utc: &DateTime<Utc>) -> String
    where
        R: ResourceTag + WithIdParameter<I>, I: std::fmt::Display
    {
        let rounded = P::floor(now_utc);
        format!("stats:{}:{}:{}:{}", R::STR, id, M::STR, P::fmt(&rounded))
    }

    pub fn get_key(self) -> String { self.0 }
    pub fn get_key_ref(&self) -> &str { &self.0 }

    pub fn get_value(self) -> V { self.1 }
    pub fn get_value_ref(&self) -> &V { &self.1 }

    pub fn extract_stat<I>(id: &I, now_utc: &DateTime<Utc>, data: &BTreeMap<String, Option<String>>) -> UsageStat<R, M, P, V>
    where
        R: ResourceTag + WithIdParameter<I>, I: std::fmt::Display,
        V: std::str::FromStr + Default,
    {
        let key = Self::generate_key(id, now_utc);
        let value = data.get(&key)
            .and_then(|opt_str| opt_str.as_ref())
            .and_then(|s| s.parse::<V>().ok())
            .unwrap_or_default();
        UsageStat(key, value, PhantomData)
    }
}

// region:    --- Usage Stats Model
#[derive(Clone, Debug, Default, Serialize)]
struct PeriodsUsageStats<Resource, Metric, ValueType> {
    current_minute: UsageStat<Resource, Metric, CurrentMinute, ValueType>,
    current_hour: UsageStat<Resource, Metric, CurrentHour, ValueType>,
    current_day: UsageStat<Resource, Metric, CurrentDay, ValueType>,
    current_month: UsageStat<Resource, Metric, CurrentMonth, ValueType>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub(crate) struct MetricsUsageStats<Resource> {
    requests: PeriodsUsageStats<Resource, Requests, u64>,
    budget: PeriodsUsageStats<Resource, Budget, f32>,
    tokens: PeriodsUsageStats<Resource, Tokens, u64>,
}


pub(crate) type VirtualKeyUsageStats = MetricsUsageStats<VirtualKey>;
pub(crate) type DeploymentUsageStats = MetricsUsageStats<Deployment>;
pub(crate) type ProjectUsageStats    = MetricsUsageStats<Project>;
pub(crate) type ConnectionUsageStats = MetricsUsageStats<Connection>;


impl<R, M, V> PeriodsUsageStats<R, M, V>
where
    M: MetricTag,
    V: std::fmt::Display,
{
    fn generate_all_keys<I>(id: &I, now_utc: &DateTime<Utc>) -> Vec<String>
    where
        R: ResourceTag + WithIdParameter<I>, I: std::fmt::Display
    {
        vec![
            UsageStat::<R, M, CurrentMinute, V>::generate_key(id, now_utc),
            UsageStat::<R, M, CurrentHour, V>::generate_key(id, now_utc),
            UsageStat::<R, M, CurrentDay, V>::generate_key(id, now_utc),
            UsageStat::<R, M, CurrentMonth, V>::generate_key(id, now_utc),
        ]
    }
}


impl<R> MetricsUsageStats<R>
where
    R: ResourceTag,
{
    pub(crate) fn generate_all_keys<I>(id: &I, now_utc: &DateTime<Utc>) -> Vec<String>
    where
        R: ResourceTag + WithIdParameter<I>, I: std::fmt::Display
    {
        let mut keys = Vec::with_capacity(3 * 4);
        keys.extend(PeriodsUsageStats::<R, Requests, u64>::generate_all_keys(id, now_utc));
        keys.extend(PeriodsUsageStats::<R, Budget, f32>::generate_all_keys(id, now_utc));
        keys.extend(PeriodsUsageStats::<R, Tokens, u64>::generate_all_keys(id, now_utc));
        keys
    }

    fn generate_all_keys_for_vector<I>(id: Vec<&I>, now_utc: &DateTime<Utc>) -> Vec<String>
    where
        R: ResourceTag + WithIdParameter<I>, I: std::fmt::Display
    {
        let mut keys = Vec::with_capacity(id.len() * 3 * 4);
        for single_id in id {
            keys.extend(PeriodsUsageStats::<R, Requests, u64>::generate_all_keys(single_id, now_utc));
            keys.extend(PeriodsUsageStats::<R, Budget, f32>::generate_all_keys(single_id, now_utc));
            keys.extend(PeriodsUsageStats::<R, Tokens, u64>::generate_all_keys(single_id, now_utc));
        }
        keys
    }
}

// endregion: --- Usage Stats Model

// region:    --- Graph Model Usage Stats Keys Generation
impl GraphData {
    pub fn generate_all_usage_stats_keys(&self, now_utc: &DateTime<Utc>) -> Vec<String> {
        let mut keys = Vec::with_capacity((self.connections.len() * 3 * 4) + (3 * 3 * 4));
        keys.extend(MetricsUsageStats::<VirtualKey>::generate_all_keys(&self.virtual_key.id, now_utc));
        keys.extend(MetricsUsageStats::<Deployment>::generate_all_keys(&self.deployment.id, now_utc));
        keys.extend(MetricsUsageStats::<Project>::generate_all_keys(&self.project.id, now_utc));
        keys.extend(MetricsUsageStats::<Connection>::generate_all_keys_for_vector(
            self.connections.iter().map(|c| &c.id).collect(),
            now_utc
        ));
        keys
    }
}
// endregion: --- Graph Model Usage Stats Keys Generation

// region:    --- Builder for Usage Stats
impl<Resource> MetricsUsageStats<Resource>
where
    Resource: ResourceTag,
{
    pub fn extract_from_map<I>(id: &I, now_utc: &DateTime<Utc>, data: &BTreeMap<String, Option<String>>) -> MetricsUsageStats<Resource>
    where
        Resource: WithIdParameter<I>,
        I: std::fmt::Display,
    {
        MetricsUsageStats {
            requests: PeriodsUsageStats {
                current_minute: UsageStat::<Resource, Requests, CurrentMinute, u64>::extract_stat(id, now_utc, data),
                current_hour:   UsageStat::<Resource, Requests, CurrentHour, u64>::extract_stat(id, now_utc, data),
                current_day:    UsageStat::<Resource, Requests, CurrentDay, u64>::extract_stat(id, now_utc, data),
                current_month:  UsageStat::<Resource, Requests, CurrentMonth, u64>::extract_stat(id, now_utc, data),
            },
            budget: PeriodsUsageStats {
                current_minute: UsageStat::<Resource, Budget, CurrentMinute, f32>::extract_stat(id, now_utc, data),
                current_hour:   UsageStat::<Resource, Budget, CurrentHour, f32>::extract_stat(id, now_utc, data),
                current_day:    UsageStat::<Resource, Budget, CurrentDay, f32>::extract_stat(id, now_utc, data),
                current_month:  UsageStat::<Resource, Budget, CurrentMonth, f32>::extract_stat(id, now_utc, data),
            },
            tokens: PeriodsUsageStats {
                current_minute: UsageStat::<Resource, Tokens, CurrentMinute, u64>::extract_stat(id, now_utc, data),
                current_hour:   UsageStat::<Resource, Tokens, CurrentHour, u64>::extract_stat(id, now_utc, data),
                current_day:    UsageStat::<Resource, Tokens, CurrentDay, u64>::extract_stat(id, now_utc, data),
                current_month:  UsageStat::<Resource, Tokens, CurrentMonth, u64>::extract_stat(id, now_utc, data),
            },
        }
    }
}
