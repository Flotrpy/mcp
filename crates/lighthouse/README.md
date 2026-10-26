# lighthouse

Audits MCP servers the way Lighthouse audits web pages. Point it at any stdio server and it reports:

- **Migration readiness:** does the server answer the `2026-07-28` stateless protocol (`server/discover`, per-request `_meta`), the `2025-11-25` initialize handshake, or both? Each generation runs in a fresh process.
- **Agent-usability lint:** missing or vague descriptions, undocumented parameters, free-text inputs with no shape, `required` fields that don't exist, duplicate names, missing `readOnlyHint`, and logging to stdout.
- **Token cost:** what each tool definition costs the model on every turn, heaviest first.
- **An A–F grade** listing every deduction.
- **`mcp.lock` drift pinning:** commit the lockfile, and `verify` fails CI when a tool's description, schema or annotations change. Changed descriptions are shown old and new.

## Install

```sh
cargo install --git https://github.com/Flotrpy/mcp lighthouse
```

## As a CLI (CI-friendly)

```sh
lighthouse-mcp audit --min-grade B -- npx -y @scope/my-server   # exit 1 below B
lighthouse-mcp pin    -- ./target/release/my-server              # writes mcp.lock
lighthouse-mcp verify -- ./target/release/my-server              # exit 1 on drift
```

```text
joinery 0.1.0  grade A (96/100)  migration: ready
4 tools, ~1461 tokens per turn
  -  4  2 lint warning(s)
  Warning  param-undocumented  check_shelf_sag: Parameters without a description: thickness.
```

*That's real output: lighthouse caught undocumented parameters in this repo's own servers, and they now all score 100.*

## As an MCP server

```json
{ "mcpServers": { "lighthouse": { "command": "lighthouse-mcp" } } }
```

| Tool | Purpose |
|---|---|
| `audit_server` | Full audit and grade of a command you name |
| `pin_server` | Write `mcp.lock` for that server |
| `verify_server` | Compare the live server against `mcp.lock` |
| `lint_tools` | Lint pasted `tools/list` output without launching anything |

`audit_server`, `pin_server` and `verify_server` start the command you pass them, so only point them at servers you would run anyway.
