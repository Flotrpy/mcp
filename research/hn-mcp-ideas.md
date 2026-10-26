# MCP project ideas with momentum — Hacker News deep search

_Research window: 2026-01-26 → 2026-10-26 (9 months). Collected 2026-10-26._

## Method

- **Sources:** HN Algolia search API (`hn.algolia.com/api/v1/search_by_date`) plus the official Firebase API (`hacker-news.firebaseio.com/v0`) for the live top/new/best feeds.
- **Collector:** `research/hn_algolia_collect.py`, which runs 26 story queries and 8 comment queries. Each query is split into 7-day windows for stories and 2-day windows for comments, so no query hits Algolia's 1,000-result cap.
- **Raw haul:** 32,007 stories and 39,120 comments.
- **MCP-specific stories:** 3,726 had "MCP" or "model context protocol" in the title, URL or text. Of those, 2,185 were Show HN posts, 288 scored 10+ points and 65 scored 100+.
- **Monthly volume:** Feb 615 · Mar 623 · Apr 436 · May 294 · Jun 331 · Jul 381 · Aug 342 · Sep 314 · Oct 310. The hype peaked in Feb–Mar, and since summer there's a steady base of about 300–380 a month.
- **Thread deep-dives:** I pulled the full comment trees for the 28 highest-signal threads (Ask HN: who uses MCP in production, MCP roadmap, Zero-Touch OAuth, Figma, Garmin, Ableton and others) and searched them for pain points and "I wish" statements.
- **Momentum filter (your criterion):** an idea qualifies if it has *already been built at least once* and got *modest attention*, plus at least one of: a high comment-to-point ratio, repeat independent builds, explicit "I'd pay / I wish" comments, or a niche whose post count grew from Feb–May to Jun–Oct.

Score notation: `points / comments`.

---

## Top momentum ideas (ranked)

### 1. Unified wearables & health-data MCP (local-first)
- **Evidence:**
  - [Creating a Garmin "MCP"](https://news.ycombinator.com/item?id=50081045), **61 / 74**. That's more comments than points, which signals real demand. Commenters say: "It's hard to get metrics from multiple devices in one place." One person built a Claude morning brief that compares sleep, HRV, training status, calendar and weather.
  - [MCP server that connects Claude to all your wearables](https://news.ycombinator.com/item?id=47658617), 20 / 13.
  - [Talk to your Oura Ring data](https://news.ycombinator.com/item?id=47805591).
  - [Fitness MCP](https://news.ycombinator.com/item?id=47433892).
  - [PTrace – iPhone running tracker with MCP](https://news.ycombinator.com/item?id=50117238), Oct 2026.
- **Gap:**
  - Nothing aggregates *multiple* vendors (Garmin, Oura, Whoop, Apple Health, Strava) behind one schema.
  - Users explicitly want **local-only** processing: "there's no way I'd ever send that data to Anthropic or OpenAI".
  - Garmin lacks strength-training data, so there's room for user-defined exercise logs.
- **Build:** a self-hosted health warehouse (SQLite/DuckDB) with vendor sync, an MCP server, a local-LLM option and a "daily readiness brief" prompt.

### 2. iOS Simulator / mobile app control MCP
- **Evidence:**
  - Explicit ask in [ProofShot](https://news.ycombinator.com/item?id=47501116) (161 / 106): "What I'm missing is a tool to inspect iOS simulator the same way. Is there a tool for that yet? The Xcode MCP wasn't really helpful."
  - [MCP app for Android, drive apps via AI](https://news.ycombinator.com/item?id=49362047), called "honestly feels like a killer app for automation", with token use described as "well optimized".
  - [Finalrun – mobile testing with vision](https://news.ycombinator.com/item?id=47676044), 28 / 15.
  - [Agents can create iOS Live Activities](https://news.ycombinator.com/item?id=50014480).
- **Momentum:** the mobile niche is the **fastest-growing** one in the data, with 11 posts in Feb–May and 27 in Jun–Oct.
- **Gap:** a token-efficient, accessibility-tree-based iOS Simulator and real-device MCP that gives coding agents "eyes" on native apps, matching what Playwright and Chrome DevTools MCP do for the web.

### 3. DAW / music-production MCPs beyond Ableton
- **Evidence:**
  - [Ableton Live MCP](https://news.ycombinator.com/item?id=47999656), **123 / 79**.
  - [Pattern-first DAW with MCP](https://news.ycombinator.com/item?id=50064438), 84 / 41.
  - [MCP that composes on a hardware synth](https://news.ycombinator.com/item?id=47804220).
- **Explicit asks:**
  - "I wish there was something like this for FL Studio."
  - "Semantic search of sample library."
  - The mastering chain "is missing the full feedback loop of actually analyzing the processed audio signal."
- **Gap:**
  - An FL Studio, Logic or Reaper MCP.
  - A semantic **sample-library search** MCP using audio embeddings.
  - An **audio-analysis feedback** MCP covering LUFS, spectrum and stereo image, so agents can hear the result of their changes.

### 4. Electronics lab / EE workflow MCP (SPICE, scopes, KiCad, embedded)
- **Evidence:**
  - [SPICE → oscilloscope → verification with Claude Code](https://news.ycombinator.com/item?id=47801255), **121 / 35**.
  - [Labgrid-MCP, AI agents drive embedded hardware labs](https://news.ycombinator.com/item?id=49182016).
  - [Ratchet, an agent reflashes BIOS over CH341A](https://news.ycombinator.com/item?id=48604403).
- **Gap:**
  - There's essentially no KiCad or PCB MCP traction on HN: one post, 2 points.
  - A commenter names the real failure mode: agents waste their token budget on simulator environment errors ("SPICE convergence failure, missing model card, wrong .include path").
- **Build:** a KiCad + ngspice MCP with a curated error-recovery layer, plus SCPI instrument control (scope, PSU, signal generator).

### 5. Text-to-CAD for makers (functional parts, not art)
- **Evidence:**
  - [SolidCraft – parametric CAD in Rust with MCP](https://news.ycombinator.com/item?id=50064840), **146 / 52**, Oct 2026. The author hopes someone builds a frontend for it.
  - [Why can AI generate Super Mario but not a wedge ramp for my robot vacuum?](https://news.ycombinator.com/item?id=49405520), 11 / 6.
  - Onshape MCP posts are minor.
  - FreeCAD, OpenSCAD and Fusion each have 1–5 posts with low scores.
- **Gap:** an MCP that generates **printable functional parts** with dimensional constraints, runs a slicer check and exports STL/3MF, with build123d/OpenSCAD as the backend.

### 6. "MCP for Wireshark" / packet & protocol analysis
- **Evidence:**
  - Explicit ask in [Mcpsnoop – Wireshark for MCP](https://news.ycombinator.com/item?id=48777144) (64 / 22): "What's missing now is an MCP for Wireshark."
  - [Ghidra MCP](https://news.ycombinator.com/item?id=46882389), **298 / 68**, shows the security-research audience is big.
  - Reverse-engineering web apps into agent tools: [96 / 40](https://news.ycombinator.com/item?id=48847834) and [Kampala, 100 / 84](https://news.ycombinator.com/item?id=47794514).
- **Gap:** a tshark/pcap MCP with streaming capture, display filters, flow summaries and protocol dissection, kept token-budgeted. Ghidra also has 110 tools, and a commenter says that's "exactly where you need a discovery layer in front."

### 7. Personal-finance apps that are "MCP-only, no UI"
- **Evidence:**
  - [Stoke Money](https://news.ycombinator.com/item?id=50122776), 33 / 24.
  - [Fungible (terminal finance)](https://news.ycombinator.com/item?id=48271942), 30 / 15.
  - [Cadence Money, "no AI features, just an MCP server"](https://news.ycombinator.com/item?id=49097110).
  - [MCP-server-ONLY personal finance app](https://news.ycombinator.com/item?id=47220211).
  - [Pane](https://news.ycombinator.com/item?id=47240363).
- **Momentum:** at least 5 independent builds, and the finance niche is still growing (50 posts in Feb–May, 54 in Jun–Oct).
- **Gap:** a local-first ledger (Plaid/SimpleFIN/OFX import) whose *only* interface is MCP, with forecasting, categorization and tax-lot tools. None of these has broken out yet.

### 8. Agent payments & paid-MCP monetization
- **Evidence:**
  - [Ask HN: Is anyone using A2A?](https://news.ycombinator.com/item?id=48582679), 96 / 45: "What's 100% missing is payments."
  - [Ask HN: Has anyone built an agent that spends real money?](https://news.ycombinator.com/item?id=47371289)
  - [Tilde Pay – give your agent a bank account](https://news.ycombinator.com/item?id=49070028).
  - Manufact founder: "monetization is still not regulated by the protocol."
  - A scraper-fight thread says "the missing piece is some micro transaction layer and attestation."
- **Gap:** pay-per-call MCP middleware with budgets, receipts and per-user attestation. Site owners would charge bots instead of blocking them.

### 9. MCP compatibility proxy for browser-based agents (CORS / DCR / OAuth fixer)
- **Evidence:**
  - [Ask HN: Who is using MCP in production?](https://news.ycombinator.com/item?id=49548600), **198 / 201**. A Fortune 100 engineer: "The biggest issues are … misconfigured CORS … bespoke OAuth setup which doesn't work with @modelcontextprotocol/client, or they don't support Dynamic Client Registration."
  - [Zero-Touch OAuth for MCP](https://news.ycombinator.com/item?id=48592163), **278 / 103**: "we lie to the MCP client telling it we support DCR."
  - The "annoying reauth every 8-12 hours" is a common complaint.
  - [Mcptunnels – ngrok for MCP with OAuth](https://news.ycombinator.com/item?id=49527807), 15.
- **Gap:** an open-source shim that sits in front of any remote MCP server. It would normalize OAuth, fake or broker DCR, add CORS and refresh tokens.

### 10. File transfer & push notifications for MCP
- **Evidence:** from [You said no MCP](https://news.ycombinator.com/item?id=49906637), **687 / 363**:
  - "My biggest frustration with MCP … no built-in way of uploading and downloading files."
  - "I wish the MCP protocol supported push notifications from the app to the model."
  - In the production thread, a Bluebeam/PDF user said "there's no way to pipe data into the MCP directly."
- **Gap:** a sidecar or convention for content-addressed file handles plus a webhook-to-agent wake channel. [Tether](https://news.ycombinator.com/item?id=47199729) did content-addressed handles between LLMs, a small proof that it works.

### 11. MCP composition / piping runtime
- **Evidence:**
  - "The biggest issue with MCP continues to be that it's hard to compose" ([You said no MCP](https://news.ycombinator.com/item?id=49906637)).
  - [Mcp2cli](https://news.ycombinator.com/item?id=47305149), 146 / 100.
  - [Mcptoon](https://news.ycombinator.com/item?id=49253721), 73 / 49.
  - [Making MCP cheaper via CLI](https://news.ycombinator.com/item?id=47157398), 324 / 119.
  - Cloudflare's [Code Mode](https://news.ycombinator.com/item?id=47089505).
  - "nobody's come up with a good way to turn the 'subagent' tool into something you can access via the CLI."
- **Gap:** a sandboxed JS/TS runtime that lets agents script across multiple MCP servers (unix pipes for MCP), with subagents exposed as callable tools.

### 12. MCP server quality grading, linting & spec-migration
- **Evidence:**
  - [I graded 36 popular MCP servers; a third got D/F](https://news.ycombinator.com/item?id=49002358).
  - [Mcpscore – validate compliance with the new MCP version](https://news.ycombinator.com/item?id=49175296).
  - [MCPJam testing & evals](https://news.ycombinator.com/item?id=49745351).
  - [PostHog MCP leaderboard](https://news.ycombinator.com/item?id=50109689).
  - The [2026-07-28 stateless spec](https://news.ycombinator.com/item?id=49088058) plus "EVERY MCP UPDATE is almost certainly a breaking change" ([roadmap thread](https://news.ycombinator.com/item?id=49399591), 270 / 161).
- **Gap:**
  - An "MCP Lighthouse" covering spec compliance, tool-description quality, token cost per call and agent-success evals.
  - An automatic **stateful → stateless migration** codemod.

### 13. MCP supply-chain security & traffic monitoring
- **Evidence:**
  - [Blender MCP maintainer GitHub account hacked](https://news.ycombinator.com/item?id=49238028).
  - [AgentBaiting: fake skills & MCP servers delivered malware](https://news.ycombinator.com/item?id=48978294).
  - [ANSI escape injection in MCP servers](https://news.ycombinator.com/item?id=48989006), 60 / 36.
  - [Traceforce (YC S26)](https://news.ycombinator.com/item?id=48937020).
  - [State of MCP Security pdf](https://news.ycombinator.com/item?id=48884647).
- **Gap:**
  - Signed or pinned MCP server releases (a Sigstore for MCP).
  - A diff-alert when a server's tool descriptions change after install, to catch rug-pulls.
  - An endpoint inventory of which MCPs each employee runs.

### 14. "Life database" MCPs (agent does the data entry)
- **Evidence:**
  - [Home Memory – a local DB of my house](https://news.ycombinator.com/item?id=47806891): "this is a killer app for mcp servers." A commenter tracks a puppy's food allergies by voice.
  - [Edi Life OS](https://news.ycombinator.com/item?id=50014150), 41 / 17.
  - [Personal Context MCP](https://news.ycombinator.com/item?id=49571566), 28.
  - [Epitome](https://news.ycombinator.com/item?id=47075158).
- **Gap:** templated, schema-on-demand trackers for pets, plants, home maintenance, medications and kids' milestones, shared across household members.

### 15. Pro creative apps via MCP (Adobe-class tools, open Figma alternatives)
- **Evidence:**
  - [Figma restricts MCP to whitelisted clients](https://news.ycombinator.com/item?id=49922729), **189 / 106**. Users are angry that Figma's MCP can do things its API can't.
  - [The creative software industry declared war on Adobe](https://news.ycombinator.com/item?id=47826399): "There should be a way where I can use these tools using MCP so that I don't have to learn the particulars."
  - [CSS Studio](https://news.ycombinator.com/item?id=47702196), 175 / 107.
- **Gap:**
  - Open MCPs for Penpot, Krita, GIMP, darktable/RawTherapee (a "Lightroom by prompt") and Inkscape.
  - A Figma-file to open-format bridge.

### 16. Agent arenas & games as benchmarks
- **Evidence:**
  - [AI agents play SimCity](https://news.ycombinator.com/item?id=46946593), **216 / 72**.
  - [LLMs play Magic: The Gathering](https://news.ycombinator.com/item?id=47049227), 117 / 83.
  - [DOOM MCP app](https://news.ycombinator.com/item?id=47939079), 93.
  - [SpaceMolt](https://news.ycombinator.com/item?id=46914267).
  - [Clawfight](https://news.ycombinator.com/item?id=49658483), 13 / 17.
  - [Game save-file MCP](https://news.ycombinator.com/item?id=47477905).
- **Gap:** a persistent, open MCP game league with leaderboards per model and harness, half game and half public benchmark.

### 17. Legacy / government API → MCP wrappers
- **Evidence:**
  - [Spain's cadastre API is SOAP from 2003, so I built a JSON wrapper (+MCP)](https://news.ycombinator.com/item?id=48793298), 17 / 10.
  - IRS e-file-as-a-tool-call startup (comment on a [dead] story).
  - [OJCP – open protocol for agent-consumable job data](https://news.ycombinator.com/item?id=49273922).
- **Gap:** a catalogue of public-sector MCPs, covering land registries, business registries, court dockets, procurement, statistics offices and permits, starting with countries with bad APIs.

### 18. Agentic desktop / OS-level MCP
- **Evidence:**
  - "I'd love to see an agentic OS where the OS and apps can both talk in terms of MCP" ([Modernizing the open-source desktop](https://news.ycombinator.com/item?id=49826715)).
  - [OculOS – any desktop app as JSON API via accessibility tree](https://news.ycombinator.com/item?id=47285616).
  - Clop's CLI + MCP with elicitation, and BetterTouchTool's MCP (comments in the 687-pt thread).
  - [Minicor (YC) Windows desktop automation](https://news.ycombinator.com/item?id=48280729), 105 / 64.
- **Gap:** a Linux desktop (GNOME/KDE via D-Bus and AT-SPI) MCP, since macOS and Windows get all the attention.

### Smaller explicit asks (single comments, worth a weekend)
- **Service-status MCP:** "Someone needs to make an mcp server … so it can check if services are down" ([GitHub outage thread](https://news.ycombinator.com/item?id=46950530)). It would aggregate statuspage.io and the cloud status feeds.
- **Kagi search MCP:** "I would love a Kagi MCP … I would pay for this" ([Back to Kagi](https://news.ycombinator.com/item?id=49008244)).
- **"Caveman" context compressor for non-code files:** [comment](https://news.ycombinator.com/item?id=47799392).
- **Reliable Chrome-MCP reconnect / restart tool:** "There should be a better way to restart just the MCP" ([comment](https://news.ycombinator.com/item?id=47573206)).
- **Credentials + sandboxing for shared `.skill` packages:** "The only thing missing for me is credentials and sandboxing" ([MCP was always a bad idea?](https://news.ycombinator.com/item?id=49779329)).

---

## Hot but crowded (high attention, many builders, so not "not done yet")
- **Context/token reduction:**
  - [Context Mode](https://news.ycombinator.com/item?id=47193064), 570.
  - [Semble](https://news.ycombinator.com/item?id=48169874), 445.
  - mcp2cli and lazy-tool.
- **Agent memory / knowledge bases:**
  - [OpenKnowledge](https://news.ycombinator.com/item?id=48675435), 381.
  - [wuphf LLM wiki](https://news.ycombinator.com/item?id=47899844), 260.
  - Dozens of smaller builds.
- **Browser automation:**
  - [Chrome DevTools MCP](https://news.ycombinator.com/item?id=47390817), 604.
  - [Safari MCP](https://news.ycombinator.com/item?id=48769639), 272.
  - Browser Harness and others.
- **Credential vaults / agent firewalls:**
  - OneCLI (161), Agent Vault (156), Claw Patrol (112).
  - SentinelGate, Zerobox and others.
- **MCP hosting / registries:**
  - Manufact (111), MCP Harbor, MCPNest and others.

## Macro trends to keep in mind
1. **The "MCP vs CLI vs Skills" debate dominates engagement.** Six of the top 15 threads are about it. The rough consensus is that MCP still wins for **remote, multi-tenant, browser-hosted and security-bounded** use, and CLIs win for local coding agents. Build ideas aimed at remote and non-developer users.
2. **Stateless spec (2026-07-28)** and **Enterprise-Managed Auth** are reshaping the server ecosystem. Migration tooling is an open window.
3. **Domain-expert MCPs are where the breakout Show HNs come from:** Ghidra, Ableton, SPICE, SimCity, Garmin, CAD. Generic infrastructure MCPs rarely break out now.

## Raw data
- `research/data/hn_live_feeds.json`: the live top/new/best scan (206 matches) from `hn_mcp_search.py`.
- The full Algolia haul is regenerable with `python3 research/hn_algolia_collect.py`.
