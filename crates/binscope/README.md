# binscope

Read-only static inspection of executables and libraries (ELF, PE, Mach-O including universal binaries) over MCP. It answers "what is this file, what does it link against, and what changed since the last build?" It never executes, patches or unpacks anything.

## Install

```sh
cargo install --git https://github.com/Flotrpy/mcp binscope
```

```json
{ "mcpServers": { "binscope": { "command": "binscope-mcp", "env": { "BINSCOPE_ROOT": "/path/to/builds" } } } }
```

`BINSCOPE_ROOT` is optional. It confines the server to one directory (symlinks are resolved first) and rejects files over 512 MiB.

## Tools

| Tool | Purpose |
|---|---|
| `overview` | Format, architecture, entry point, stripped flag, counts, a whole-file entropy sparkline, the largest sections, notes and **suggested next calls** |
| `inspect` | One list at a time: `sections`, `libraries`, `imports`, `exports` or `strings`, with a filter and a `max_tokens` budget |
| `hexdump` | Up to 4 KiB of hex and ASCII at an offset |
| `diff` | Two builds compared: size, sections, entropy jumps, libraries, imports and exports, with highlights first |

## What makes it different
- **Progressive disclosure.** Four coarse tools instead of a hundred narrow ones, and `overview` tells the model which call to make next for *this* file.
- **Token budgets everywhere.** Lists are ranked, then trimmed whole-entry by whole-entry to fit `max_tokens`, and the result says how many entries were omitted.
- **Ranked strings.** URLs, paths, version banners, messages and format strings come before identifier noise, across both ASCII and UTF-16LE.
- **Release diffing.** It surfaces new libraries, removed exports (breaking changes), sections that suddenly became near-random, and new writable + executable sections.
