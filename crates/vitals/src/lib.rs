//! A local health-data warehouse for MCP.
//!
//! Exports from Apple Health (and anything converted to the vitals CSV
//! format) are loaded into a SQLite file on your machine. Tools answer
//! questions over that file: daily summaries with cross-device
//! de-duplication, trends, and an explainable readiness score. Nothing is
//! uploaded anywhere; with redaction on, timestamps are coarsened and device
//! names hidden before results reach the model.

pub mod metric;
