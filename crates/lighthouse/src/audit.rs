//! Combine probe, lint and cost into one graded audit.

use serde::Serialize;
use serde_json::Value;

use crate::{
    cost::{self, CostReport},
    lint::{self, Finding, Severity},
    probe::{self, Generation, Target},
};

#[derive(Debug, Clone, Serialize)]
pub struct Score {
    pub points: u32,
    pub grade: char,
    /// Why points were lost, largest first.
    pub deductions: Vec<(String, u32)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Audit {
    pub server_name: Option<String>,
    pub server_version: Option<String>,
    pub stateless: Generation,
    pub legacy: Generation,
    /// `ready`, `legacy-only`, `stateless-only` or `unreachable`.
    pub migration: &'static str,
    pub tool_count: usize,
    pub findings: Vec<Finding>,
    pub cost: CostReport,
    pub score: Score,
}

pub fn letter(points: u32) -> char {
    match points {
        90.. => 'A',
        80..=89 => 'B',
        70..=79 => 'C',
        60..=69 => 'D',
        _ => 'F',
    }
}

/// Score an audit out of 100. Each category is capped so one noisy rule
/// cannot sink an otherwise good server.
pub fn score(stateless: &Generation, legacy: &Generation, findings: &[Finding], cost: &CostReport) -> Score {
    let mut d: Vec<(String, u32)> = Vec::new();
    if !stateless.supported && !legacy.supported {
        return Score {
            points: 0,
            grade: 'F',
            deductions: vec![("Server did not answer on either protocol generation".into(), 100)],
        };
    }
    if !stateless.supported {
        d.push(("No 2026-07-28 stateless support (server/discover)".into(), 15));
    }
    if !legacy.supported {
        d.push(("No 2025-11-25 initialize handshake for older clients".into(), 10));
    }
    let count = |s: Severity| findings.iter().filter(|f| f.severity == s).count() as u32;
    let errors = count(Severity::Error);
    let warnings = count(Severity::Warning);
    let infos = count(Severity::Info);
    if errors > 0 {
        d.push((format!("{errors} lint error(s)"), (errors * 6).min(30)));
    }
    if warnings > 0 {
        d.push((format!("{warnings} lint warning(s)"), (warnings * 2).min(20)));
    }
    if infos > 0 {
        d.push((format!("{infos} lint note(s)"), (infos / 2).min(5)));
    }
    if cost.heavy {
        d.push((format!("Tool list costs {} tokens per turn", cost.total_tokens), 10));
    } else if cost.total_tokens > cost::HEAVY_TOTAL / 2 {
        d.push((format!("Tool list costs {} tokens per turn", cost.total_tokens), 5));
    }
    if let Some(t) = cost.tools.first().filter(|t| t.tokens > cost::HEAVY_TOOL) {
        d.push((format!("{} alone costs {} tokens", t.tool, t.tokens), 3));
    }
    let stray = !stateless.stray_output.is_empty() || !legacy.stray_output.is_empty();
    if stray {
        d.push(("Non-JSON output on stdout (logs belong on stderr)".into(), 10));
    }
    d.retain(|(_, p)| *p > 0);
    d.sort_by(|a, b| b.1.cmp(&a.1));
    let lost: u32 = d.iter().map(|(_, p)| p).sum();
    let points = 100u32.saturating_sub(lost);
    Score { points, grade: letter(points), deductions: d }
}

/// Audit a tool list that was obtained some other way (pasted JSON).
pub fn offline(tools: &[Value]) -> (Vec<Finding>, CostReport) {
    (lint::lint_all(tools), cost::measure(tools))
}

/// Launch `target` and run the full audit.
pub async fn run(target: &Target) -> Audit {
    let (stateless, legacy) = tokio::join!(probe::stateless(target), probe::legacy(target));
    // Prefer the stateless tool list; fall back to legacy.
    let tools = if stateless.supported { &stateless.tools } else { &legacy.tools };
    let findings = lint::lint_all(tools);
    let cost = cost::measure(tools);
    let migration = match (stateless.supported, legacy.supported) {
        (true, true) => "ready",
        (false, true) => "legacy-only",
        (true, false) => "stateless-only",
        (false, false) => "unreachable",
    };
    let score = score(&stateless, &legacy, &findings, &cost);
    Audit {
        server_name: stateless.server_name.clone().or_else(|| legacy.server_name.clone()),
        server_version: stateless.server_version.clone().or_else(|| legacy.server_version.clone()),
        tool_count: tools.len(),
        migration,
        findings,
        cost,
        score,
        stateless,
        legacy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_ok() -> Generation {
        Generation { supported: true, ..Default::default() }
    }

    #[test]
    fn perfect_server_gets_an_a() {
        let s = score(&gen_ok(), &gen_ok(), &[], &cost::measure(&[]));
        assert_eq!((s.points, s.grade), (100, 'A'));
    }

    #[test]
    fn legacy_only_with_errors_loses_points_in_order() {
        let f = |sev| Finding { severity: sev, rule: "r", tool: "t".into(), message: "m".into() };
        let findings = vec![f(Severity::Error), f(Severity::Error), f(Severity::Warning)];
        let s = score(&Generation::default(), &gen_ok(), &findings, &cost::measure(&[]));
        assert_eq!(s.points, 100 - 15 - 12 - 2);
        assert_eq!(s.grade, 'C');
        assert!(s.deductions[0].0.contains("stateless"));
    }

    #[test]
    fn unreachable_is_zero() {
        let s = score(&Generation::default(), &Generation::default(), &[], &cost::measure(&[]));
        assert_eq!(s.grade, 'F');
        assert_eq!(s.points, 0);
    }

    #[test]
    fn stray_stdout_is_penalised() {
        let mut noisy = gen_ok();
        noisy.stray_output.push("Server started on stdio".into());
        let s = score(&noisy, &gen_ok(), &[], &cost::measure(&[]));
        assert_eq!(s.points, 90);
    }
}
