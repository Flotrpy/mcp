//! The vitals CSV format: one row per sample, for anything that isn't an
//! Apple Health export (Garmin, Oura, Whoop, a spreadsheet…).
//!
//! ```csv
//! metric,start,end,value,unit,source
//! steps,2026-10-01T08:00:00-07:00,2026-10-01T08:10:00-07:00,812,count,Garmin
//! hrv,2026-10-01T06:00:00+01:00,,48,ms,Oura
//! ```
//!
//! `end` may be empty (instantaneous readings) and `unit` may be empty (the
//! metric's canonical unit is assumed). Timestamps are RFC 3339 with an offset,
//! or a bare `YYYY-MM-DD` for daily totals.

use std::io::Read;

use chrono::{DateTime, NaiveDate};
use serde::Deserialize;

use crate::{
    import::{ImportError, Parsed},
    metric::{self, Metric},
    store::Sample,
};

#[derive(Debug, Deserialize)]
struct Row {
    metric: String,
    start: String,
    #[serde(default)]
    end: String,
    value: String,
    #[serde(default)]
    unit: String,
    #[serde(default)]
    source: String,
}

fn parse_time(s: &str) -> Option<(i64, String)> {
    let s = s.trim();
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some((dt.timestamp(), dt.format("%Y-%m-%d").to_string()));
    }
    let d = NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()?;
    let ts = d.and_hms_opt(12, 0, 0)?.and_utc().timestamp();
    Some((ts, d.format("%Y-%m-%d").to_string()))
}

pub fn parse<R: Read>(input: R, default_source: &str) -> Result<Parsed, ImportError> {
    let mut reader = csv::ReaderBuilder::new().trim(csv::Trim::All).flexible(true).from_reader(input);
    let headers = reader.headers().map_err(|e| ImportError::Malformed(e.to_string()))?.clone();
    for required in ["metric", "start", "value"] {
        if !headers.iter().any(|h| h == required) {
            return Err(ImportError::Malformed(format!(
                "missing `{required}` column; expected metric,start,end,value,unit,source"
            )));
        }
    }
    let mut parsed = Parsed::default();
    for record in reader.deserialize::<Row>() {
        parsed.read += 1;
        let Ok(row) = record else {
            parsed.skipped += 1;
            continue;
        };
        let Some(metric) = Metric::from_id(&row.metric) else {
            parsed.skipped += 1;
            continue;
        };
        let Some((start, start_day)) = parse_time(&row.start) else {
            parsed.skipped += 1;
            continue;
        };
        let (end, end_day) = if row.end.is_empty() { (start, start_day.clone()) } else {
            match parse_time(&row.end) {
                Some(t) => t,
                None => {
                    parsed.skipped += 1;
                    continue;
                }
            }
        };
        let unit = if row.unit.is_empty() { metric.unit().to_string() } else { row.unit };
        let Some(value) = row.value.parse::<f64>().ok().and_then(|v| metric::normalise(metric, v, &unit)) else {
            parsed.skipped += 1;
            continue;
        };
        parsed.samples.push(Sample {
            metric,
            start,
            end,
            day: if metric == Metric::Sleep { end_day } else { start_day },
            value,
            source: if row.source.is_empty() { default_source.to_string() } else { row.source },
            device: None,
        });
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rfc3339_dates_and_daily_rows() {
        let data = "metric,start,end,value,unit,source\n\
                    steps,2026-10-01T08:00:00-07:00,2026-10-01T08:10:00-07:00,812,count,Garmin\n\
                    hrv,2026-10-01,,48,,Oura\n\
                    weight,2026-10-01,,150,lb,\n\
                    sleep,2026-09-30T23:00:00+01:00,2026-10-01T07:00:00+01:00,8,h,Oura\n\
                    nonsense,2026-10-01,,1,,x\n";
        let p = parse(data.as_bytes(), "import.csv").unwrap();
        assert_eq!(p.read, 5);
        assert_eq!(p.samples.len(), 4);
        assert_eq!(p.skipped, 1);
        assert_eq!(p.samples[1].source, "Oura");
        assert_eq!(p.samples[2].source, "import.csv", "default source when blank");
        assert_eq!(p.samples[3].value, 480.0);
        assert_eq!(p.samples[3].day, "2026-10-01");
    }

    #[test]
    fn missing_required_columns_is_a_clear_error() {
        let err = parse("when,how_much\n1,2\n".as_bytes(), "x").unwrap_err();
        assert!(err.to_string().contains("missing `metric` column"));
    }
}
