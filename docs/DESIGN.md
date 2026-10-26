# Design decisions

This records the options weighed before any code was written, and why each choice won. Research inputs:
- `research/mcp-ideas-master.md`, the idea ranking.
- The Vercel Web Interface Guidelines.
- Apple's Human Interface Guidelines (Motion).
- Rauno Freiberg, *Invisible Details of Interaction Design*.
- Emil Kowalski, *Great Animations*.
- The MCP `2026-07-28` changelog.

## 1. Protocol layer

| Option | Pros | Cons |
|---|---|---|
| Hand-rolled JSON-RPC over stdio | No dependencies, full control | Must track every spec revision by hand; easy to get the stateless `2026-07-28` details wrong |
| **`rmcp` 3.7 (official Rust SDK)** | Speaks `2026-07-28` (`server/discover`, per-request `_meta`, `ttlMs`/`cacheScope`) *and* falls back to the `2025-11-25` initialize handshake; `#[tool]` macros generate JSON Schema from Rust types | Macro-heavy API; a larger dependency tree |
| TypeScript SDK | The largest ecosystem | Users need Node; no single static binary |

**Decision: `rmcp`.** A probe server confirmed that one binary answers both a stateless `server/discover` → `tools/call` exchange and a legacy `initialize` → `tools/list` exchange. Users get one static binary per server, installed with `cargo install`, that works with old and new clients.

## 2. Transport

stdio is the default for all five servers. Every desktop MCP client (Claude Code, Claude Desktop, Codex, Cursor) launches stdio servers from a JSON config, so there is no port, no auth and no daemon to babysit. That is the easiest path for users.

## 3. Repository shape

A single Cargo workspace:
- `crates/mcp-kit` holds the shared code: the server bootstrap, result helpers and the token estimator.
- There is one crate per server.

Shared code keeps the five servers consistent: the same `--help`, the same error style, and the same "budget" convention described below.

## 4. Which five servers (from the master sheet)

| Rank | Idea | Server | Why it's buildable well in Rust |
|---|---|---|---|
| 1 | Woodworking / maker CAD | **`joinery`** | Pure geometry and optimization: the parametric model, cut list, sheet nesting and shelf-sag physics all run deterministically, with no external CAD needed |
| 2 | Reverse-engineering toolkit | **`binscope`** | `goblin` parses ELF/PE/Mach-O in pure Rust. Scope is deliberately **read-only static inspection** of files you already have: headers, sections, imports, strings, entropy and diffing. There is no patching, unpacking or execution |
| 3 | Stateless-spec conformance | **`lighthouse`** | `rmcp`'s client side can launch any stdio server and probe it on both protocol generations |
| 4 | Local health warehouse | **`vitals`** | Bundled SQLite (`rusqlite`), streaming XML for Apple Health exports, CSV/JSON importers, all fully offline |
| 5 | Native mobile control | **`handset`** | Wraps `adb` and `xcrun simctl`, which users already have; the novel part is compressing UI trees, which is pure Rust and unit-testable |

## 5. Uniqueness: at least 3 features per server that similar tools lack

Each list below was checked against the prior art found in the research (ShopSpec/FluidCAD, Ghidra-MCP, mcp-scan/RugSnare/MCPJam, single-vendor wearable MCPs, and ios-sim-mcp/mobile-mcp).

**joinery**
1. **Shelf-sag check.** It computes deflection from span, load, thickness and species stiffness, and warns past the 1/32″-per-foot rule. None of the woodworking MCPs found check structural sanity.
2. **Grain- and kerf-aware sheet nesting.** A guillotine packer respects grain direction per part and saw kerf, then reports the yield %.
3. **Assembly-order generator.** It derives build steps from the joinery graph (carcass before back panel, dry-fit before glue).
4. **Plain SVG cut diagrams.** Diagrams come back as SVG text that any client can render.

**binscope**
1. **Progressive disclosure.** The server offers 4 coarse tools and returns *next-step hints* instead of 100+ tools. This directly answers the HN complaint about Ghidra-MCP's 110 tools.
2. **Token budgets.** Every tool takes `max_tokens` and truncates *structurally* (top-N sections or strings, ranked by interest), never mid-record.
3. **Binary diff.** It compares two builds (sections, imports, symbols), which is useful for "what changed in this release?" and supply-chain review.
4. **Entropy map.** A per-section entropy chart flags compressed or encrypted regions at a glance.

**lighthouse**
1. **Dual-generation probe.** One run tests a server over `2026-07-28` (stateless) *and* `2025-11-25` (handshake), then reports migration readiness.
2. **`mcp.lock` drift pinning.** It stores SHA-256 hashes of every tool's name, description and schema, and `verify` names exactly which tool changed. That's a lockfile, not just a scan.
3. **Token-cost report.** It estimates how many context tokens each tool definition costs and ranks the heaviest.
4. **Description lint and a letter grade.** It flags vague descriptions, missing parameter docs and unbounded string inputs, then produces an A–F scorecard.

**vitals**
1. **Multi-vendor merge with dedup.** Overlapping samples from two devices (Apple Watch and Garmin both logging steps) are de-duplicated by time window and source priority.
2. **Explainable readiness score.** The score comes from HRV vs a 7-day baseline, sleep vs need, and acute:chronic load, and every factor is shown with its contribution.
3. **Privacy modes.** `--redact` rounds timestamps and drops device identifiers before anything reaches the model, and the database never leaves the machine.

**handset**
1. **Ref-tagged compact UI tree.** UI-Automator XML becomes `e12 Button "Sign in"` lines, about 10× fewer tokens than raw XML, and agents act by ref.
2. **Snapshot diffs.** After an action it returns only what changed, not the whole screen.
3. **`wait_for` with a predicate.** It polls until text or a ref appears, so there are no sleep-and-retry loops.
4. **One API for both platforms.** Android (`adb`) and the iOS Simulator (`simctl`) share the same tool names.

## 6. Website stack

| Option | Pros | Cons |
|---|---|---|
| Next.js | First-class on Vercel | A heavy client runtime for a mostly static page |
| Leptos / Dioxus (Rust → WASM) | "Use Rust" literally | WASM payload, hydration cost, worse first paint |
| **Static HTML + modern CSS + a small vanilla JS module** | Zero framework runtime, instant paint, deploys anywhere (including Vercel) | More hand-written CSS |

**Decision: static.** The guidelines say "CSS > Web Animations API > JS libraries" and to animate only `transform` and `opacity`. A static page meets that best, and the Rust lives where it pays off: in the servers.

### What was built
The UI features avoid generic AI landing-page fare, and every one shows real output:
1. **Recorded terminal replay.** The hero replays genuine JSON-RPC exchanges that `site/tools/record.py` captured from the release binaries. It is input-driven, never an autoplaying loop.
2. **Real cut diagram.** The joinery tile shows the actual SVG that `nest_parts` drew for the transcript's bookcase.
3. **Shelf-sag calculator.** This is joinery's formula ported to JS. A test parses the Rust species table, so the two cannot drift. The support can be dragged 1:1, with a velocity-preserving spring.
4. **Config builder.** It covers Claude Code, Claude Desktop, Cursor and Codex, with options shown only for the selected servers. The URL becomes shareable after the first change.
5. **⌘K palette.** A native `<dialog>` with combobox semantics. Keyboard opening is instant; pointer opening grows the panel out of its trigger.
6. **Dashboard** (`site/dashboard.html`). Lighthouse grades, protocol support, handshake latency, lockfile status and per-tool token cost, all from a real audit. Every chart has a sentence description and a data table.
7. **Scroll-driven reveals** via `animation-timeline: view()`, with an IntersectionObserver fallback.

The motion, material, typography and colour rules were refined after comparing the Apple HIG, Apple's kits, Emil Kowalski's *apple-design* notes and Dribbble trends. The decisions are recorded in [`UI-RESEARCH.md`](UI-RESEARCH.md).

**Verification:**
- **axe-core** reports 0 violations on both pages in light and dark mode.
- **CSP:** both pages run under the strict CSP in `site/vercel.json` with no violations.
