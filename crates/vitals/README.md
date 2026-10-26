# vitals

A local-first health-data warehouse for MCP. Import an Apple Health export (or any wearable data converted to a simple CSV) into a SQLite file on your machine, then ask questions about it. Nothing is uploaded.

## Install

```sh
cargo install --git https://github.com/Flotrpy/mcp vitals
```

```json
{ "mcpServers": { "vitals": { "command": "vitals-mcp", "env": { "VITALS_REDACT": "1" } } } }
```

| Variable | Meaning |
|---|---|
| `VITALS_DB` | SQLite file. The default is `~/.local/share/vitals/vitals.sqlite` |
| `VITALS_REDACT` | `1` replaces source and device names with `Source A`, `Source B`, … in every result |

## Tools

| Tool | Purpose |
|---|---|
| `import_health_data` | Load an iPhone `export.zip`, an `export.xml` or a vitals `.csv`. Idempotent |
| `data_overview` | Each metric's unit, sample count, day range and sources |
| `daily_summary` | One value per day per metric, de-duplicated across devices |
| `readiness` | A 0–100 score with every factor explained |
| `trend` | A sparkline, min/mean/max, and the last 7 days vs the 7 before |

## What makes it different
- **Cross-device de-duplication.** Carry a phone and wear a watch, and both count your steps. vitals keeps the best source per clock hour (watch > ring > phone, or your own `source_priority`) and reports how much double-counting it removed. Two sleep trackers resolve to one night.
- **An explainable readiness score.** It compares HRV and resting heart rate against *your* 7-day baseline, sleep against your need, and acute:chronic training load against the 0.8–1.3 sweet spot. Each factor shows its value, baseline, weight and points contributed. Missing data is re-weighted, never invented.
- **Privacy by default.** Data lives in a local file. Redaction hides device names, and results report days, never times.
- **Local calendar days.** Every sample is bucketed by the timezone it was recorded in, so travel doesn't scramble your days.

## CSV format (Garmin, Oura, Whoop, spreadsheets)

```csv
metric,start,end,value,unit,source
steps,2026-10-01T08:00:00-07:00,2026-10-01T08:10:00-07:00,812,count,Garmin
hrv,2026-10-01,,48,ms,Oura
sleep,2026-09-30T23:00:00+01:00,2026-10-01T07:00:00+01:00,8,h,Oura
```

The metrics are `steps`, `heart_rate`, `resting_heart_rate`, `hrv`, `sleep`, `active_energy`, `weight` and `vo2_max`. Units convert automatically (lb→kg, kJ→kcal, h→min).

*Readiness is a wellbeing heuristic, not medical advice.*
