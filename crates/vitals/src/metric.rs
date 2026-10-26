//! The metric vocabulary vitals stores, and how vendor types map onto it.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    /// Step count.
    Steps,
    /// Heart rate in beats per minute.
    HeartRate,
    /// Resting heart rate in bpm.
    RestingHeartRate,
    /// Heart-rate variability (SDNN) in milliseconds.
    Hrv,
    /// Minutes asleep.
    Sleep,
    /// Active energy in kilocalories.
    ActiveEnergy,
    /// Body mass in kilograms.
    Weight,
    /// VO₂ max in mL/(kg·min).
    Vo2Max,
}

/// How samples of a metric combine within a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aggregation {
    /// Add up (steps, energy, sleep minutes). Overlapping sources double-count.
    Sum,
    /// Average (heart rate, HRV).
    Mean,
    /// Lowest of the day (resting heart rate).
    Min,
    /// Most recent reading (weight, VO₂ max).
    Last,
}

pub const ALL: [Metric; 8] = [
    Metric::Steps,
    Metric::HeartRate,
    Metric::RestingHeartRate,
    Metric::Hrv,
    Metric::Sleep,
    Metric::ActiveEnergy,
    Metric::Weight,
    Metric::Vo2Max,
];

impl Metric {
    pub fn id(self) -> &'static str {
        match self {
            Metric::Steps => "steps",
            Metric::HeartRate => "heart_rate",
            Metric::RestingHeartRate => "resting_heart_rate",
            Metric::Hrv => "hrv",
            Metric::Sleep => "sleep",
            Metric::ActiveEnergy => "active_energy",
            Metric::Weight => "weight",
            Metric::Vo2Max => "vo2_max",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        ALL.into_iter().find(|m| m.id() == id.trim().to_ascii_lowercase().replace('-', "_"))
    }

    pub fn unit(self) -> &'static str {
        match self {
            Metric::Steps => "count",
            Metric::HeartRate | Metric::RestingHeartRate => "bpm",
            Metric::Hrv => "ms",
            Metric::Sleep => "min",
            Metric::ActiveEnergy => "kcal",
            Metric::Weight => "kg",
            Metric::Vo2Max => "mL/kg/min",
        }
    }

    pub fn aggregation(self) -> Aggregation {
        match self {
            Metric::Steps | Metric::ActiveEnergy | Metric::Sleep => Aggregation::Sum,
            Metric::HeartRate | Metric::Hrv => Aggregation::Mean,
            Metric::RestingHeartRate => Aggregation::Min,
            Metric::Weight | Metric::Vo2Max => Aggregation::Last,
        }
    }

    /// Map an Apple HealthKit quantity type identifier.
    pub fn from_healthkit(kind: &str) -> Option<Self> {
        Some(match kind {
            "HKQuantityTypeIdentifierStepCount" => Metric::Steps,
            "HKQuantityTypeIdentifierHeartRate" => Metric::HeartRate,
            "HKQuantityTypeIdentifierRestingHeartRate" => Metric::RestingHeartRate,
            "HKQuantityTypeIdentifierHeartRateVariabilitySDNN" => Metric::Hrv,
            "HKQuantityTypeIdentifierActiveEnergyBurned" => Metric::ActiveEnergy,
            "HKQuantityTypeIdentifierBodyMass" => Metric::Weight,
            "HKQuantityTypeIdentifierVO2Max" => Metric::Vo2Max,
            "HKCategoryTypeIdentifierSleepAnalysis" => Metric::Sleep,
            _ => return None,
        })
    }
}

/// Convert a value in `unit` to the metric's canonical unit, if known.
pub fn normalise(metric: Metric, value: f64, unit: &str) -> Option<f64> {
    let u = unit.trim();
    let v = match (metric, u) {
        (Metric::Weight, "lb") => value * 0.453_592_37,
        (Metric::Weight, "g") => value / 1000.0,
        (Metric::Weight, "kg") => value,
        (Metric::ActiveEnergy, "kJ") => value / 4.184,
        (Metric::ActiveEnergy, "kcal" | "Cal" | "cal") => value,
        (Metric::HeartRate | Metric::RestingHeartRate, "count/min" | "bpm") => value,
        (Metric::Hrv, "ms") => value,
        (Metric::Steps, "count" | "") => value,
        (Metric::Sleep, "min") => value,
        (Metric::Sleep, "h" | "hr") => value * 60.0,
        (Metric::Vo2Max, "mL/min·kg" | "mL/(kg·min)" | "ml/kg/min" | "mL/kg/min") => value,
        (m, u) if u == m.unit() => value,
        _ => return None,
    };
    Some(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip() {
        for m in ALL {
            assert_eq!(Metric::from_id(m.id()), Some(m));
        }
        assert_eq!(Metric::from_id("Heart-Rate"), Some(Metric::HeartRate));
    }

    #[test]
    fn normalises_imperial_and_kilojoules() {
        assert!((normalise(Metric::Weight, 150.0, "lb").unwrap() - 68.038_855_5).abs() < 1e-6);
        assert!((normalise(Metric::ActiveEnergy, 418.4, "kJ").unwrap() - 100.0).abs() < 1e-9);
        assert_eq!(normalise(Metric::Steps, 10.0, "furlongs"), None);
    }

    #[test]
    fn maps_healthkit_types() {
        assert_eq!(Metric::from_healthkit("HKQuantityTypeIdentifierStepCount"), Some(Metric::Steps));
        assert_eq!(Metric::from_healthkit("HKQuantityTypeIdentifierDietaryWater"), None);
    }
}
