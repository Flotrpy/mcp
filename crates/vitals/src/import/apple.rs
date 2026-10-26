//! Apple Health `export.xml` (from Health → profile → Export All Health Data).
//!
//! The file is often hundreds of megabytes, so it is streamed record by
//! record rather than parsed into a tree.

use std::io::BufRead;

use chrono::DateTime;
use quick_xml::{Reader, events::Event};

use crate::{
    import::{ImportError, Parsed},
    metric::{self, Metric},
    store::Sample,
};

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

/// Sleep categories that count as asleep (in-bed and awake do not).
fn is_asleep(value: &str) -> bool {
    value.starts_with("HKCategoryValueSleepAnalysisAsleep")
}

/// Parse a HealthKit timestamp into (unix seconds, local day).
pub fn parse_date(s: &str) -> Option<(i64, String)> {
    let dt = DateTime::parse_from_str(s.trim(), DATE_FORMAT).ok()?;
    Some((dt.timestamp(), dt.format("%Y-%m-%d").to_string()))
}

pub fn parse<R: BufRead>(input: R) -> Result<Parsed, ImportError> {
    let mut reader = Reader::from_reader(input);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut parsed = Parsed::default();

    loop {
        let event = reader
            .read_event_into(&mut buf)
            .map_err(|e| ImportError::Malformed(format!("XML error at byte {}: {e}", reader.buffer_position())))?;
        match event {
            Event::Eof => break,
            Event::Start(ref e) | Event::Empty(ref e) if e.name().as_ref() == b"Record" => {
                parsed.read += 1;
                let mut kind = String::new();
                let mut source = String::new();
                let mut device = None;
                let mut unit = String::new();
                let mut start = String::new();
                let mut end = String::new();
                let mut value = String::new();
                for attr in e.attributes().flatten() {
                    let v = attr.unescape_value().map(|c| c.into_owned()).unwrap_or_default();
                    match attr.key.as_ref() {
                        b"type" => kind = v,
                        b"sourceName" => source = v,
                        b"device" => device = Some(v),
                        b"unit" => unit = v,
                        b"startDate" => start = v,
                        b"endDate" => end = v,
                        b"value" => value = v,
                        _ => {}
                    }
                }
                let Some(metric) = Metric::from_healthkit(&kind) else {
                    parsed.skipped += 1;
                    continue;
                };
                let (Some((start_ts, start_day)), Some((end_ts, end_day))) = (parse_date(&start), parse_date(&end)) else {
                    parsed.skipped += 1;
                    continue;
                };
                let (value, day) = if metric == Metric::Sleep {
                    if !is_asleep(&value) {
                        parsed.skipped += 1;
                        continue;
                    }
                    (((end_ts - start_ts) as f64 / 60.0).max(0.0), end_day)
                } else {
                    match value.parse::<f64>().ok().and_then(|v| metric::normalise(metric, v, &unit)) {
                        Some(v) => (v, start_day),
                        None => {
                            parsed.skipped += 1;
                            continue;
                        }
                    }
                };
                parsed.samples.push(Sample {
                    metric,
                    start: start_ts,
                    end: end_ts,
                    day,
                    value,
                    source: if source.is_empty() { "Apple Health".into() } else { source },
                    device: device.map(|d| short_device(&d)),
                });
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(parsed)
}

/// HealthKit device strings look like
/// `<<HKDevice: 0x…>, name:Apple Watch, manufacturer:Apple Inc., model:Watch, …>`.
/// Keep just the name.
fn short_device(raw: &str) -> String {
    raw.split(", ")
        .find_map(|part| part.strip_prefix("name:"))
        .unwrap_or(raw)
        .trim_end_matches('>')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<HealthData locale="en_US">
 <Record type="HKQuantityTypeIdentifierStepCount" sourceName="Jo's iPhone" unit="count" startDate="2026-10-01 08:00:00 -0700" endDate="2026-10-01 08:10:00 -0700" value="812"/>
 <Record type="HKQuantityTypeIdentifierBodyMass" sourceName="Scale" unit="lb" startDate="2026-10-01 07:00:00 -0700" endDate="2026-10-01 07:00:00 -0700" value="150"/>
 <Record type="HKCategoryTypeIdentifierSleepAnalysis" sourceName="Apple Watch" device="&lt;&lt;HKDevice: 0x1&gt;, name:Apple Watch, manufacturer:Apple Inc.&gt;" startDate="2026-09-30 23:30:00 -0700" endDate="2026-10-01 06:30:00 -0700" value="HKCategoryValueSleepAnalysisAsleepCore"/>
 <Record type="HKCategoryTypeIdentifierSleepAnalysis" sourceName="Apple Watch" startDate="2026-09-30 23:00:00 -0700" endDate="2026-09-30 23:30:00 -0700" value="HKCategoryValueSleepAnalysisInBed"/>
 <Record type="HKQuantityTypeIdentifierDietaryWater" sourceName="App" unit="mL" startDate="2026-10-01 09:00:00 -0700" endDate="2026-10-01 09:00:00 -0700" value="250"/>
</HealthData>"#;

    #[test]
    fn parses_quantities_sleep_and_skips_the_rest() {
        let p = parse(SAMPLE.as_bytes()).unwrap();
        assert_eq!(p.read, 5);
        assert_eq!(p.samples.len(), 3);
        assert_eq!(p.skipped, 2, "in-bed sleep and dietary water");

        let steps = &p.samples[0];
        assert_eq!((steps.metric, steps.value, steps.day.as_str()), (Metric::Steps, 812.0, "2026-10-01"));
        assert!((p.samples[1].value - 68.04).abs() < 0.01, "lb converted to kg");

        let sleep = &p.samples[2];
        assert_eq!(sleep.metric, Metric::Sleep);
        assert_eq!(sleep.value, 420.0, "7 hours asleep");
        assert_eq!(sleep.day, "2026-10-01", "attributed to the wake-up day");
        assert_eq!(sleep.device.as_deref(), Some("Apple Watch"));
    }

    #[test]
    fn local_day_uses_the_timestamps_own_offset() {
        // 23:30 in UTC-7 is already the next day in UTC; the local day wins.
        let (ts, day) = parse_date("2026-10-01 23:30:00 -0700").unwrap();
        assert_eq!(day, "2026-10-01");
        assert_eq!(ts, 1_790_922_600); // 2026-10-02T06:30:00Z
    }
}
