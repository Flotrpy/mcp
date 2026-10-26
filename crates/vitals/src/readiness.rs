//! An explainable readiness score.
//!
//! Wearable vendors publish a single number and keep the recipe secret.
//! vitals shows its work: every factor reports today's value, the personal
//! baseline it is compared with, its 0–100 sub-score, its weight and how many
//! points it contributed. Missing data drops a factor and re-weights the
//! rest instead of inventing a value.
//!
//! Factors:
//! * HRV vs the previous 7 days (higher than usual is good).
//! * Resting heart rate vs the previous 7 days (lower than usual is good).
//! * Sleep vs a nightly need (default 8 h).
//! * Training load as the acute:chronic ratio of active energy (7-day vs
//!   28-day average); 0.8–1.3 is the well-known sweet spot, above ~1.5 is a
//!   spike associated with fatigue.
//!
//! This is a wellbeing heuristic, not medical advice.

use std::collections::BTreeMap;

use chrono::{Duration, NaiveDate};
use serde::Serialize;

/// Score a factor sits at on an ordinary day.
pub const NEUTRAL: f64 = 75.0;
pub const DEFAULT_SLEEP_NEED_MIN: f64 = 480.0;
/// Fewest prior days needed for a baseline.
pub const MIN_BASELINE_DAYS: usize = 3;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Factor {
    pub name: &'static str,
    pub value: f64,
    pub baseline: Option<f64>,
    pub score: f64,
    /// Share of the total after re-weighting, 0–1.
    pub weight: f64,
    /// Points this factor added to the overall score.
    pub contribution: f64,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Readiness {
    pub day: String,
    pub score: Option<f64>,
    /// `primed`, `steady`, `go easy`, `rest` or `not enough data`.
    pub label: &'static str,
    pub factors: Vec<Factor>,
    pub missing: Vec<&'static str>,
}

/// Daily series the score needs, keyed by `YYYY-MM-DD`.
#[derive(Debug, Default, Clone)]
pub struct Series {
    pub hrv: BTreeMap<String, f64>,
    pub resting_hr: BTreeMap<String, f64>,
    pub sleep_min: BTreeMap<String, f64>,
    pub active_energy: BTreeMap<String, f64>,
}

fn day_str(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

/// Mean of `series` over the `days` days before `day` (excluding `day`).
fn prior_mean(series: &BTreeMap<String, f64>, day: NaiveDate, days: i64, min: usize) -> Option<f64> {
    let values: Vec<f64> = (1..=days).filter_map(|i| series.get(&day_str(day - Duration::days(i))).copied()).collect();
    (values.len() >= min).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn clamp(v: f64) -> f64 {
    v.clamp(0.0, 100.0)
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub fn score(series: &Series, day: NaiveDate, sleep_need_min: f64) -> Readiness {
    let key = day_str(day);
    let mut raw: Vec<(&'static str, f64, f64, Option<f64>, f64, String)> = Vec::new(); // name, weight, value, baseline, score, why
    let mut missing = Vec::new();

    match (series.hrv.get(&key), prior_mean(&series.hrv, day, 7, MIN_BASELINE_DAYS)) {
        (Some(&v), Some(base)) => {
            let ratio = v / base;
            let s = clamp(NEUTRAL + (ratio - 1.0) * 250.0);
            let pct = (ratio - 1.0) * 100.0;
            raw.push(("hrv", 0.35, v, Some(base), s, format!(
                "HRV {v:.0} ms is {:.0}% {} your 7-day average of {base:.0} ms.",
                pct.abs(), if pct >= 0.0 { "above" } else { "below" }
            )));
        }
        _ => missing.push("hrv"),
    }

    match (series.resting_hr.get(&key), prior_mean(&series.resting_hr, day, 7, MIN_BASELINE_DAYS)) {
        (Some(&v), Some(base)) => {
            let delta = v - base;
            let s = clamp(NEUTRAL - delta * 8.0);
            raw.push(("resting_heart_rate", 0.20, v, Some(base), s, format!(
                "Resting heart rate {v:.0} bpm is {:.1} bpm {} your 7-day average.",
                delta.abs(), if delta <= 0.0 { "below" } else { "above" }
            )));
        }
        _ => missing.push("resting_heart_rate"),
    }

    match series.sleep_min.get(&key) {
        Some(&v) => {
            let s = clamp(100.0 * v / sleep_need_min);
            raw.push(("sleep", 0.30, v, Some(sleep_need_min), s, format!(
                "Slept {}h {:02}m of a {:.0}h need.",
                (v / 60.0).floor() as i64, (v % 60.0).round() as i64, sleep_need_min / 60.0
            )));
        }
        None => missing.push("sleep"),
    }

    let acute = prior_mean(&series.active_energy, day + Duration::days(1), 7, 4);
    let chronic = prior_mean(&series.active_energy, day + Duration::days(1), 28, 14);
    match (acute, chronic) {
        (Some(a), Some(c)) if c > 0.0 => {
            let acwr = a / c;
            let s = if (0.8..=1.3).contains(&acwr) {
                100.0
            } else if acwr > 1.3 {
                clamp(100.0 - (acwr - 1.3) / 0.7 * 100.0)
            } else {
                clamp(100.0 - (0.8 - acwr) / 0.5 * 60.0)
            };
            let verdict = match acwr {
                r if r > 1.5 => "a spike — recent load is well above what you are used to",
                r if r > 1.3 => "slightly high",
                r if r < 0.8 => "below your usual — fine for recovery, detraining if it lasts",
                _ => "in the 0.8–1.3 sweet spot",
            };
            raw.push(("training_load", 0.15, round1(acwr * 100.0) / 100.0, Some(1.0), s, format!(
                "Acute:chronic load {acwr:.2} ({a:.0} vs {c:.0} kcal/day) is {verdict}."
            )));
        }
        _ => missing.push("training_load"),
    }

    if raw.len() < 2 {
        return Readiness { day: key, score: None, label: "not enough data", factors: Vec::new(), missing };
    }
    let total_weight: f64 = raw.iter().map(|r| r.1).sum();
    // Sum unrounded contributions; rounding each first can drift by 0.1.
    let overall = round1(raw.iter().map(|r| r.1 / total_weight * r.4).sum());
    let factors: Vec<Factor> = raw
        .into_iter()
        .map(|(name, w, value, baseline, s, explanation)| {
            let weight = w / total_weight;
            Factor {
                name,
                value: round1(value),
                baseline: baseline.map(round1),
                score: round1(s),
                weight: (weight * 1000.0).round() / 1000.0,
                contribution: round1(weight * s),
                explanation,
            }
        })
        .collect();
    let label = match overall {
        s if s >= 80.0 => "primed",
        s if s >= 60.0 => "steady",
        s if s >= 40.0 => "go easy",
        _ => "rest",
    };
    Readiness { day: key, score: Some(overall), label, factors, missing }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn fill(map: &mut BTreeMap<String, f64>, end: &str, days: i64, value: f64) {
        for i in 1..=days {
            map.insert(day_str(d(end) - Duration::days(i)), value);
        }
    }

    fn baseline_series() -> Series {
        let mut s = Series::default();
        fill(&mut s.hrv, "2026-10-20", 7, 50.0);
        fill(&mut s.resting_hr, "2026-10-20", 7, 55.0);
        fill(&mut s.active_energy, "2026-10-21", 28, 500.0);
        s
    }

    #[test]
    fn an_ordinary_well_slept_day_is_steady_and_explained() {
        let mut s = baseline_series();
        s.hrv.insert("2026-10-20".into(), 50.0);
        s.resting_hr.insert("2026-10-20".into(), 55.0);
        s.sleep_min.insert("2026-10-20".into(), 480.0);
        let r = score(&s, d("2026-10-20"), DEFAULT_SLEEP_NEED_MIN);
        assert!(r.missing.is_empty(), "{:?}", r.missing);
        assert_eq!(r.factors.len(), 4);
        // 0.35·75 + 0.20·75 + 0.30·100 + 0.15·100 = 86.25
        assert_eq!(r.score, Some(86.3));
        assert_eq!(r.label, "primed");
        let sum: f64 = r.factors.iter().map(|f| f.weight).sum();
        assert!((sum - 1.0).abs() < 0.01);
        assert!(r.factors.iter().all(|f| !f.explanation.is_empty()));
    }

    #[test]
    fn low_hrv_high_rhr_and_short_sleep_says_rest() {
        let mut s = baseline_series();
        s.hrv.insert("2026-10-20".into(), 35.0); // 30% below
        s.resting_hr.insert("2026-10-20".into(), 62.0); // +7 bpm
        s.sleep_min.insert("2026-10-20".into(), 300.0);
        let r = score(&s, d("2026-10-20"), DEFAULT_SLEEP_NEED_MIN);
        assert!(r.score.unwrap() < 40.0, "{r:?}");
        assert_eq!(r.label, "rest");
        assert!(r.factors[0].explanation.contains("30% below"));
    }

    #[test]
    fn missing_factors_are_reweighted_not_invented() {
        let mut s = Series::default();
        s.sleep_min.insert("2026-10-20".into(), 480.0);
        fill(&mut s.hrv, "2026-10-20", 7, 50.0);
        s.hrv.insert("2026-10-20".into(), 50.0);
        let r = score(&s, d("2026-10-20"), DEFAULT_SLEEP_NEED_MIN);
        assert_eq!(r.missing, vec!["resting_heart_rate", "training_load"]);
        // hrv 75 at 0.35/0.65, sleep 100 at 0.30/0.65
        assert_eq!(r.score, Some(86.5));
    }

    #[test]
    fn one_factor_is_not_enough() {
        let mut s = Series::default();
        s.sleep_min.insert("2026-10-20".into(), 480.0);
        let r = score(&s, d("2026-10-20"), DEFAULT_SLEEP_NEED_MIN);
        assert_eq!(r.score, None);
        assert_eq!(r.label, "not enough data");
    }

    #[test]
    fn a_load_spike_is_flagged() {
        let mut s = baseline_series();
        for i in 0..7 {
            s.active_energy.insert(day_str(d("2026-10-20") - Duration::days(i)), 1200.0);
        }
        s.sleep_min.insert("2026-10-20".into(), 480.0);
        let r = score(&s, d("2026-10-20"), DEFAULT_SLEEP_NEED_MIN);
        let load = r.factors.iter().find(|f| f.name == "training_load").unwrap();
        assert!(load.value > 1.5, "{load:?}");
        assert!(load.explanation.contains("spike"));
    }
}
