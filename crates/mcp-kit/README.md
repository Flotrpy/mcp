# mcp-kit

Shared plumbing for the servers in this workspace, so all five behave the same from an agent's point of view.

| Module | Purpose |
|---|---|
| `run_stdio`, `server_config` | Serve a `rmcp` handler over stdio with logs on stderr; report the crate's own name, version, title and instructions |
| `budget` | `estimate_tokens` and `fit`: trim a ranked list to a token budget whole entries at a time, reporting how many were omitted |
| `result` | `structured` (text summary + structured content), `tool_error` (recoverable, with a fix in the message), `invalid_params` |
| `cli` | Uniform `--help` / `--version` |
| `testing` | Drive a compiled server binary over stdio with the stateless (`2026-07-28`) or legacy (`2025-11-25`) protocol |
