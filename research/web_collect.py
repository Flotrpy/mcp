#!/usr/bin/env python3
"""Collect MCP-related posts from dev.to, Mastodon (several instances), Lobsters and the
official MCP Registry for the last N days. Standard library only.

    python3 research/web_collect.py --days 273 --out research/data/web.json
"""
import argparse
import json
import time
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone

UA = {"User-Agent": "Mozilla/5.0 (mcp-research script)"}

DEVTO_TAGS = ["mcp", "modelcontextprotocol", "mcpserver", "claude", "claudecode",
              "aiagents", "agents", "agenticai", "codex", "llm", "ai", "opensource"]
MASTODON = ["mastodon.social", "fosstodon.org", "hachyderm.io", "infosec.exchange",
            "mas.to", "techhub.social", "floss.social"]
MASTO_TAGS = ["mcp", "modelcontextprotocol", "mcpserver", "claudecode", "agenticai"]
LOBSTERS_Q = ["mcp", "model context protocol", "mcp server"]


def get(url, retries=4):
    for a in range(retries):
        try:
            with urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=30) as r:
                return json.load(r), r.headers
        except Exception:
            time.sleep(2 ** a)
    return None, None


def ts(s):
    return datetime.fromisoformat(s.replace("Z", "+00:00")).timestamp()


def devto(tag, start, days):
    out, page = [], 1
    while page <= 30:
        data, _ = get(f"https://dev.to/api/articles?tag={tag}&top={days}&per_page=1000&page={page}")
        if not data:
            break
        out += [a for a in data if ts(a["published_at"]) >= start]
        if len(data) < 1000:
            break
        page += 1
    return "devto", tag, out


def mastodon(host, tag, start):
    out, max_id = [], None
    for _ in range(60):
        q = f"https://{host}/api/v1/timelines/tag/{tag}?limit=40" + (f"&max_id={max_id}" if max_id else "")
        data, _ = get(q)
        if not data:
            break
        out += [p for p in data if ts(p["created_at"]) >= start]
        if ts(data[-1]["created_at"]) < start:
            break
        max_id = data[-1]["id"]
    for p in out:
        p["_instance"] = host
    return "mastodon", f"{host}#{tag}", out


def lobsters(q, start):
    out = []
    for page in range(1, 11):
        data, _ = get("https://lobste.rs/search.json?" + urllib.parse.urlencode(
            {"q": q, "what": "stories", "order": "newest", "page": page}))
        if not data:
            break
        out += [s for s in data if ts(s["created_at"]) >= start]
        if ts(data[-1]["created_at"]) < start:
            break
    return "lobsters", q, out


def registry():
    out, cursor = [], None
    for _ in range(500):
        url = "https://registry.modelcontextprotocol.io/v0/servers?limit=100" + (
            f"&cursor={urllib.parse.quote(cursor)}" if cursor else "")
        data, _ = get(url)
        if not data:
            break
        out += data.get("servers", [])
        cursor = (data.get("metadata") or {}).get("nextCursor") or (data.get("metadata") or {}).get("next_cursor")
        if not cursor:
            break
    return "registry", "all", out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--days", type=int, default=273)
    ap.add_argument("--out", default="research/data/web.json")
    args = ap.parse_args()
    start = time.time() - args.days * 86400

    jobs = [lambda t=t: devto(t, start, args.days) for t in DEVTO_TAGS]
    jobs += [lambda h=h, t=t: mastodon(h, t, start) for h in MASTODON for t in MASTO_TAGS]
    jobs += [lambda q=q: lobsters(q, start) for q in LOBSTERS_Q]
    jobs += [registry]

    result = {"devto": {}, "mastodon": {}, "lobsters": {}, "registry": []}
    with ThreadPoolExecutor(max_workers=10) as ex:
        for src, key, items in ex.map(lambda f: f(), jobs):
            print(f"{src:9} {key:40} {len(items)}", flush=True)
            if src == "registry":
                result["registry"] = items
                continue
            for it in items:
                result[src].setdefault(str(it.get("id") or it.get("short_id")), it)
    for k in ("devto", "mastodon", "lobsters"):
        result[k] = list(result[k].values())
    with open(args.out, "w") as f:
        json.dump(result, f)
    print({k: len(v) for k, v in result.items()})


if __name__ == "__main__":
    main()
