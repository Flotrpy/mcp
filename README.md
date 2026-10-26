# Five MCP servers in Rust

**Five servers. Each does one hard thing well.** Five open-source [Model Context Protocol](https://modelcontextprotocol.io) servers that give your agent a woodshop calculator, a binary inspector, a protocol auditor, a private health warehouse and a pair of hands for your phone. Each is a single static binary with no accounts or daemons, speaks both protocol generations, and is graded A by its own auditor on every push.

<p>
  <a href="https://github.com/Flotrpy/mcp/actions/workflows/ci.yml"><img alt="checks" src="https://img.shields.io/github/actions/workflow/status/Flotrpy/mcp/ci.yml?branch=main&label=checks&logo=github"></a>
  <img alt="version: 0.1.0" src="https://img.shields.io/badge/version-0.1.0-5d4ed8">
  <img alt="rust: 1.89+" src="https://img.shields.io/badge/rust-1.89%2B-b7410e?logo=rust&logoColor=white">
  <img alt="node: 22+" src="https://img.shields.io/badge/node-22%2B-2e7d32?logo=nodedotjs&logoColor=white">
  <a href="LICENSE"><img alt="license: MIT" src="https://img.shields.io/badge/license-MIT-7b2fbe"></a>
</p>

<p>
  <img alt="servers: 5 · 21 tools" src="https://img.shields.io/badge/servers-5%20%C2%B7%2021%20tools-0a7e72">
  <img alt="SDK: rmcp 3.7" src="https://img.shields.io/badge/SDK-rmcp%203.7-3b6fd8?logo=rust&logoColor=white">
  <img alt="frontend: HTML + CSS + ES modules" src="https://img.shields.io/badge/frontend-HTML%20%2B%20CSS%20%2B%20ES%20modules-e44d26?logo=html5&logoColor=white">
  <img alt="protocol: MCP 2026-07-28 + 2025-11-25" src="https://img.shields.io/badge/protocol-MCP%202026--07--28%20%2B%202025--11--25-c62828">
  <img alt="transport: stdio" src="https://img.shields.io/badge/transport-stdio-2f6fdb">
</p>

<p>
  <img alt="tests: 130 Rust · 10 site · 5 audits" src="https://img.shields.io/badge/tests-130%20Rust%20%C2%B7%2010%20site%20%C2%B7%205%20audits-1f8a3b">
  <img alt="lint: rustfmt + clippy -D warnings" src="https://img.shields.io/badge/lint-rustfmt%20%2B%20clippy%20--D%20warnings-9bbd3a">
  <a href="https://flotrpy.github.io/mcp/dashboard.html"><img alt="lighthouse: grade A" src="https://img.shields.io/badge/lighthouse-grade%20A-f2a900"></a>
  <a href="https://flotrpy.github.io/mcp/"><img alt="deploy: GitHub Pages" src="https://img.shields.io/badge/deploy-GitHub%20Pages-24b47e?logo=githubpages&logoColor=white"></a>
  <img alt="tool surface: pinned in mcp.lock" src="https://img.shields.io/badge/tool%20surface-pinned%20in%20mcp.lock-6a2bd9">
</p>

<a href="https://flotrpy.github.io/mcp/">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/images/site-dark.png">
    <img alt="The project website: Five servers. Each does one hard thing well." src="docs/images/site-light.png">
  </picture>
</a>

| Server | What it does | Tools |
|---|---|---|
| [**joinery**](crates/joinery) | Woodworking: parametric furniture, cut lists, kerf- and grain-aware sheet nesting, shelf-sag physics | 4 |
| [**binscope**](crates/binscope) | Read-only inspection of ELF/PE/Mach-O files: sections, entropy, imports, ranked strings, a diff of two builds | 4 |
| [**lighthouse**](crates/lighthouse) | Audits any MCP server: both protocols, tool lint, token cost, an A–F grade, `mcp.lock` drift pinning | 4 |
| [**vitals**](crates/vitals) | A local health warehouse: Apple Health and CSV import, phone/watch de-duplication, explainable readiness | 5 |
| [**handset**](crates/handset) | Drives Android and the iOS Simulator with compact ref-tagged screens and change-only replies | 4 |

## Install

```sh
cargo install --git https://github.com/Flotrpy/mcp joinery binscope lighthouse vitals handset
```

Prebuilt binaries for Linux, macOS and Windows are attached to each [release](https://github.com/Flotrpy/mcp/releases). Then add the servers you want to your client:

```json
{
  "mcpServers": {
    "joinery": { "command": "joinery-mcp" },
    "lighthouse": { "command": "lighthouse-mcp" }
  }
}
```

The [website](site/) has a config builder for Claude Code, Claude Desktop, Cursor and Codex.

## Quality gates
- **Tests:** `cargo test --workspace` runs 130 unit and end-to-end tests. The end-to-end tests drive each compiled binary over stdio, on both protocol generations.
- **CI** runs fmt, `clippy -D warnings`, the tests, and the site's JS tests. It then has **lighthouse audit every server** and fails below an A, and **verify each against its committed `locks/*.lock`**, which fails on any silent change to a tool's description or schema.
- **Dashboard:** `site/dashboard.html` shows the latest audit, with grades, token cost per tool and lockfile status.

## Layout

```text
crates/
  mcp-kit/      shared bootstrap, token budgets, result helpers, CLI, test harness
  joinery/  binscope/  lighthouse/  vitals/  handset/
locks/          mcp.lock for each server (checked in CI)
site/           static website and dashboard (no build step)
docs/           design decisions and UI research
research/       the ideas research, its data and the scripts that collected it
examples/       a 35-day multi-device vitals dataset
```

## Docs
- [Design decisions](docs/DESIGN.md): protocol layer, transport, stack, and what makes each server different.
- [UI research](docs/UI-RESEARCH.md): what the Apple HIG, Apple's kits, Emil Kowalski and Dribbble each recommend, and which won.
- [Research](research/mcp-ideas-master.md): the 34 ideas these were chosen from, with the Hacker News and web sweeps behind them.

## License

MIT
