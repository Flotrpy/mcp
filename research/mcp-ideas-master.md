# MCP ideas: master sheet

_Merged from `hn-mcp-ideas.md` (Hacker News, 3,726 MCP stories + 39k comments) and `mcp-ideas-web.md` (dev.to 12.2k posts, Mastodon 12.3k posts, the official MCP Registry with 50,204 servers, X, and ~25 targeted web checks). Window: 2026-01-26 → 2026-10-26._

**Selection rule (from the brief):** something *has been built*, it got *a small amount of attention*, and it shows *momentum* toward something bigger.

**How each idea is scored:**
- **Momentum ★1–5:** combines engagement, growth, repeat independent builds and explicit "I'd pay / I wish" comments.
- **Crowding:** how many existing implementations there are (Low <10, Med 10–50, High 50+ in the registry or on the web).
- **Open angle:** what nobody has nailed yet.

## Summary table

| # | Idea | Niche | Momentum | Crowding | Open angle |
|---|---|---|:-:|:-:|---|
| 1 | **Woodworking / maker text-to-CAD** (cut lists, printable parts) | Maker / CAD | ★★★★★ | Med | One tool from description to parametric model, cut list, STL and supplier BOM |
| 2 | **Reverse-engineer-anything toolkit** (binaries, apps, gadgets → MCP) | Security / RE | ★★★★★ | Med | A tool-discovery layer for 100+-tool RE servers; gadget/BLE protocol capture to MCP |
| 3 | **Stateless-spec migration & conformance** ("MCP Lighthouse") | MCP infra | ★★★★★ | Med | A codemod from stateful to 2026-07-28 stateless, plus a public scorecard |
| 4 | **Unified wearables + nutrition health warehouse** (local-first) | Health | ★★★★ | High (fragmented) | Multi-vendor schema, local LLM, morning readiness brief |
| 5 | **Native mobile control for agents** (iOS Sim, real iPhone, Android) | Mobile / dev tools | ★★★★ | Med (fragmented) | One maintained, token-efficient AX-tree tool across iOS and Android |
| 6 | **EE lab loop**: KiCad + ngspice + SCPI instruments + PCB ordering | Hardware | ★★★★ | Med | Simulation-error recovery and one schematic → sim → bench → fab loop |
| 7 | **MCP compatibility proxy** (CORS / DCR / OAuth refresh fixer) | MCP infra | ★★★★ | Med | Drop-in shim for browser-hosted agents connecting to any remote server |
| 8 | **Consequential-action gate** (reversibility classifier + approval) | Safety | ★★★★ | Low–Med | Auto-tag tools as reversible or irreversible; hold sign/send/pay/delete for approval |
| 9 | **Ecosystem-wide tool-drift feed** ("MCP changelog") | Supply-chain security | ★★★★ | Low (as a feed) | A public, continuously updated diff of tool descriptions across popular servers |
| 10 | **DAW producer kit**: sample-library semantic search + audio-analysis feedback | Music | ★★★★ | Low (these parts) | CLAP-embedding sample search; LUFS/spectrum "ears" so agents hear their mix |
| 11 | **MCP composition runtime** (pipes, code mode, subagents as tools) | MCP infra | ★★★★ | Med | Sandboxed TS runtime scripting across many servers; file handles between servers |
| 12 | **Local-first smart home bundle** (Home Assistant + local LLM) | Home / IoT | ★★★★ | Med | Privacy-first packaging; the "home agent" that learns household routines |
| 13 | **MCP-only personal finance ledger** | Finance | ★★★ | High (fragmented) | Local ledger whose *only* UI is MCP; forecasting, tax lots, SimpleFIN/OFX |
| 14 | **File transfer + push/wake channel for MCP** | MCP infra | ★★★ | Low | Content-addressed file handles; app → agent push notifications |
| 15 | **Pro photo/design via open tools** (Penpot, darktable, RawTherapee, GIMP, Krita) | Creative | ★★★ | Low–Med | A cross-app "Lightroom by prompt" workflow; Figma → Penpot bridge |
| 16 | **Gov / legacy public-data MCPs with citations** | Civic data | ★★★ | High (shallow) | Evidence-grade provenance; SOAP/legacy wrappers (cadastre, planning portals) |
| 17 | **Life databases** (pets, home, household) where the agent does data entry | Personal | ★★★ | Med | Schema-on-demand trackers shared across household members |
| 18 | **Linux desktop agent layer** (AT-SPI, D-Bus, systemd/varlink, KDE/GNOME) | OS | ★★★ | Low | An integrated Linux equivalent of macOS/Windows computer-use MCPs |
| 19 | **Agent game arenas as public benchmarks** | Games / evals | ★★★ | Med | A persistent league with per-model and per-harness leaderboards |
| 20 | **Wireshark / pcap analysis** | Network security | ★★★ | Low | Awareness and polish; streaming capture with token budgets |
| 21 | **Shell / work-activity memory** ("what was I doing yesterday") | Dev productivity | ★★★ | Med | Local daemon for terminal, editor and browser events, exposed as MCP |
| 22 | **Citizen-science astronomy** (TESS / lightkurve MCP) | Science | ★★ | **None found** | A light-curve fetch/detrend/BLS-search MCP with false-positive vetting |
| 23 | **WebMCP site tooling** (declare, test, analyze site-side tools) | Web | ★★★ | Low | A Lighthouse-style audit and analytics for WebMCP-enabled sites |
| 24 | **Caregiver / elder-care coordination** (family side) | Health / family | ★★ | Very low | Open, family-side meds, visits and shared tasks with an audit trail |
| 25 | **Service-status / outage MCP** | Dev tools | ★★ | Med | Aggregate statuspage feeds so agents stop "working around" outages |
| 26 | **Spend caps & cost tripwires for agents** | Safety / cost | ★★★ | Med | A hard per-task budget proxy across providers (the $78k Codex incident) |
| 27 | **Grok Bot / new-marketplace connectors** | Distribution | ★★ | Low (new) | Be early in the marketplace opened Oct 14, 2026 |
| 28 | **Library & catalogue MCPs** (GND, Wikidata, HAL, Zotero) | Libraries / academia | ★★ | Med | Cataloguer workflows: subject heading suggestion, authority control |

---

## Detail cards

### 1. Woodworking / maker text-to-CAD ★★★★★
- **Evidence:**
  - [Ask HN: CAD for wood working with LLM support?](https://news.ycombinator.com/item?id=50067325), 128 / 127, Oct 13 2026. At least six indie builders replied with their own MCP tools.
  - [SolidCraft](https://news.ycombinator.com/item?id=50064840), 146 / 52.
  - [Printytron: describe a part, get a printable STL](https://news.ycombinator.com/item?id=49310829), 95.
  - "[Why can AI generate Super Mario but not a wedge ramp for my robot vacuum?](https://news.ycombinator.com/item?id=49405520)"
  - On Mastodon, a FreeCAD wrench tray built from about 100 measurements.
  - Registry "parametric CAD" demand grew 2.0×.
- **Prior art:**
  - ShopSpec (MCP beta)
  - FluidCAD (local MCP)
  - claydash
  - Sawdust.diy
  - neka-nat/freecad-mcp
  - build123d-mcp and openscad-mcp-server
  - Kiln (printing)
- **Open angle:** nobody combines *intent → parametric model → cut list or sheet nesting → STL/3MF → real supplier BOM* in one local tool, and builders are asking publicly.

### 2. Reverse-engineer-anything toolkit ★★★★★
- **Evidence:**
  - [Ghidra MCP](https://news.ycombinator.com/item?id=46882389), 298.
  - REA ("Reverse Engineer Anything"), reported at 115K GitHub stars in a week (Oct 2026, dev.to).
  - [Kampala (YC)](https://news.ycombinator.com/item?id=47794514), 100 / 84.
  - [Reverse-engineering web apps into agent tools](https://news.ycombinator.com/item?id=48847834), 96.
  - [Fable hacked my piano](https://news.ycombinator.com/item?id=49577129), 299 / 166.
  - Official Hex-Rays IDA MCP.
  - Demand per server is 13.3, the second-highest of any niche, with growth of 2.55×.
- **Open angle:**
  - Ghidra's 110 tools need "a discovery layer in front" (HN).
  - Consumer gadgets (vacuums, pianos, wearables) keep being RE'd one at a time. Nothing yet turns BLE sniffs into a ready MCP server.

### 3. Stateless-spec migration & conformance ★★★★★
- **Evidence:**
  - [2026-07-28 spec](https://news.ycombinator.com/item?id=49088058).
  - [Stateless MCP (Simon Willison)](https://news.ycombinator.com/item?id=49131438), 386.
  - [New MCP Roadmap](https://news.ycombinator.com/item?id=49399591), 270 / 161: "EVERY MCP UPDATE is almost certainly a breaking change".
  - Growth is **8.75×**, the fastest of any niche.
  - [Graded 36 servers, a third D/F](https://news.ycombinator.com/item?id=49002358).
  - dev.to: "Only 3% passed contract compliance".
- **Prior art:** mcpscore, MCPJam, the PostHog leaderboard and mcp-use v2.
- **Open angle:**
  - An automated *migration codemod* for the TS/Python SDKs.
  - A neutral public scorecard covering spec compliance, tool-description quality and tokens per call.

### 4. Unified wearables + nutrition health warehouse ★★★★
- **Evidence:**
  - [Creating a Garmin MCP](https://news.ycombinator.com/item?id=50081045), 61 / **74**: "hard to get metrics from multiple devices in one place."
  - The registry has 33 Garmin, 69 Oura, 19 Whoop, 11 Strava, 13 Apple Health and 3 Cronometer servers, all single-vendor.
  - Nutrition demand growth is 3.7×.
  - Privacy pushback: "there's no way I'd send that data to Anthropic or OpenAI."
- **Open angle:** one local store with vendor sync, a local-LLM option and a daily brief. People like the multi-device "morning readiness" use case.

### 5. Native mobile control ★★★★
- **Evidence:**
  - Mobile is the fastest-growing HN niche (11 → 27 posts).
  - The explicit ask: "What I'm missing is a tool to inspect iOS simulator… The Xcode MCP wasn't really helpful" ([thread](https://news.ycombinator.com/item?id=47501116)).
  - Android MCP called a "killer app" ([thread](https://news.ycombinator.com/item?id=49362047)).
  - Real-iPhone MCP demo on X (Oct 23).
  - Google AI Edge Gallery on-device MCP (dev.to, 55 / 16).
- **Prior art:** 16 iOS servers in the registry (ios-sim-mcp, XC-MCP and others) and 144 Android ones.
- **Open angle:** consolidation and quality. One maintained, AX-tree-first tool that works on both platforms.

### 6. EE lab loop ★★★★
- **Evidence:**
  - [SPICE → scope → verification](https://news.ycombinator.com/item?id=47801255), 121.
  - [Labgrid-MCP](https://news.ycombinator.com/item?id=49182016).
  - [MCP for ordering assembled PCBs](https://news.ycombinator.com/item?id=50155440) (Oct 19).
  - dev.to: "I gave Claude a PCB design tool over MCP" (Oct 24).
  - KiCad demand growth is 2.4×.
- **Prior art:** KiCAD-MCP-Server (216 tools), the K-Dense labmcp suite (Rigol, Keithley, generic SCPI), Digilent WaveForms and EasyEDA copilot.
- **Gap:** **ngspice has 0 servers.** The HN failure mode is agents burning tokens on SPICE convergence and model-card errors.

### 7. MCP compatibility proxy ★★★★
- **Evidence:**
  - [Who uses MCP in production?](https://news.ycombinator.com/item?id=49548600), 198 / 201: CORS, bespoke OAuth and no DCR are "the biggest issues".
  - [Zero-Touch OAuth](https://news.ycombinator.com/item?id=48592163), 278: "we lie to the MCP client telling it we support DCR".
  - OAuth niche growth is 2.66×.
- **Open angle:** a single open-source shim for browser-hosted and enterprise agents that normalizes auth to any remote MCP server.

### 8. Consequential-action gate ★★★★
- **Evidence:**
  - [Claude Code signed a contract for me without asking](https://news.ycombinator.com/item?id=49798257), 51 / 98: "signing and sending looked like the same step as saving a draft."
  - [Codex agents consumed $78k](https://news.ycombinator.com/item?id=49861047).
  - Commenter: "put guardrails around consequential actions."
- **Prior art:** Approval Gate (MCP App), Gatehouse, TrueFoundry approvals (commercial), Claw Patrol and SentinelGate.
- **Open angle:** an automatic *reversibility classifier* plus policy, so users don't have to write rules by hand.

### 9. Ecosystem-wide tool-drift feed ★★★★
- **Evidence:**
  - dev.to: "4.4% of MCP servers changed their tool contract in 36 hours" and "A fake MCP server spent three months earning trust".
  - HN: [Blender MCP maintainer hacked](https://news.ycombinator.com/item?id=49238028) and [AgentBaiting malware](https://news.ycombinator.com/item?id=48978294).
  - SquatGuard (namespace squatting) on Mastodon.
- **Prior art:** local pinning tools (mcp-scan, RugSnare, MCP Sentinel and others).
- **Open angle:** a *public* crawler and changelog for the top N servers that alerts subscribers, like Dependabot advisories for MCP.

### 10. DAW producer kit ★★★★
- **Evidence:**
  - [Ableton Live MCP](https://news.ycombinator.com/item?id=47999656), 123 / 79.
  - [DAW with MCP](https://news.ycombinator.com/item?id=50064438), 84.
  - Comments ask for "semantic search of sample library" and a mastering "feedback loop of actually analyzing the processed audio".
- **Prior art:** many DAW controllers (Ableton ×5, FL ×4, Logic, Reaper, openDAW). There's only soundgrep for sample search, a few LUFS tools, and 0 Bitwig servers.
- **Open angle:** the *perception* side (search and listening), not more transport control.

### 11. MCP composition runtime ★★★★
- **Evidence:**
  - "Biggest issue with MCP continues to be that it's hard to compose" ([You said no MCP](https://news.ycombinator.com/item?id=49906637), 687 / 363).
  - [mcp2cli](https://news.ycombinator.com/item?id=47305149), 146.
  - Cloudflare Code Mode.
  - dev.to "Save bloated MCP with Code Mode" (73 / 31).
  - "Nobody's come up with a good way to turn the subagent tool into something you can access via the CLI."
- **Open angle:** a sandboxed runtime with cross-server pipes, file handles and subagents exposed as tools.

### 12. Local-first smart home bundle ★★★★
- **Evidence:**
  - Home Assistant MCPs are frequently cited in [What tools have you made since AI](https://news.ycombinator.com/item?id=48449187) (445 / 788).
  - The "Home Agent" learns from household feedback.
  - German-language Mastodon privacy pushback.
  - BSI guidance.
- **Prior art:** 10 HA servers in the registry, including ha-mcp.
- **Open angle:** an opinionated package of local LLM, HA MCP, a gated-actions policy and household memory.

### 13. MCP-only personal finance ledger ★★★
- **Evidence:** [Stoke](https://news.ycombinator.com/item?id=50122776), [Fungible](https://news.ycombinator.com/item?id=48271942), [Cadence: "no AI features, just an MCP server"](https://news.ycombinator.com/item?id=49097110), "MCP-server-ONLY finance app" and Pane. That's 5+ independent builds.
- **Supply:** 514 budget/finance servers, mostly SaaS wrappers.
- **Open angle:** a local, open ledger with *no UI* plus importers.

### 14. File transfer + push/wake channel ★★★
- **Evidence:**
  - "My biggest frustration… no built-in way of uploading and downloading files" and "I wish MCP supported push notifications" ([687-pt thread](https://news.ycombinator.com/item?id=49906637)).
  - Bluebeam/PDF users report "no way to pipe data into the MCP".
- **Open angle:** a convention plus reference implementation, small enough to become a de-facto extension.

### 15. Pro photo/design via open tools ★★★
- **Evidence:**
  - [Figma restricts MCP clients](https://news.ycombinator.com/item?id=49922729), 189 / 106.
  - "Use these tools using MCP so I don't have to learn the particulars" ([Adobe thread](https://news.ycombinator.com/item?id=47826399)).
  - The official Penpot MCP and darktable-mcp now exist.
- **Open angle:** a RAW → edit → export workflow across darktable/RawTherapee/GIMP, plus a Figma → Penpot migration MCP.

### 16. Gov / legacy public data with citations ★★★
- **Evidence:**
  - [Spain cadastre SOAP → MCP](https://news.ycombinator.com/item?id=48793298).
  - [Scraping 241 UK council planning portals](https://news.ycombinator.com/item?id=47931470), 48 / 70.
  - dev.to Korean gov data plus x402.
  - "Gov open data → agent-callable *evidence*".
  - HAL (France) and lobid-GND on Mastodon.
- **Open angle:** provenance and citations as a first-class part of the tool output; country-by-country legacy wrappers.

### 17. Life databases ★★★
- **Evidence:**
  - [Home Memory, "killer app for mcp servers"](https://news.ycombinator.com/item?id=47806891) (puppy allergy tracking).
  - [Edi Life OS](https://news.ycombinator.com/item?id=50014150).
  - HutchDB, "an MCP service… to store data for you".
  - FurEver Care (dev.to).
  - A Scouts unit-management MCP.
- **Open angle:** schema-on-demand, multi-user household trackers.

### 18. Linux desktop agent layer ★★★
- **Evidence:**
  - [systemd MCP via varlink](https://github.com/mvo5/systemd-mcp) (Mastodon, 14 / 10).
  - "I'd love to see an agentic OS where the OS and apps talk MCP" ([thread](https://news.ycombinator.com/item?id=49826715)).
- **Prior art:** kwin-mcp, gnome-ui-mcp and a Hyprland CUA MCP. KDE has 0 registry entries.
- **Open angle:** one integrated AT-SPI + D-Bus + portal stack.

### 19. Agent game arenas ★★★
- **Evidence:** [SimCity](https://news.ycombinator.com/item?id=46946593) 216, [MTG](https://news.ycombinator.com/item?id=47049227) 117, [DOOM](https://news.ycombinator.com/item?id=47939079) 93, SpaceMolt, Clawfight and Slay the Spire 2 (dev.to).
- **Open angle:** persistence plus leaderboards. It's a benchmark disguised as a game.

### 20. Wireshark / pcap ★★★
- **Evidence:**
  - "What's missing now is an MCP for Wireshark" ([thread](https://news.ycombinator.com/item?id=48777144)).
  - Demand per server is 10.8.
- **Prior art:** wireshark-mcp (51 tools), tshark-mcp and mcpcap.
- **Open angle:** awareness, polish and token-budgeted streaming capture.

### 21. Shell / work-activity memory ★★★
- **Evidence:** "Claudhd" daemon, which logs shell and editor events into SQLite and exposes them as MCP ("what was I doing yesterday"), from the [tools-you-built thread](https://news.ycombinator.com/item?id=48449187).
- **Related:** Screenpipe (YC) at [88](https://news.ycombinator.com/item?id=49024620) and Breadcrumb at 49.
- **Open angle:** local and terminal-native, without screen recording.

### 22. Citizen-science astronomy ★★
- **Evidence:** "[I think I found a planet nobody knew existed. I used Claude Code](https://news.ycombinator.com/item?id=50002665)", 302 / 132, though the thread is skeptical.
- **Prior art:** astro_mcp, astroquery MCP and exoplanet-archive. **No lightkurve/TESS MCP found.**
- **Open angle:** fetch, detrend and transit search with built-in false-positive vetting, so results aren't slop.

### 23. WebMCP site tooling ★★★
- **Evidence:** dev.to [webMCP live demo](https://dev.to/sylwia-lask/is-this-how-well-build-websites-soon-webmcp-live-demo), 157 / 145, and the browser-agent demo at 187 / 126.
- **Open angle:** audit, testing and analytics for site-declared tools.

### 24. Caregiver coordination ★★
- **Supply:** 0 registry servers.
- **Prior art:** dala.care (vendor) and university projects.
- **Demand:** small but real (an elder-care product builder on HN, and caregiver voice-AI founders).
- **Open angle:** a family-side open tool for meds, visits, shared tasks and audit trails.

### 25–28. Smaller bets
- **Service status (25):** an explicit HN ask; registry growth is 2.25×.
- **Spend caps (26):** the $78k Codex story; "Does OpenAI not support spending caps?"
- **Grok Bot marketplace (27):** opened Oct 14, 2026.
- **Library catalogue MCPs (28):** lobid-GND, Wikidata, HAL and Zotero (18 servers).

---

## Long-tail niches: thin supply, early demand
These are the registry niches with ≤7 servers. Demand is small, so they suit a weekend MVP and a dev.to write-up rather than a startup.

| Niche | Registry servers | Demand signal seen |
|---|---:|---|
| Elder care / caregiving | 0 | HN elder-care builder; dala.care vendor launch |
| Woodworking | 1 | **Strong:** Ask HN, 128 points (see #1) |
| Aquarium / reef | 2 | Minimal |
| Beekeeping | 2 | Minimal |
| Gardening / plant care | 2 | A professional gardener runs job scheduling via a custom MCP (HN) |
| OSS maintainer triage | 2 | dev.to "Ghost Maintainer" (64 reactions) |
| Home weather station | 2 | A few comments |
| Homebrewing | 4 | Minimal |
| Knitting / sewing | 4 | Minimal |
| Birding (eBird) | 5 | Minimal |
| Sign language / braille | 5 | Minimal; worth an accessibility angle |
| Board games (BGG) | 6 | Board-game and TTRPG tools mentioned in the HN tools thread |
| Smart meter / home energy | 6 | Minimal; EU energy-price interest |
| Telescope control (INDI/ASCOM) | 6 | Pairs with #22 |
| Satellite tracking (TLE) | 7 | Minimal |
| Ham radio (Hamlib/WSJT-X/APRS/Meshtastic) | ~2 | Registry growth 3.9× from a low base |

## Avoid: crowded or oversupplied
- **x402 / pay-per-call wrappers:** 2,323 registry servers, and the lowest demand per server of any niche.
- **Generic agent memory / knowledge graphs:** 1,542 servers, and dozens of Show HNs.
- **Generic context/token reducers:** Context Mode (570), Semble (445), mcp2cli and others.
- **Enterprise MCP gateways:** Bifrost, Archestra ($10M raise), TrueFoundry, Manufact and others.
- **Generic browser automation:** Chrome DevTools MCP (604), Safari MCP (272), Browser Harness and others.
- **Obsidian / Notion connectors:** 64+ Obsidian servers, and demand is decelerating.

## Method caveats
- Niche matching is regex-based. Counts are directional, not exact.
- **Reddit could not be accessed directly** (it blocks crawlers and requests, and the archive refuses agents). X coverage is limited to indexed posts.
- Mastodon timelines are recency-biased, so Mastodon growth figures are excluded from the momentum scores.
- "Registry servers" counts only the official MCP Registry. Many servers live only on GitHub, npm or Glama.
