# Changelog

## 0.1.0 — 2026-10-26

First release of five MCP servers, all speaking the `2026-07-28` stateless protocol with fallback to the `2025-11-25` handshake.

- **joinery:**
  - `design_furniture`: a bookcase, cabinet or drawer, with parts, cut list, glue-up order and shelf sag checks.
  - `check_shelf_sag`.
  - `nest_parts`: guillotine nesting with kerf, grain and an SVG cut diagram.
  - `list_species`.
- **binscope:**
  - `overview`: entropy map and next-step hints.
  - `inspect`: sections, libraries, imports, exports or ranked strings, under a token budget.
  - `hexdump`.
  - `diff`: two builds compared.
  - `BINSCOPE_ROOT` sandboxing.
- **lighthouse:**
  - `audit_server`: dual-protocol probe, lint, token cost and an A–F grade.
  - `pin_server` and `verify_server` for `mcp.lock`.
  - `lint_tools`.
  - A CLI with CI exit codes.
- **vitals:**
  - `import_health_data`: Apple Health zip/xml and vitals CSV.
  - `data_overview`.
  - `daily_summary`: cross-device de-duplication.
  - `readiness`: explainable score.
  - `trend`.
  - `VITALS_REDACT`.
- **handset:**
  - `list_devices`.
  - `snapshot`: compact ref-tagged tree.
  - `act`: tap, type, scroll, key or launch, with a change-only diff.
  - `wait_for`.
  - Android via adb; iOS Simulator via idb/simctl.
- **mcp-kit:** shared stdio bootstrap, token budgets, CLI flags and the stdio test harness.
- **Site and dashboard:** a static site with recorded transcripts, a sag calculator, a config builder, a ⌘K palette and a lighthouse dashboard.
