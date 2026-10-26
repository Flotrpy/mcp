//! `lighthouse-mcp`: an MCP server, and a CLI for CI pipelines.
//!
//! ```text
//! lighthouse-mcp                                   # serve MCP over stdio
//! lighthouse-mcp audit [--json] [--min-grade B] -- <command> [args…]
//! lighthouse-mcp pin    [--lock mcp.lock]       -- <command> [args…]
//! lighthouse-mcp verify [--lock mcp.lock]       -- <command> [args…]
//! ```
//!
//! `audit` exits 1 when the grade is below `--min-grade`; `verify` exits 1 on
//! any drift, so both slot straight into CI.

use std::process::ExitCode;

use lighthouse::{audit, lock, probe::Target};

const USAGE: &str = "usage:
  lighthouse-mcp                                  serve MCP over stdio
  lighthouse-mcp audit [--json] [--min-grade X] -- <command> [args…]
  lighthouse-mcp pin [--lock PATH] -- <command> [args…]
  lighthouse-mcp verify [--lock PATH] -- <command> [args…]";

struct Cli {
    sub: String,
    json: bool,
    min_grade: Option<char>,
    lock: String,
    target: Target,
}

fn parse(args: &[String]) -> Result<Cli, String> {
    let sub = args.first().cloned().ok_or(USAGE)?;
    let split = args.iter().position(|a| a == "--").ok_or("missing `--` before the server command")?;
    let (flags, rest) = (&args[1..split], &args[split + 1..]);
    let command = rest.first().ok_or("missing server command after `--`")?.clone();
    let mut cli = Cli {
        sub,
        json: false,
        min_grade: None,
        lock: lock::DEFAULT_PATH.into(),
        target: Target::new(command, rest[1..].to_vec()),
    };
    let mut it = flags.iter();
    while let Some(f) = it.next() {
        match f.as_str() {
            "--json" => cli.json = true,
            "--min-grade" => {
                let g = it.next().and_then(|g| g.chars().next()).ok_or("--min-grade needs a letter")?;
                cli.min_grade = Some(g.to_ascii_uppercase());
            }
            "--lock" => cli.lock = it.next().ok_or("--lock needs a path")?.clone(),
            other => return Err(format!("unknown flag {other}\n{USAGE}")),
        }
    }
    Ok(cli)
}

async fn run_cli(cli: Cli) -> Result<ExitCode, String> {
    match cli.sub.as_str() {
        "audit" => {
            let report = audit::run(&cli.target).await;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?);
            } else {
                println!(
                    "{} {}  grade {} ({}/100)  migration: {}",
                    report.server_name.as_deref().unwrap_or("?"),
                    report.server_version.as_deref().unwrap_or(""),
                    report.score.grade,
                    report.score.points,
                    report.migration
                );
                println!("{} tools, ~{} tokens per turn", report.tool_count, report.cost.total_tokens);
                for (why, pts) in &report.score.deductions {
                    println!("  -{pts:>3}  {why}");
                }
                for f in &report.findings {
                    println!("  {:?}  {}  {}: {}", f.severity, f.rule, f.tool, f.message);
                }
            }
            // Grades are ordered A < B < … < F as characters.
            let below = cli.min_grade.is_some_and(|min| report.score.grade > min);
            Ok(if below { ExitCode::FAILURE } else { ExitCode::SUCCESS })
        }
        "pin" | "verify" => {
            let s = lighthouse::probe::stateless(&cli.target).await;
            let g = if s.supported { s } else { lighthouse::probe::legacy(&cli.target).await };
            if !g.supported {
                return Err(g.error.unwrap_or_else(|| "server did not answer".into()));
            }
            let current = lock::Lockfile::from_tools(g.server_name, g.server_version, &g.tools);
            if cli.sub == "pin" {
                std::fs::write(&cli.lock, current.to_pretty_json()).map_err(|e| format!("{}: {e}", cli.lock))?;
                println!("pinned {} tool(s) to {}", current.tools.len(), cli.lock);
                return Ok(ExitCode::SUCCESS);
            }
            let text = std::fs::read_to_string(&cli.lock).map_err(|e| format!("{}: {e}", cli.lock))?;
            let pinned: lock::Lockfile = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", cli.lock))?;
            let drift = lock::compare(&pinned, &current);
            println!("{}", drift.summary());
            for c in &drift.changed {
                if let (Some(old), Some(new)) = (&c.old_description, &c.new_description) {
                    println!("  {}:\n    - {old}\n    + {new}", c.tool);
                }
            }
            Ok(if drift.is_clean() { ExitCode::SUCCESS } else { ExitCode::FAILURE })
        }
        other => Err(format!("unknown command {other}\n{USAGE}")),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<ExitCode> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("lighthouse-mcp {}\n\n{USAGE}", env!("CARGO_PKG_VERSION"));
        return Ok(ExitCode::SUCCESS);
    }
    if args.is_empty() {
        mcp_kit::run_stdio(lighthouse::server::Lighthouse::new()).await?;
        return Ok(ExitCode::SUCCESS);
    }
    match parse(&args) {
        Ok(cli) => match run_cli(cli).await {
            Ok(code) => Ok(code),
            Err(e) => {
                eprintln!("lighthouse: {e}");
                Ok(ExitCode::from(2))
            }
        },
        Err(e) => {
            eprintln!("lighthouse: {e}");
            Ok(ExitCode::from(2))
        }
    }
}
