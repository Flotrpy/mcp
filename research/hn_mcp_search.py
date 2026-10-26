#!/usr/bin/env python3
"""Search Hacker News for stories about MCP, Claude, Codex, AI tooling and open source.

Uses the public HN Firebase API (no key required). Standard library only.

Examples:
    python3 research/hn_mcp_search.py                       # scan top 200 stories
    python3 research/hn_mcp_search.py --feed new --limit 300
    python3 research/hn_mcp_search.py --feed top --feed new --comments --json out.json
    python3 research/hn_mcp_search.py --extra "langgraph" --extra "a2a"
"""
import argparse
import json
import re
import sys
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone

BASE = "https://hacker-news.firebaseio.com/v0"
FEEDS = {"top": "topstories", "new": "newstories", "best": "beststories"}

# Topic -> list of patterns. Matched case-insensitively with word boundaries.
TOPICS = {
    "mcp": [
        "mcp", "model context protocol", "mcp server", "mcp client",
        "context protocol", "tool protocol", "agent protocol", "a2a",
        "agent2agent", "agents.md",
    ],
    "claude": [
        "claude", "claude code", "anthropic", "claude desktop", "claude skills",
        "sonnet", "opus", "haiku", "agent sdk",
    ],
    "codex": [
        "codex", "codex cli", "openai codex", "gpt-5", "chatgpt agent",
        "gemini cli", "copilot", "cursor", "aider", "cline", "opencode",
        "coding agent", "ai coding", "vibe coding",
    ],
    "tools": [
        "tool use", "tool calling", "function calling", "tool call",
        "agentic", "ai agent", "agent framework", "plugin", "extension",
        "skills", "sdk", "langchain", "langgraph", "llm tool",
    ],
    "open-source": [
        "open source", "open-source", "oss", "github.com", "self-hosted",
        "apache 2.0", "mit license", "foss",
    ],
}


def compile_patterns(extra):
    topics = {k: list(v) for k, v in TOPICS.items()}
    if extra:
        topics["extra"] = list(extra)
    compiled = {}
    for topic, words in topics.items():
        alt = "|".join(re.escape(w) for w in sorted(set(words), key=len, reverse=True))
        compiled[topic] = re.compile(rf"(?<![\w]){alt}(?![\w])", re.I)
    return compiled


def get_json(path, timeout=15):
    req = urllib.request.Request(f"{BASE}/{path}.json", headers={"User-Agent": "hn-mcp-search/1.0"})
    for attempt in range(3):
        try:
            with urllib.request.urlopen(req, timeout=timeout) as r:
                return json.load(r)
        except Exception:
            if attempt == 2:
                return None


def fetch_items(ids, workers):
    with ThreadPoolExecutor(max_workers=workers) as ex:
        return [i for i in ex.map(lambda i: get_json(f"item/{i}"), ids) if i]


def fetch_comments(item, workers, max_depth, depth=0):
    """Recursively rebuild the comment tree via the 'kids' arrays."""
    kids = item.get("kids") or []
    if not kids or depth >= max_depth:
        return []
    out = []
    for c in fetch_items(kids, workers):
        if c.get("deleted") or c.get("dead"):
            continue
        c["children"] = fetch_comments(c, workers, max_depth, depth + 1)
        out.append(c)
    return out


def flatten_text(comments):
    parts = []
    for c in comments:
        parts.append(c.get("text") or "")
        parts.append(flatten_text(c.get("children", [])))
    return " ".join(parts)


def match_topics(text, patterns):
    hits = {}
    for topic, rx in patterns.items():
        found = sorted({m.group(0).lower() for m in rx.finditer(text)})
        if found:
            hits[topic] = found
    return hits


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--feed", action="append", choices=FEEDS, help="top/new/best (repeatable, default: top)")
    ap.add_argument("--limit", type=int, default=200, help="stories to scan per feed (default 200)")
    ap.add_argument("--extra", action="append", default=[], help="extra keyword (repeatable)")
    ap.add_argument("--min-score", type=int, default=0)
    ap.add_argument("--min-topics", type=int, default=1,
                    help="require at least N distinct topic groups to match (default 1)")
    ap.add_argument("--comments", action="store_true", help="also search comment text (slower)")
    ap.add_argument("--depth", type=int, default=2, help="comment tree depth when --comments (default 2)")
    ap.add_argument("--workers", type=int, default=16)
    ap.add_argument("--json", metavar="FILE", help="write full results (incl. comment trees) to FILE")
    args = ap.parse_args()

    patterns = compile_patterns(args.extra)
    ids = []
    for feed in args.feed or ["top"]:
        feed_ids = get_json(FEEDS[feed])
        if feed_ids is None:
            sys.exit(f"Could not fetch {feed} stories from {BASE}")
        ids.extend(feed_ids[: args.limit])
    ids = list(dict.fromkeys(ids))
    print(f"Scanning {len(ids)} stories...", file=sys.stderr)

    results = []
    for story in fetch_items(ids, args.workers):
        if story.get("type") != "story" or story.get("deleted") or story.get("dead"):
            continue
        if story.get("score", 0) < args.min_score:
            continue
        text = " ".join([story.get("title", ""), story.get("url", ""), story.get("text") or ""])
        hits = match_topics(text, patterns)
        tree = []
        if args.comments and story.get("kids"):
            tree = fetch_comments(story, args.workers, args.depth)
            for t, w in match_topics(flatten_text(tree), patterns).items():
                hits[t] = sorted(set(hits.get(t, [])) | set(w))
        if len(hits) >= args.min_topics:
            results.append({
                "id": story["id"], "title": story.get("title"), "url": story.get("url"),
                "score": story.get("score", 0), "comments": story.get("descendants", 0),
                "time": datetime.fromtimestamp(story.get("time", 0), timezone.utc).isoformat(),
                "hn_link": f"https://news.ycombinator.com/item?id={story['id']}",
                "matches": hits, "comment_tree": tree,
            })

    results.sort(key=lambda r: (len(r["matches"]), r["score"]), reverse=True)
    if not results:
        print("No matching stories found.")
    for r in results:
        print(f"[{r['score']:>4}] {r['title']}")
        print(f"       {r['url'] or r['hn_link']}")
        print(f"       HN: {r['hn_link']}  ({r['comments']} comments)")
        print("       topics: " + "; ".join(f"{t}={','.join(w)}" for t, w in r["matches"].items()))
    print(f"\n{len(results)} matching stories.", file=sys.stderr)

    if args.json:
        with open(args.json, "w") as f:
            json.dump(results, f, indent=2)
        print(f"Wrote {args.json}", file=sys.stderr)


if __name__ == "__main__":
    main()
