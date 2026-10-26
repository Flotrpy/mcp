//! SQLite storage.
//!
//! Each sample keeps its UTC start/end plus the *local* calendar day it
//! belongs to, computed at import from the timestamp's own offset. Sleep is
//! attributed to the day you woke up. A uniqueness constraint makes
//! re-importing the same export a no-op.

use std::path::Path;

use rusqlite::{Connection, params};
use serde::Serialize;

use crate::metric::Metric;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sample {
    pub metric: Metric,
    /// Unix seconds, UTC.
    pub start: i64,
    pub end: i64,
    /// Local calendar day, `YYYY-MM-DD`.
    pub day: String,
    /// Value in the metric's canonical unit.
    pub value: f64,
    /// App or vendor that recorded it, e.g. "Apple Watch" or "Oura".
    pub source: String,
    pub device: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricInfo {
    pub metric: Metric,
    pub unit: &'static str,
    pub samples: u64,
    pub first_day: String,
    pub last_day: String,
    /// `(source, samples)`, most samples first.
    pub sources: Vec<(String, u64)>,
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Sql(#[from] rusqlite::Error),
}

pub struct Store {
    conn: Connection,
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS samples (
    metric  TEXT    NOT NULL,
    start   INTEGER NOT NULL,
    end     INTEGER NOT NULL,
    day     TEXT    NOT NULL,
    value   REAL    NOT NULL,
    source  TEXT    NOT NULL,
    device  TEXT,
    UNIQUE (metric, start, end, value, source)
);
CREATE INDEX IF NOT EXISTS samples_metric_day ON samples (metric, day);
CREATE TABLE IF NOT EXISTS imports (
    id          INTEGER PRIMARY KEY,
    path        TEXT    NOT NULL,
    format      TEXT    NOT NULL,
    imported_at INTEGER NOT NULL,
    read        INTEGER NOT NULL,
    inserted    INTEGER NOT NULL
);
";

impl Store {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self, StoreError> {
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// Insert samples in one transaction; returns how many were new.
    pub fn insert(&mut self, samples: &[Sample]) -> Result<usize, StoreError> {
        let tx = self.conn.transaction()?;
        let mut inserted = 0;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR IGNORE INTO samples (metric, start, end, day, value, source, device)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )?;
            for s in samples {
                inserted += stmt.execute(params![s.metric.id(), s.start, s.end, s.day, s.value, s.source, s.device])?;
            }
        }
        tx.commit()?;
        Ok(inserted)
    }

    pub fn record_import(&self, path: &str, format: &str, read: usize, inserted: usize) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO imports (path, format, imported_at, read, inserted) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![path, format, chrono::Utc::now().timestamp(), read as i64, inserted as i64],
        )?;
        Ok(())
    }

    /// Samples of `metric` whose local day is within `[from, to]` (inclusive).
    pub fn samples(&self, metric: Metric, from: &str, to: &str) -> Result<Vec<Sample>, StoreError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT start, end, day, value, source, device FROM samples
             WHERE metric = ?1 AND day BETWEEN ?2 AND ?3 ORDER BY start",
        )?;
        let rows = stmt.query_map(params![metric.id(), from, to], |r| {
            Ok(Sample {
                metric,
                start: r.get(0)?,
                end: r.get(1)?,
                day: r.get(2)?,
                value: r.get(3)?,
                source: r.get(4)?,
                device: r.get(5)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// What is in the warehouse, per metric.
    pub fn overview(&self) -> Result<Vec<MetricInfo>, StoreError> {
        let mut out = Vec::new();
        for metric in crate::metric::ALL {
            let (samples, first, last): (u64, Option<String>, Option<String>) = self.conn.query_row(
                "SELECT COUNT(*), MIN(day), MAX(day) FROM samples WHERE metric = ?1",
                params![metric.id()],
                |r| Ok((r.get::<_, i64>(0)? as u64, r.get(1)?, r.get(2)?)),
            )?;
            if samples == 0 {
                continue;
            }
            let mut stmt = self.conn.prepare_cached(
                "SELECT source, COUNT(*) AS n FROM samples WHERE metric = ?1 GROUP BY source ORDER BY n DESC",
            )?;
            let sources = stmt
                .query_map(params![metric.id()], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u64)))?
                .collect::<Result<_, _>>()?;
            out.push(MetricInfo {
                metric,
                unit: metric.unit(),
                samples,
                first_day: first.unwrap_or_default(),
                last_day: last.unwrap_or_default(),
                sources,
            });
        }
        Ok(out)
    }

    /// The most recent day with any data, if any.
    pub fn last_day(&self) -> Result<Option<String>, StoreError> {
        Ok(self.conn.query_row("SELECT MAX(day) FROM samples", [], |r| r.get(0))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn sample(metric: Metric, day: &str, start: i64, value: f64, source: &str) -> Sample {
        Sample { metric, start, end: start + 60, day: day.into(), value, source: source.into(), device: None }
    }

    #[test]
    fn reimporting_the_same_samples_is_a_no_op() {
        let mut s = Store::open_in_memory().unwrap();
        let batch = vec![
            sample(Metric::Steps, "2026-10-01", 1000, 500.0, "iPhone"),
            sample(Metric::Steps, "2026-10-01", 1000, 480.0, "Apple Watch"),
        ];
        assert_eq!(s.insert(&batch).unwrap(), 2);
        assert_eq!(s.insert(&batch).unwrap(), 0);
        assert_eq!(s.samples(Metric::Steps, "2026-10-01", "2026-10-01").unwrap().len(), 2);
    }

    #[test]
    fn overview_counts_sources_and_day_range() {
        let mut s = Store::open_in_memory().unwrap();
        s.insert(&[
            sample(Metric::Hrv, "2026-09-30", 1, 40.0, "Apple Watch"),
            sample(Metric::Hrv, "2026-10-02", 2, 42.0, "Apple Watch"),
            sample(Metric::Hrv, "2026-10-01", 3, 55.0, "Oura"),
        ])
        .unwrap();
        let o = s.overview().unwrap();
        assert_eq!(o.len(), 1);
        assert_eq!(o[0].samples, 3);
        assert_eq!(o[0].first_day, "2026-09-30");
        assert_eq!(o[0].last_day, "2026-10-02");
        assert_eq!(o[0].sources[0], ("Apple Watch".to_string(), 2));
        assert_eq!(s.last_day().unwrap().as_deref(), Some("2026-10-02"));
    }
}
