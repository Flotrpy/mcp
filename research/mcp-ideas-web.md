# MCP projects & ideas across the wider web

_Window: 2026-01-26 → 2026-10-26. Collected 2026-10-26. Companion to `hn-mcp-ideas.md`; the merged list is in `mcp-ideas-master.md`._

## Sources and what was reachable

| Source | How | Haul | Notes |
|---|---|---|---|
| **dev.to** | Public API (`research/web_collect.py`), 12 tags | 108,765 articles → **12,221 MCP posts** | MCP volume keeps climbing (Feb 395 → Jul–Oct ~1,600/month) |
| **Mastodon** | Tag timelines on 7 instances × 5 tags | 34,892 posts → **12,326 MCP posts** | Timelines cap at the most recent ~2,400 posts per tag, so recent months are over-represented. Bot or aggregator accounts were removed before ranking |
| **Official MCP Registry** | `registry.modelcontextprotocol.io/v0/servers?version=latest` | **50,204 servers** | Used as the *supply* side of the gap analysis |
| **X / Twitter** | Web search restricted to x.com | ~20 relevant posts | Only indexed posts are reachable, no API |
| **Web at large** | ~25 targeted searches (Glama, mcpservers.org, PyPI, forums, vendor blogs) | — | Used to check whether candidate gaps were already built |
| **Reddit** | ✗ **Not reachable** | — | reddit.com blocks both the search crawler and direct requests (403/429). PullPush explicitly refuses agent scraping, so I respected that. Reddit evidence here only comes second-hand, via HN or dev.to links |
| **Lobsters** | Search endpoint | 0 | The JSON search returned nothing |
| **Bluesky** | Public API | ✗ 401/403 | Requires auth now |

**Reproduce:**
1. `python3 research/web_collect.py`
2. `python3 research/hn_algolia_collect.py`
3. Filter both outputs to MCP posts.
4. Run `research/supply_demand_gap.py`, which writes `research/data/niche_supply_demand.csv`.

---

## Headline findings

1. **Supply has exploded and is mostly commercial.**
   - The registry lists 50,204 servers.
   - About 2,300 mention x402, pay-per-call or stablecoins, but those topics get little engagement on HN or dev.to. Payment servers are oversupplied.
   - "Agent payments" ranks *last* on demand per server.
2. **Almost nothing is truly unbuilt.** Every HN gap I checked has at least one implementation. So the opportunity is in niches where:
   - attempts are fragmented (many 0–20-star repos, no clear winner),
   - demand is visible (HN threads, dev.to engagement), and
   - demand is growing.

   That is exactly your "small attention, momentum" bar.
3. **dev.to and HN disagree on timing.**
   - HN MCP volume peaked in Feb–Mar 2026 and then plateaued.
   - dev.to has grown every month.
   - Practitioners are still ramping up while HN's taste-makers have moved on to "MCP vs CLI" debates. A tutorial-plus-tool launch on dev.to still has a growing audience.
4. **Domain-expert, local-first and hardware niches** have the best demand-to-supply ratio.

## Demand vs supply by niche (top of `niche_supply_demand.csv`)

`demand` = HN points+comments + dev.to reactions+comments + Mastodon mentions. `supply` = registry servers matching. `growth` = Jun–Oct vs Feb–May post count, normalized for month count.

| Niche | Registry servers | Demand | Demand / server | Growth |
|---|---:|---:|---:|---:|
| Notes / PKM (Obsidian, Notion, Joplin…) | 153 | 2,970 | 18.8 | 0.13 ↓ |
| **Reverse engineering** (Ghidra/IDA/Frida) | 43 | 636 | 13.3 | **2.55 ↑** |
| **Music DAW** (Ableton/FL/Logic/Reaper) | 25 | 347 | 11.6 | 0.48 |
| Blender / 3D art | 272 | 3,044 | 11.0 | 1.76 ↑ |
| **Wireshark / pcap** | 5 | 108 | 10.8 | 0.53 |
| **Spec migration / stateless** | 160 | 1,556 | 9.4 | **8.75 ↑↑** |
| Figma / design | 85 | 583 | 6.5 | 0.91 |
| MCP security scanner / pinning | 46 | 233 | 4.6 | 1.16 |
| Home Assistant / IoT | 54 | 262 | 4.4 | 0.83 |
| **Parametric CAD** | 58 | 277 | 4.4 | **2.0 ↑** |
| Lab instruments / SPICE | 57 | 192 | 3.1 | 1.2 |
| iOS Simulator / Xcode | 45 | 115 | 2.3 | 0.6 |
| OAuth / auth proxy | 522 | 1,206 | 2.3 | **2.66 ↑** |
| **KiCad / PCB** | 50 | 87 | 1.6 | **2.4 ↑** |
| Service status / outage | 316 | 335 | 1.0 | 2.25 ↑ |
| Agent payments / x402 | **2,323** | 317 | **0.14** | 1.3 |

_Regex-based, so treat it as directional. "Notes/PKM" is high-demand but decelerating and crowded with 64+ Obsidian servers._

### Near-zero supply in the registry (exact-term counts)
- **Zero servers:** Penpot, GIMP, darktable/RawTherapee, KDE, Matrix, Bitwig, ngspice, Binary Ninja, Klipper/Moonraker, local sample-library search, elder care / caregiving.
- **One server:** Wireshark, OctoPrint, Logseq, Immich, Frigate, Inkscape, Krita, woodworking.
- **Two servers:** aquarium, beekeeping, gardening/plant care, home weather stations, open-source maintainer issue triage, astronomy data (exoplanet archive), FreeCAD.
- **Caveat:** registry absence ≠ nonexistence. Web search found community servers for Penpot (now **official**), darktable (official `darktable-mcp` build flag), RawTherapee (49 tools), GIMP, Matrix (four servers), Klipper (Kiln, OctoEverywhere, klipper-mcp) and a CLAP-embedding sample search (soundgrep). Each still has thin adoption.

---

## What the wider web adds to the HN picture

### dev.to: what practitioners are building and reading
- **Gateway and governance content dominates engagement.** Bifrost, enterprise MCP gateways and "how to run MCP in production" pieces each got 50–116 reactions. Content demand is high, but so is supply.
- **WebMCP / in-browser agents** are the most-discussed new topic: [webMCP live demo](https://dev.to/sylwia-lask/is-this-how-well-build-websites-soon-webmcp-live-demo) (157 reactions / 145 comments) and [Agent that never leaves the browser](https://dev.to/sylwia-lask/what-if-your-ai-agent-never-had-to-leave-the-browser-) (187 / 126). **Momentum signal:** WebMCP tooling (site-side tool declaration, testing, analytics) is early.
- **Tool-contract drift is a new research genre:**
  - ["4.4% of MCP servers changed their tool contract in 36 hours"](https://dev.to/theopslog/44-of-mcp-servers-changed-their-tool-contract-in-36-hours-i3m) (15 comments).
  - ["Pin your MCP server contracts the way you pin your dependencies"](https://dev.to/tsvetang2/pin-your-mcp-server-contracts-the-way-you-pin-your-dependencies-43j8).
  - ["I tested 31 MCP servers for contract compliance. Only 3% passed."](https://dev.to/tim860/i-tested-31-mcp-servers-for-contract-compliance-only-3-passed-25gp)
  - ["A fake MCP server spent three months earning trust"](https://dev.to/kielltampubolon/a-fake-mcp-server-spent-three-months-earning-trust-the-tells-were-there-192d).
  - ["Post-mortem: a trojan slipped into my MCP marketplace"](https://dev.to/edison_flores_6d2cd381b13/post-mortem-a-trojan-slipped-into-my-mcp-marketplace-heres-what-i-built-to-stop-it-happening-42oc).
  - Pinning tools exist (mcp-scan, RugSnare, MCP Sentinel, rug-pull-detector), but **nobody publishes a public, continuously-updated drift feed or "MCP changelog" across the ecosystem**.
- **On-device MCP:** ["MCP Just Landed on Your Phone: Google AI Edge Gallery"](https://dev.to/dannwaneri/mcp-just-landed-on-your-phone-what-google-ai-edge-gallery-actually-does-1567) (55 / 16) and its privacy-architecture follow-up (48 / 31). Mobile and on-device MCP is ramping.
- **Agent-payable public data:** ["I made Korean government data agent-payable with MCP and x402"](https://dev.to/wonderfulian/i-made-korean-government-data-agent-payable-with-mcp-and-x402-heres-what-actually-happened-40e8) and ["What Broke When We Turned Government Open Data Into Agent-Callable Evidence"](https://dev.to/civicdataforge/what-broke-when-we-turned-government-open-data-into-agent-callable-evidence-1m81). Gov-data MCPs are appearing, but the hard part is **evidence and citation integrity**, not access.
- **REA "Reverse Engineer Anything"** was reported at 115K GitHub stars in a week (Oct 2026). It confirms reverse engineering as the hottest expert niche.
- **Long tail of hobby and life builds:**
  - [FurEver Care: pet health timelines for the vet](https://dev.to/siti_aisyahmatzainal_73/furever-care-ai-turning-pet-health-observations-into-vet-ready-timelines-with-notion-ai-2n41) via Notion MCP.
  - Garmin/Strava/Cronometer/nutrition servers.
  - Raspberry Pi relays and I2C sensors over MCP.
  - "I Wrote an MCP Server for My 3D Printer".
  - "I gave Claude a PCB design tool over MCP" (Oct 24).
  - Apple Music (74 tools) and Sonos.
  - "earshot", which gives your coding agent ears (CPU-only).
  - Tour-guide MCP (~100 tools).
  - Brazilian economic data, Polish company registries (VAT/KRS/REGON) and SEC filings.

### Mastodon: the open-source and EU crowd
- **Vendors shipping first-party MCP endpoints:**
  - [Fastmail](https://mastodon.social/@fastmail) (54 favourites, the top MCP post).
  - Joplin 3.7.
  - Wikidata / Semantic MediaWiki (ProWiki).
  - HAL open-science archive (France).
  - MindNode, Luku (reading) and Obsidian Local REST API.
  - **Pattern:** open-standard-minded vendors treat MCP like IMAP/CalDAV, as just another open protocol.
- **Privacy pushback is strong, especially in German-language posts:**
  - Smart-home MCP raised the worry "first make everything local, then open my most intimate living space to an arbitrary LLM?"
  - BSI (the German federal security office) published MCP risk guidance.
  - **Momentum signal:** *local-LLM-first* MCP bundles for Home Assistant and personal data.
- **Linux / OS plumbing:** [MCP for systemd via varlink](https://github.com/mvo5/systemd-mcp) (14 / 10), plus kwin-mcp, gnome-ui-mcp and a Hyprland/Wayland computer-use MCP. **This is early: there's no integrated Linux-desktop agent layer yet.**
- **Maker evidence for text-to-CAD:** a FreeCAD user made a combination-wrench tray by giving AI ~100 measurements (18 / 7).
- **Security research tooling:** VulnMCP (CIRCL), the official Hex-Rays **IDA MCP**, SquatGuard (MCP/package namespace squatting) and Perplexity's open-sourced Bumblebee supply-chain scanner.
- **Library and science metadata:** lobid-GND (German library subject headings), HAL Discovery and libgen-mcp. **Niche:** cataloguers and librarians.

### X / Twitter (indexed posts only)
- **Real-iPhone control MCP:** an open-source demo from Oct 23 has early, sub-1k views. That's mobile momentum again.
- **Grok Bot marketplace** opened to connector submissions on Oct 14. It's a new distribution channel, and early submitters face little competition.
- **X's official XMCP server** was announced in June, and Elon Musk's post about it got 2.4M views.
- **TwelveLabs Jockey** video-intelligence agent went GA via MCP (Oct 15). Video-library MCPs are ramping.
- Mostly marketing or tutorial noise otherwise. X is a weak source without API access.

### Second-hand Reddit signals (via HN links)
- "I think I found a planet nobody knew existed. I used Claude Code" (r/ClaudeAI, 302 points / 132 comments on HN). **Citizen science with agents.** The registry has only 2 exoplanet MCPs and **no TESS/lightkurve light-curve MCP** (confirmed by web search).
- "Reddit user creates DB and MCP to mine Polygon, finds patterns on Polymarket" (r/ClaudeAI).
- "1.25K → 9.04K clicks in 60 days: pointed an agent to GSC every morning" (r/better_claw). Search-Console agents for SEO.
- A new **r/RunMCP** subreddit launched (Aug 2026), dedicated to MCP-driven apps.

---

## Verification log: HN "gaps" vs reality

| HN idea | Prior art found | Verdict |
|---|---|---|
| iOS Simulator MCP | ios-sim-mcp, ios-agent-driver, XC-MCP, xcode-studio-mcp, app-screen-mcp and others (16 in registry); real-iPhone MCP on X | **Crowded but fragmented.** No winner, and still "not helpful" per users. Opportunity: one excellent, maintained tool |
| FL Studio / Logic / Reaper | fl-mcp, flemcee, karl-andres FL MCP, Logic Pro MCP (35 tools), TwelveTake Reaper MCP, openDAW (262 tools), livepilot (317 tools) | **Built.** Gaps: Bitwig (0), **sample-library semantic search** (soundgrep only), **audio-analysis feedback loop** (a handful of LUFS tools) |
| Wireshark MCP | wireshark-mcp (51 tools), tshark-mcp, mcpcap, Glama remote-SSH variant | **Built, thin adoption.** HN asked in July 2026, so awareness is the gap |
| KiCad MCP | mixelpixx KiCAD-MCP-Server (up to 216 tools, JLCPCB + Freerouting), mcp-server-kicad, kicad-mcp-pro, lamaalrajih/kicad-mcp; "MCP for ordering assembled PCBs" (Show HN, Oct 19) | **Built and moving fast.** Gap: **ngspice / simulation-error recovery** and tying into KiCad |
| Rug-pull / tool-drift detection | mcp-scan (Snyk), RugSnare, MCP Sentinel, aiguard, MCPRadar, rug-pull-detector ×2 | **Crowded.** Remaining gap: **an ecosystem-wide public drift feed** |
| x402 paid MCP | Thousands of servers; Coinbase and Cloudflare reference implementations | **Oversupplied** |
| Human approval for consequential actions | Approval Gate (MCP App), Gatehouse, TrueFoundry (commercial) | **Early.** No mature open-source *reversibility classifier* ("signing looked like saving a draft") |
| Penpot / GIMP / darktable / RawTherapee | Official Penpot MCP; darktable-mcp build flag; RawTherapee MCP (49 tools); gimp-mcp | **Built recently.** Gap: a cross-app "Lightroom by prompt" photo workflow |
| Klipper / 3D printing | Kiln (800+ tools), OctoEverywhere MCP, klipper-mcp, bambu-printer-mcp | **Built.** Gap: integration with the text-to-CAD front end |
| Matrix | 4 community servers | Built, low adoption |
| Elder care | dala.care (vendor, first MCP home-care platform); student projects | **Mostly vendor.** No open, family-side caregiver coordination MCP |
| Woodworking CAD | ShopSpec (MCP beta), FluidCAD, claydash, Sawdust.diy, FreeCAD MCP | **Hot right now:** [Ask HN](https://news.ycombinator.com/item?id=50067325), 128 points (Oct 13). No consolidated winner |
| TESS / light-curve astronomy | astro_mcp, astroquery MCP, exoplanet-archive | **Gap confirmed:** no lightkurve/TESS MCP |

## Sources (web searches)
- KiCad:
  - [KiCAD-MCP-Server (mixelpixx)](https://glama.ai/mcp/servers/@mixelpixx/KiCAD-MCP-Server/blob/bce60eb0fcb8797530914d361e3120c6014db9d5/README.md)
  - [KiCad forum: open-source MCP server](https://forum.kicad.info/t/i-built-an-open-source-mcp-server-for-ai-assisted-kicad-workflows/68678)
  - [mcp-server-kicad](https://enterprisedna.co/directories/mcp/productofamerica-mcp-server-kicad)
- iOS Simulator:
  - [ios-sim-mcp](https://socket.dev/npm/package/ios-sim-mcp)
  - [XC-MCP](https://glama.ai/mcp/servers/@conorluddy/xc-mcp)
  - [iOS Simulator MCP Server](https://glama.ai/mcp/servers/AKAICH00/ios-simulator-mcp-server)
- Wireshark:
  - [wireshark-mcp](https://mcpservers.org/tr/servers/bx33661/wireshark-mcp)
  - [tshark-mcp](https://pypi.org/project/tshark-mcp/0.1.2/)
- DAWs:
  - [flemcee](https://pypi.org/project/flemcee/)
  - [FL Studio MCP Server](https://glama.ai/mcp/servers/@karl-andres/fl-studio-mcp/blob/05595015e7c825d83a6639a66e25abb5fac8b28d/README.md)
  - [Logic Pro MCP](https://glama.ai/mcp/servers/meo5uninsk)
- Tool pinning / rug-pull detection:
  - [mcp-scan](https://invariantlabs.ai/blog/introducing-mcp-scan)
  - [RugSnare](https://mcpservers.org/servers/paraphern/rugsnare)
  - [MCP Sentinel](https://glama.ai/mcp/servers/qmckda6hyq)
- x402 payments:
  - [systemprompt x402 guide](https://systemprompt.io/guides/monetize-mcp-server-x402)
  - [Eco x402 MCP](https://eco.com/support/en/articles/14846276-build-an-mcp-server-with-x402-monetization)
  - [AGNTCON session on paid tools](https://agntconmcpconjapan26.sched.com/event/2QlDO/what-happens-when-your-mcp-tools-cost-money-prakash-rao-aig-technologies-marco-gonzalez-red-hat)
- Penpot:
  - [Penpot MCP (official)](https://help.penpot.app/mcp/)
  - [Penpot MCP server page](https://penpot.app/penpot-mcp-server)
- 3D printing:
  - [Kiln 3D printing MCP](https://mcpservers.org/servers/codeofaxel/kiln)
  - [OctoEverywhere MCP](https://blog.octoeverywhere.com/mcp-server-for-3d-printing/)
- Matrix: [Matrix MCP servers](https://glama.ai/mcp/servers/Baho73/mcp-server-matrix)
- Photo editing:
  - [darktable-mcp docs](https://docs.darktable.org/usermanual/development/fr/special-topics/program-invocation/darktable-mcp/)
  - [gimp-mcp](https://glama.ai/mcp/servers/pflerman/gimp-mcp)
- Astronomy: [astro_mcp](https://glama.ai/mcp/servers/@SandyYuan/astro_mcp)
- Sample search:
  - [soundgrep / sample search](https://glama.ai/mcp/servers/ly7odkdrsu)
  - [Freesound MCP](https://glama.ai/mcp/servers/@timjrobinson/FreesoundMCPServer/blob/77c486ae44963224ef3ea940009fa982634eeae9/README.md)
- Approval gates: [TrueFoundry tool approvals](https://www.truefoundry.com/blog/mcp-tool-approval-human-gate-call-path)
- Elder care: [dala.care MCP home care](https://dala.care/blog/dala.care-the-first-mcp-enabled-home-care-platform)
- Woodworking:
  - [ShopSpec](https://alternativeto.net/software/shopspec/about/)
  - [Ask HN CAD for woodworking](https://zeli.app/story/50067325)
- X posts:
  - [Real-iPhone MCP demo on X](https://x.com/SemsTurgut/status/2113628142481150273)
  - [Grok Bot marketplace on X](https://x.com/Chaosai0/status/2110619052427337801)
  - [TwelveLabs Jockey on X](https://x.com/lmoroney/status/2111321594400223727)
- Reddit API status: [Reddit MCP status 2026](https://www.sprites.ai/blog/reddit-mcp)
- MCP gap analyses:
  - [MCP 2026 gap analysis (dev.to)](https://dev.to/ai-agent-economy/mcp-2026-gap-analysis-what-it-doesnt-include-and-why-thats-our-opportunity-43f)
  - [State of MCP servers 2026](https://tooldirectory.ai/blog/state-of-mcp-servers-2026)
- Homelab MCPs:
  - [homelab-mcp (HavartiBard)](https://glama.ai/mcp/servers/@HavartiBard/homelab-mcp/blob/012fd30d0319d6ebd9dc466930216dccda301b7b/README.md)
  - [homelab-mcp (bjeans)](https://glama.ai/mcp/servers/bjeans/homelab-mcp)
