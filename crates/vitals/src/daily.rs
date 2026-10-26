//! Daily values with cross-device de-duplication.
//!
//! Wear a watch and carry a phone and both count your steps; naive sums
//! double-count. For additive metrics vitals keeps, per clock hour, only the
//! best source (by your priority list, else a wearable-first heuristic, else
//! the larger reading) — the same idea Apple Health uses internally. Sleep
//! from two trackers is resolved per night the same way.

use std::collections::{BTreeMap, HashMap};

use serde::Serialize;

use crate::{
    metric::{Aggregation, Metric},
    store::Sample,
};

/// Default ordering when the caller gives no priority list: dedicated
/// wearables beat phones, which beat everything else.
pub fn heuristic_rank(source: &str) -> u8 {
    let s = source.to_ascii_lowercase();
    const WEARABLES: [&str; 7] = ["watch", "oura", "whoop", "garmin", "fitbit", "polar", "ring"];
    if WEARABLES.iter().any(|w| s.contains(w)) {
        3
    } else if s.contains("phone") || s.contains("pixel") || s.contains("galaxy") {
        1
    } else {
        2
    }
}

fn rank(source: &str, priority: &[String]) -> (u8, u8) {
    let explicit = priority
        .iter()
        .position(|p| source.to_ascii_lowercase().contains(&p.to_ascii_lowercase()))
        .map_or(0, |i| u8::try_from(priority.len() - i).unwrap_or(u8::MAX));
    (explicit, heuristic_rank(source))
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DayValue {
    pub day: String,
    pub value: f64,
    /// Sources that contributed after de-duplication.
    pub sources: Vec<String>,
    /// How much was dropped as double-counted (additive metrics only).
    pub deduplicated: f64,
}

/// Pick the best source among `totals` (source → amount).
fn best<'a>(totals: &'a HashMap<&'a str, f64>, priority: &[String]) -> Option<(&'a str, f64)> {
    totals
        .iter()
        .max_by(|a, b| rank(a.0, priority).cmp(&rank(b.0, priority)).then(a.1.total_cmp(b.1)))
        .map(|(s, v)| (*s, *v))
}

/// Collapse samples (all of one metric) into one value per local day.
pub fn daily(metric: Metric, samples: &[Sample], priority: &[String]) -> Vec<DayValue> {
    let mut by_day: BTreeMap<&str, Vec<&Sample>> = BTreeMap::new();
    for s in samples {
        by_day.entry(s.day.as_str()).or_default().push(s);
    }
    by_day
        .into_iter()
        .map(|(day, day_samples)| {
            let (value, sources, deduplicated) = match metric.aggregation() {
                Aggregation::Sum if metric == Metric::Sleep => {
                    let mut totals: HashMap<&str, f64> = HashMap::new();
                    for s in &day_samples {
                        *totals.entry(s.source.as_str()).or_default() += s.value;
                    }
                    let all: f64 = totals.values().sum();
                    let (src, v) = best(&totals, priority).unwrap_or(("", 0.0));
                    (v, vec![src.to_string()], all - v)
                }
                Aggregation::Sum => {
                    let mut hours: BTreeMap<i64, HashMap<&str, f64>> = BTreeMap::new();
                    for s in &day_samples {
                        *hours
                            .entry(s.start.div_euclid(3600))
                            .or_default()
                            .entry(s.source.as_str())
                            .or_default() += s.value;
                    }
                    let mut kept = 0.0;
                    let mut dropped = 0.0;
                    let mut used: Vec<String> = Vec::new();
                    for totals in hours.values() {
                        if let Some((src, v)) = best(totals, priority) {
                            kept += v;
                            dropped += totals.values().sum::<f64>() - v;
                            if !used.iter().any(|u| u == src) {
                                used.push(src.to_string());
                            }
                        }
                    }
                    (kept, used, dropped)
                }
                Aggregation::Mean => {
                    let v = day_samples.iter().map(|s| s.value).sum::<f64>() / day_samples.len() as f64;
                    (v, distinct_sources(&day_samples), 0.0)
                }
                Aggregation::Min => {
                    let v = day_samples.iter().map(|s| s.value).fold(f64::INFINITY, f64::min);
                    (v, distinct_sources(&day_samples), 0.0)
                }
                Aggregation::Last => {
                    let last = day_samples.iter().max_by_key(|s| s.end).expect("non-empty day");
                    (last.value, vec![last.source.clone()], 0.0)
                }
            };
            DayValue {
                day: day.to_string(),
                value: round2(value),
                sources,
                deduplicated: round2(deduplicated),
            }
        })
        .collect()
}

fn distinct_sources(samples: &[&Sample]) -> Vec<String> {
    let mut v: Vec<String> = samples.iter().map(|s| s.source.clone()).collect();
    v.sort();
    v.dedup();
    v
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(metric: Metric, day: &str, start: i64, value: f64, source: &str) -> Sample {
        Sample {
            metric,
            start,
            end: start + 600,
            day: day.into(),
            value,
            source: source.into(),
            device: None,
        }
    }

    #[test]
    fn phone_and_watch_steps_are_not_double_counted() {
        let samples = vec![
            s(Metric::Steps, "d", 0, 1000.0, "Jo's iPhone"),
            s(Metric::Steps, "d", 600, 950.0, "Jo's Apple Watch"),
            // A later hour where only the phone was carried.
            s(Metric::Steps, "d", 7200, 300.0, "Jo's iPhone"),
        ];
        let d = daily(Metric::Steps, &samples, &[]);
        assert_eq!(d[0].value, 950.0 + 300.0, "watch wins hour 0, phone fills hour 2");
        assert_eq!(d[0].deduplicated, 1000.0);
        assert_eq!(d[0].sources, vec!["Jo's Apple Watch".to_string(), "Jo's iPhone".to_string()]);
    }

    #[test]
    fn explicit_priority_overrides_the_heuristic() {
        let samples =
            vec![s(Metric::Steps, "d", 0, 1000.0, "iPhone"), s(Metric::Steps, "d", 0, 950.0, "Apple Watch")];
        let d = daily(Metric::Steps, &samples, &["iphone".to_string()]);
        assert_eq!(d[0].value, 1000.0);
    }

    #[test]
    fn two_sleep_trackers_resolve_to_one_night() {
        let samples = vec![
            s(Metric::Sleep, "d", 0, 400.0, "Apple Watch"),
            s(Metric::Sleep, "d", 0, 30.0, "Apple Watch"),
            s(Metric::Sleep, "d", 0, 445.0, "Oura"),
        ];
        let d = daily(Metric::Sleep, &samples, &["oura".to_string()]);
        assert_eq!(d[0].value, 445.0);
        assert_eq!(d[0].deduplicated, 430.0);
    }

    #[test]
    fn mean_min_and_last_aggregations() {
        let hr = vec![s(Metric::HeartRate, "d", 0, 60.0, "W"), s(Metric::HeartRate, "d", 1, 80.0, "W")];
        assert_eq!(daily(Metric::HeartRate, &hr, &[])[0].value, 70.0);
        let rhr = vec![
            s(Metric::RestingHeartRate, "d", 0, 58.0, "W"),
            s(Metric::RestingHeartRate, "d", 1, 55.0, "W"),
        ];
        assert_eq!(daily(Metric::RestingHeartRate, &rhr, &[])[0].value, 55.0);
        let w = vec![s(Metric::Weight, "d", 10, 70.0, "Scale"), s(Metric::Weight, "d", 0, 71.0, "Scale")];
        assert_eq!(daily(Metric::Weight, &w, &[])[0].value, 70.0);
    }
}
