//! MCP tool surface for `vitals`.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use chrono::{Duration, NaiveDate};
use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ServerConfig},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    daily::{self, DayValue},
    import,
    metric::{self, Metric},
    readiness::{self, Series},
    redact::{self, Redactor},
    store::Store,
};
use mcp_kit::result::{structured, tool_error};

/// Where the database lives unless `VITALS_DB` says otherwise:
/// `$XDG_DATA_HOME/vitals/vitals.sqlite` or `~/.local/share/vitals/vitals.sqlite`.
pub fn default_db_path() -> PathBuf {
    if let Some(p) = std::env::var_os("VITALS_DB").filter(|v| !v.is_empty()) {
        return PathBuf::from(p);
    }
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("vitals").join("vitals.sqlite")
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImportArgs {
    /// Apple Health `export.zip` / `export.xml`, or a vitals-format `.csv`.
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SummaryArgs {
    /// Metrics to include. Default: steps, sleep, resting_heart_rate, hrv, active_energy.
    pub metrics: Option<Vec<Metric>>,
    /// First day, `YYYY-MM-DD`. Default: 13 days before `to`.
    pub from: Option<String>,
    /// Last day, `YYYY-MM-DD`. Default: the latest day with data.
    pub to: Option<String>,
    /// Sources to prefer when devices overlap, best first, e.g. `["watch", "oura"]`.
    #[serde(default)]
    pub source_priority: Vec<String>,
    /// Hide source names and report days only.
    #[serde(default)]
    pub redact: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadinessArgs {
    /// Day to score, `YYYY-MM-DD`. Default: the latest day with data.
    pub day: Option<String>,
    /// Nightly sleep need in hours. Default 8.
    pub sleep_need_hours: Option<f64>,
    /// Sources to prefer when devices overlap, best first.
    #[serde(default)]
    pub source_priority: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TrendArgs {
    /// Metric to chart.
    pub metric: Metric,
    /// How many days back from the latest day. Default 30, at most 365.
    pub days: Option<u32>,
    /// Sources to prefer when devices overlap, best first.
    #[serde(default)]
    pub source_priority: Vec<String>,
}

#[derive(Debug, Serialize)]
struct MetricDays {
    metric: Metric,
    unit: &'static str,
    days: Vec<DayValue>,
    double_counted_removed: f64,
}

#[derive(Clone)]
pub struct Vitals {
    store: Arc<Mutex<Store>>,
    redact_by_default: bool,
    tool_router: ToolRouter<Self>,
}

fn parse_day(s: &str) -> Result<NaiveDate, CallToolResult> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").map_err(|_| tool_error(format!("`{s}` is not a YYYY-MM-DD date.")))
}

fn fmt(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

macro_rules! try_tool {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(err) => return Ok(err),
        }
    };
}

fn db_err(e: impl std::fmt::Display) -> CallToolResult {
    tool_error(format!("Database error: {e}"))
}

impl Vitals {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        Ok(Self::with_store(Store::open(path)?, redact::from_env()))
    }

    pub fn with_store(store: Store, redact_by_default: bool) -> Self {
        Self { store: Arc::new(Mutex::new(store)), redact_by_default, tool_router: Self::tool_router() }
    }

    fn latest(&self) -> Result<NaiveDate, CallToolResult> {
        let last = self.store.lock().unwrap().last_day().map_err(db_err)?;
        match last {
            Some(d) => parse_day(&d),
            None => Err(tool_error("The warehouse is empty. Import an Apple Health export or a vitals CSV with import_health_data first.")),
        }
    }

    fn days(&self, m: Metric, from: NaiveDate, to: NaiveDate, priority: &[String]) -> Result<Vec<DayValue>, CallToolResult> {
        let samples = self.store.lock().unwrap().samples(m, &fmt(from), &fmt(to)).map_err(db_err)?;
        Ok(daily::daily(m, &samples, priority))
    }

    fn redactor(&self, requested: bool) -> Result<Redactor, CallToolResult> {
        let enabled = requested || self.redact_by_default;
        let known = if enabled {
            let o = self.store.lock().unwrap().overview().map_err(db_err)?;
            o.into_iter().flat_map(|m| m.sources.into_iter().map(|(s, _)| s)).collect()
        } else {
            Vec::new()
        };
        Ok(Redactor::new(enabled, known))
    }
}

#[tool_router]
impl Vitals {
    #[tool(
        description = "Import an Apple Health export (export.zip or export.xml) or a vitals-format CSV into the local warehouse. Re-importing the same file adds nothing. Data never leaves this machine.",
        annotations(title = "Import health data", read_only_hint = false, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    pub fn import_health_data(&self, Parameters(a): Parameters<ImportArgs>) -> Result<CallToolResult, ErrorData> {
        let path = PathBuf::from(a.path.trim());
        let (format, parsed) = match import::load(&path) {
            Ok(v) => v,
            Err(e) => return Ok(tool_error(e.to_string())),
        };
        let mut store = self.store.lock().unwrap();
        let inserted = try_tool!(store.insert(&parsed.samples).map_err(db_err));
        let _ = store.record_import(&path.display().to_string(), &format!("{format:?}"), parsed.read, inserted);
        structured(
            format!(
                "Read {} records: {} new samples stored, {} already present, {} skipped (unsupported type or unit).",
                parsed.read,
                inserted,
                parsed.samples.len() - inserted,
                parsed.skipped
            ),
            &json!({"format": format, "read": parsed.read, "parsed": parsed.samples.len(), "inserted": inserted, "skipped": parsed.skipped}),
        )
    }

    #[tool(
        description = "What is in the local warehouse: each metric with its unit, sample count, first/last day and contributing sources.",
        annotations(title = "Data overview", read_only_hint = true, open_world_hint = false)
    )]
    pub fn data_overview(&self) -> Result<CallToolResult, ErrorData> {
        let overview = try_tool!(self.store.lock().unwrap().overview().map_err(db_err));
        if overview.is_empty() {
            return Ok(tool_error("The warehouse is empty. Import data with import_health_data first."));
        }
        let r = try_tool!(self.redactor(false));
        let rows: Vec<_> = overview
            .iter()
            .map(|m| {
                json!({
                    "metric": m.metric, "unit": m.unit, "samples": m.samples,
                    "first_day": m.first_day, "last_day": m.last_day,
                    "sources": m.sources.iter().map(|(s, n)| json!({"source": r.source(s), "samples": n})).collect::<Vec<_>>()
                })
            })
            .collect();
        structured(format!("{} metric(s) stored.", rows.len()), &rows)
    }

    #[tool(
        description = "One value per day for each requested metric, with phone/watch/ring overlap de-duplicated (per hour for steps and energy, per night for sleep). Reports how much double-counting was removed.",
        annotations(title = "Daily summary", read_only_hint = true, open_world_hint = false)
    )]
    pub fn daily_summary(&self, Parameters(a): Parameters<SummaryArgs>) -> Result<CallToolResult, ErrorData> {
        let to = match &a.to {
            Some(d) => try_tool!(parse_day(d)),
            None => try_tool!(self.latest()),
        };
        let from = match &a.from {
            Some(d) => try_tool!(parse_day(d)),
            None => to - Duration::days(13),
        };
        if from > to || (to - from).num_days() > 366 {
            return Ok(tool_error("`from` must be on or before `to`, and the range at most a year."));
        }
        let metrics = a.metrics.unwrap_or_else(|| {
            vec![Metric::Steps, Metric::Sleep, Metric::RestingHeartRate, Metric::Hrv, Metric::ActiveEnergy]
        });
        let r = try_tool!(self.redactor(a.redact));
        let mut out = Vec::new();
        for m in metrics {
            let mut days = try_tool!(self.days(m, from, to, &a.source_priority));
            for d in &mut days {
                d.sources = r.sources(&d.sources);
            }
            let removed = (days.iter().map(|d| d.deduplicated).sum::<f64>() * 100.0).round() / 100.0;
            out.push(MetricDays { metric: m, unit: m.unit(), days, double_counted_removed: removed });
        }
        let removed_steps = out.iter().find(|m| m.metric == Metric::Steps).map_or(0.0, |m| m.double_counted_removed);
        structured(
            format!(
                "{} to {}: {} metric(s).{}",
                fmt(from),
                fmt(to),
                out.len(),
                if removed_steps > 0.0 { format!(" Removed {removed_steps:.0} double-counted steps.") } else { String::new() }
            ),
            &out,
        )
    }

    #[tool(
        description = "Explainable readiness for a day (0–100): HRV and resting heart rate vs your 7-day baseline, sleep vs need, and acute:chronic training load, each with its value, baseline, weight and contribution. A wellbeing heuristic, not medical advice.",
        annotations(title = "Readiness", read_only_hint = true, open_world_hint = false)
    )]
    pub fn readiness(&self, Parameters(a): Parameters<ReadinessArgs>) -> Result<CallToolResult, ErrorData> {
        let day = match &a.day {
            Some(d) => try_tool!(parse_day(d)),
            None => try_tool!(self.latest()),
        };
        let from = day - Duration::days(28);
        let to_map = |v: Vec<DayValue>| v.into_iter().map(|d| (d.day, d.value)).collect();
        let p = &a.source_priority;
        let series = Series {
            hrv: to_map(try_tool!(self.days(Metric::Hrv, from, day, p))),
            resting_hr: to_map(try_tool!(self.days(Metric::RestingHeartRate, from, day, p))),
            sleep_min: to_map(try_tool!(self.days(Metric::Sleep, from, day, p))),
            active_energy: to_map(try_tool!(self.days(Metric::ActiveEnergy, from, day, p))),
        };
        let need = a.sleep_need_hours.filter(|h| (3.0..=14.0).contains(h)).map_or(readiness::DEFAULT_SLEEP_NEED_MIN, |h| h * 60.0);
        let r = readiness::score(&series, day, need);
        let summary = match r.score {
            Some(s) => format!(
                "{}: {s:.0}/100 ({}). {}",
                r.day,
                r.label,
                r.factors.iter().map(|f| f.explanation.as_str()).collect::<Vec<_>>().join(" ")
            ),
            None => format!(
                "{}: not enough data (need at least two of HRV, resting HR, sleep, training load; missing {}).",
                r.day,
                r.missing.join(", ")
            ),
        };
        structured(summary, &r)
    }

    #[tool(
        description = "Trend of one metric over recent days: a sparkline, min/max/mean, and the change of the last 7 days vs the 7 before.",
        annotations(title = "Trend", read_only_hint = true, open_world_hint = false)
    )]
    pub fn trend(&self, Parameters(a): Parameters<TrendArgs>) -> Result<CallToolResult, ErrorData> {
        let to = try_tool!(self.latest());
        let n = a.days.unwrap_or(30).clamp(7, 365);
        let from = to - Duration::days(i64::from(n) - 1);
        let days = try_tool!(self.days(a.metric, from, to, &a.source_priority));
        if days.is_empty() {
            return Ok(tool_error(format!("No {} data between {} and {}.", a.metric.id(), fmt(from), fmt(to))));
        }
        let values: Vec<f64> = days.iter().map(|d| d.value).collect();
        let (min, max) = values.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| (lo.min(v), hi.max(v)));
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let window_mean = |a: i64, b: i64| {
            let v: Vec<f64> = days
                .iter()
                .filter(|d| parse_day(&d.day).is_ok_and(|x| (to - x).num_days() >= a && (to - x).num_days() < b))
                .map(|d| d.value)
                .collect();
            (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
        };
        let recent = window_mean(0, 7);
        let prior = window_mean(7, 14);
        let change = recent.zip(prior).filter(|(_, p)| *p != 0.0).map(|(r, p)| (r - p) / p * 100.0);
        const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let spark: String = values
            .iter()
            .map(|v| if max > min { BARS[(((v - min) / (max - min)) * 7.0).round() as usize] } else { BARS[3] })
            .collect();
        let unit = a.metric.unit();
        structured(
            format!(
                "{} {}: {spark}  min {min:.0} · mean {mean:.0} · max {max:.0} {unit}{}",
                a.metric.id(),
                fmt(from),
                change.map_or(String::new(), |c| format!(" · last 7 days {c:+.0}% vs the 7 before"))
            ),
            &json!({"metric": a.metric, "unit": unit, "from": fmt(from), "to": fmt(to), "sparkline": spark,
                    "min": min, "max": max, "mean": (mean * 10.0).round() / 10.0,
                    "change_percent": change.map(|c| (c * 10.0).round() / 10.0), "days": days}),
        )
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Vitals {
    fn get_info(&self) -> ServerConfig {
        let metrics: Vec<&str> = metric::ALL.iter().map(|m| m.id()).collect();
        mcp_kit::server_config(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            "Vitals — local health warehouse",
            &format!(
                "Health data stays in a local SQLite file. Import with import_health_data, check data_overview, \
                 then use daily_summary, trend and readiness. Metrics: {}. Readiness is a wellbeing heuristic, not medical advice.",
                metrics.join(", ")
            ),
        )
    }
}
