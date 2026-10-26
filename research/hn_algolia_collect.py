#!/usr/bin/env python3
"""Collect MCP-related Hacker News stories and comments from the last N days via the
HN Algolia search API, slicing time into windows so no query hits the 1000-result cap.

    python3 research/hn_algolia_collect.py --days 273 --out research/data/hn_algolia.json
"""
import argparse
import json
import time
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor

API = "https://hn.algolia.com/api/v1/search_by_date"

STORY_QUERIES = [
    "mcp", "model context protocol", "mcp server", "mcp servers", "mcp client",
    "mcp gateway", "mcp proxy", "mcp security", "mcp registry", "mcp tool",
    "mcp tools", "mcp apps", "mcp ui", "remote mcp", "mcp oauth", "mcp auth",
    "agent protocol", "a2a protocol", "tool calling", "agent skills", "claude skills",
    "claude code", "codex cli", "agentic", "ai agent tools", "computer use",
]
COMMENT_QUERIES = [
    "mcp server", "model context protocol", "mcp", "someone should build mcp",
    "wish there was an mcp", "mcp for", "would pay mcp", "mcp missing",
]


def get(params):
    url = API + "?" + urllib.parse.urlencode(params)
    for attempt in range(5):
        try:
            with urllib.request.urlopen(url, timeout=30) as r:
                return json.load(r)
        except Exception:
            time.sleep(2 ** attempt)
    return {"hits": [], "nbPages": 0}


def collect(query, tags, start, end, window):
    hits = []
    t = start
    while t < end:
        w_end = min(t + window, end)
        page = 0
        while True:
            data = get({
                "query": query, "tags": tags, "hitsPerPage": 1000, "page": page,
                "numericFilters": f"created_at_i>={t},created_at_i<{w_end}",
            })
            hits.extend(data.get("hits", []))
            page += 1
            if page >= data.get("nbPages", 0):
                break
        t = w_end
    return query, tags, hits


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--days", type=int, default=273)
    ap.add_argument("--out", default="research/data/hn_algolia.json")
    args = ap.parse_args()
    end = int(time.time())
    start = end - args.days * 86400

    jobs = [(q, "story", start, end, 7 * 86400) for q in STORY_QUERIES]
    jobs += [(q, "comment", start, end, 2 * 86400) for q in COMMENT_QUERIES]

    stories, comments = {}, {}
    with ThreadPoolExecutor(max_workers=8) as ex:
        for query, tags, hits in ex.map(lambda j: collect(*j), jobs):
            bucket = stories if tags == "story" else comments
            for h in hits:
                rec = bucket.setdefault(h["objectID"], h)
                rec.setdefault("_queries", [])
                if query not in rec["_queries"]:
                    rec["_queries"].append(query)
            print(f"{tags:7} {query!r}: {len(hits)} hits", flush=True)

    with open(args.out, "w") as f:
        json.dump({"start": start, "end": end, "stories": list(stories.values()),
                   "comments": list(comments.values())}, f)
    print(f"{len(stories)} stories, {len(comments)} comments -> {args.out}")


if __name__ == "__main__":
    main()
