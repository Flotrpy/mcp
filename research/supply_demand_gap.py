"""Demand-vs-supply gap table per MCP niche.

Inputs (filtered from the collectors' output, see Method in research/mcp-ideas-web.md):
  registry_latest.json  official MCP Registry, version=latest
  mcp_stories.json      HN Algolia stories mentioning MCP
  devto_mcp.json        dev.to MCP articles
  masto_mcp.json        Mastodon MCP posts (with _text)
Output: research/data/niche_supply_demand.csv
"""
import json,re
R=json.load(open("registry_latest.json"))
H=json.load(open("mcp_stories.json")); D=json.load(open("devto_mcp.json")); M=json.load(open("masto_mcp.json"))
reg=[(s["server"].get("name","")+" "+(s["server"].get("title") or "")+" "+(s["server"].get("description") or "")) for s in R]
N={
"wearables/health data":r"garmin|oura|whoop|strava|apple health|healthkit|fitbit|polar\b|wearable",
"medical/clinical":r"fhir|hl7|clinical|ehr\b|patient|icd-?10|snomed|pubmed|drug",
"nutrition/food":r"nutrition|calorie|cronometer|recipe|meal|food",
"ios simulator/xcode":r"ios simulator|xcode|simctl|swiftui",
"android/adb":r"android|\badb\b|emulator",
"music daw":r"ableton|fl studio|logic pro|reaper|\bdaw\b|bitwig|cubase|pro tools",
"audio analysis/samples":r"audio analysis|loudness|lufs|sample library|stem|spectrogram|mastering",
"spotify/sonos/music svc":r"spotify|sonos|apple music|last\.fm|soundcloud",
"kicad/pcb/eda":r"kicad|pcb|schematic|altium|eda\b|gerber",
"spice/lab instruments":r"spice|ngspice|oscilloscope|scpi|labgrid|multimeter|instrument",
"cad/3d parametric":r"\bcad\b|freecad|openscad|build123d|cadquery|onshape|fusion 360|solidworks",
"3d printing":r"3d print|octoprint|klipper|bambu|slicer|prusa|gcode",
"blender/3d art":r"blender|maya|houdini|cinema 4d|unreal|godot|unity",
"home assistant/iot":r"home assistant|homeassistant|zigbee|mqtt|smart home|matter\b",
"embedded/arduino":r"arduino|esp32|esp-idf|platformio|raspberry|stm32|microcontroller|firmware",
"robotics/ros/drones":r"\bros2?\b|robot|drone|px4|ardupilot|mavlink",
"wireshark/pcap":r"wireshark|tshark|pcap|packet capture",
"reverse engineering":r"ghidra|\bida\b|radare|binary ninja|frida|decompil|reverse engineer",
"pentest/offsec":r"pentest|nmap|burp|metasploit|nuclei|bug bounty|ctf\b",
"vuln/cve intel":r"\bcve\b|vulnerab|nvd\b|exploit-?db|osv\b",
"mcp security scanner/pin":r"rug.?pull|tool poisoning|mcp.?scan|pinning|tool hash",
"personal finance/budget":r"budget|personal finance|ynab|plaid|simplefin|actual budget|firefly|ledger|beancount",
"accounting/invoice/tax":r"quickbooks|xero|invoice|accounting|tax|bookkeep",
"trading/markets":r"stock|trading|broker|alpaca|polymarket|kalshi|options|forex",
"crypto/defi":r"crypto|defi|ethereum|solana|wallet|blockchain|onchain",
"agent payments/x402":r"x402|pay.?per.?call|micropayment|agent payment|stablecoin|usdc|l402|lightning",
"oauth/auth proxy":r"oauth|dynamic client registration|\bdcr\b|auth proxy|token broker",
"cors/compat proxy":r"cors|compatib|stdio.?to.?http|bridge|proxy",
"file transfer":r"file transfer|upload|download|s3\b|blob|attachment",
"notifications/push/webhook":r"push notif|webhook|notification|ntfy|pushover|alert",
"service status/outage":r"status ?page|outage|uptime|downdetector|incident",
"composition/code mode":r"code mode|compose|pipeline|orchestrat|chain tools|script",
"mcp testing/lint/grade":r"mcp (test|lint|eval|inspector|validator)|compliance|conformance|grade",
"spec migration/stateless":r"stateless|2026-07-28|migrat",
"life tracking/pets/home inv":r"\bpet\b|\bdog\b|plant|home inventory|household|habit|journal|diary|life ?log",
"creative/photo editing":r"photoshop|lightroom|gimp|krita|darktable|inkscape|penpot|affinity|canva",
"figma/design":r"figma|penpot|sketch\b|framer",
"games/agent arenas":r"\bgame|chess|minecraft|poker|arena|simcity|roguelike",
"gov/public data":r"government|census|cadastre|land registry|court|legislat|procurement|open data|parliament|eurostat|statistic",
"legal":r"\blegal|\blaw\b|contract|case law|statute|patent",
"linux desktop/dbus":r"d-?bus|gnome|kde|at-spi|wayland|systemd|varlink|linux desktop",
"macos/windows desktop":r"applescript|macos|accessibility api|windows ui|uia\b|powershell|automator",
"astronomy/space":r"astronom|telescope|nasa|exoplanet|tess\b|kepler|satellite|space weather",
"geo/gis/maps":r"\bgis\b|qgis|openstreetmap|osm\b|geocod|arcgis|mapbox|google maps",
"weather/climate":r"weather|forecast|climate|noaa|meteo",
"travel/flights":r"flight|airline|hotel|travel|booking\.com|airbnb|itinerar",
"education/tutoring":r"tutor|flashcard|anki|lesson|curriculum|student|quiz|course",
"library/books/reading":r"zotero|calibre|kindle|epub|library|goodreads|isbn|citation",
"science/math/proofs":r"lean\b|coq|theorem|wolfram|sympy|mathematica|arxiv|latex",
"messaging whatsapp/imessage/sms":r"whatsapp|imessage|\bsms\b|signal|telegram",
"email":r"\bemail|gmail|imap|fastmail|outlook|jmap",
"calendar/scheduling":r"calendar|caldav|scheduling|cal\.com|calendly",
"notes/pkm":r"obsidian|notion|logseq|joplin|roam|evernote|bear\b|apple notes",
"crm/sales":r"\bcrm\b|salesforce|hubspot|pipedrive|lead gen",
"ecommerce":r"shopify|woocommerce|ecommerce|e-commerce|amazon seller|etsy",
"seo/ads":r"\bseo\b|search console|google ads|meta ads|adwords",
"devops k8s/terraform":r"kubernetes|k8s|terraform|helm|ansible|pulumi|proxmox",
"observability/logs":r"datadog|grafana|prometheus|sentry|loki|opentelemetry|logs?\b",
"databases":r"postgres|mysql|sqlite|mongodb|redis|duckdb|clickhouse|snowflake|bigquery",
"browser automation":r"browser|playwright|puppeteer|selenium|chrome devtools",
"memory/knowledge graph":r"memory|knowledge graph|second brain|rag\b|vector",
"voice/telephony":r"voice|telephony|twilio|phone call|speech|tts\b|transcrib",
"video/youtube":r"youtube|video edit|ffmpeg|premiere|davinci|transcript",
"accessibility a11y":r"a11y|accessibility audit|wcag|screen reader",
"bible/religion":r"bible|quran|torah|scripture",
"agriculture/farming":r"agricultur|farm|crop|soil|livestock",
"real estate/property":r"real estate|property|zillow|mortgage|rent\b",
"hr/recruiting/jobs":r"recruit|resume|\bats\b|job (board|search)|hiring|linkedin",
"supply chain/logistics":r"logistic|shipping|freight|warehouse|inventory|supply chain|fedex|ups\b",
"manufacturing/plc/scada":r"\bplc\b|scada|modbus|opc.?ua|industrial|manufactur|cnc",
"energy/solar/ev":r"solar|inverter|\bev\b|charger|energy|grid|battery|tesla",
"automotive/obd":r"obd|can ?bus|automotive|vehicle|car\b",
"ham radio/sdr":r"ham radio|\bsdr\b|aprs|meshtastic|lora|radio",
"genomics/bio":r"genom|protein|bioinformatic|dna|rna|crispr|pdb\b|uniprot",
"chemistry/materials":r"chemistr|molecul|rdkit|pubchem|material science",
}
rx={k:re.compile(v,re.I) for k,v in N.items()}
rows=[]
for k,r in rx.items():
    sup=sum(1 for t in reg if r.search(t))
    hs=[s for s in H if r.search((s.get("title") or "")+" "+(s.get("url") or ""))]
    ds=[a for a in D if r.search(a["title"]+" "+(a.get("description") or ""))]
    ms=[p for p in M if r.search(p["_text"])]
    hp=sum((s.get("points") or 0)+(s.get("num_comments") or 0) for s in hs)
    dp=sum(a["public_reactions_count"]+a["comments_count"] for a in ds)
    late=sum(1 for s in hs if s["created_at"]>="2026-06")+sum(1 for a in ds if a["published_at"]>="2026-06")
    early=sum(1 for s in hs if s["created_at"]<"2026-06")+sum(1 for a in ds if a["published_at"]<"2026-06")
    demand=hp+dp+len(ms)
    rows.append((k,sup,len(hs),hp,len(ds),dp,len(ms),demand,round(demand/(sup+5),2),round(late/(early+1)*0.8,2)))
rows.sort(key=lambda r:-r[8])
print(f"{'niche':32}{'supply':>7}{'hnN':>5}{'hnEng':>6}{'devN':>5}{'devEng':>7}{'masto':>6}{'demand':>7}{'D/S':>6}{'growth':>7}")
for r in rows: print(f"{r[0][:32]:32}{r[1]:>7}{r[2]:>5}{r[3]:>6}{r[4]:>5}{r[5]:>7}{r[6]:>6}{r[7]:>7}{r[8]:>6}{r[9]:>7}")
json.dump(rows,open("gap_rows.json","w"))
