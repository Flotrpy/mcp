# Contributing

## Before you push

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
(cd site && npm test)
```

## Changing a tool
Tool names, descriptions and schemas are pinned in `locks/<server>.lock`, so a change fails CI until you re-pin on purpose:

```sh
cargo build --release --workspace
./target/release/lighthouse-mcp verify --lock locks/joinery.lock -- ./target/release/joinery-mcp  # see what changed
./target/release/lighthouse-mcp pin    --lock locks/joinery.lock -- ./target/release/joinery-mcp  # accept it
```

Commit the updated lockfile with the change, so reviewers see the new description in the diff.

Every server must also keep an **A** from `lighthouse audit`. Document every parameter, say what a tool returns and when to use it, and set `readOnlyHint`.

## Conventions
- **Results:** tools return `mcp_kit::result::structured`, a short text summary plus structured content. Recoverable problems are `tool_error`s whose message says how to fix the call.
- **Token budgets:** lists that can grow take `max_tokens` and are trimmed with `mcp_kit::budget::fit`, whole entries at a time.
- **End-to-end tests** live in `crates/<server>/tests/` and drive the compiled binary with `mcp_kit::testing`.

## The website
`site/` is static: no build step and no dependencies. After changing a server, refresh the recorded transcripts and dashboard data:

```sh
cargo build --release --workspace
python3 site/tools/record.py
```
