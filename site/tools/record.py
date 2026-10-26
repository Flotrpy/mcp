#!/usr/bin/env python3
"""Record real MCP exchanges from the built servers into site/data/transcripts.json.

The website replays these, so every example on it is genuine output.

    cargo build --release --workspace
    python3 site/tools/record.py            # uses target/release
"""
import json
import os
import subprocess
import sys
import tempfile
import time

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BIN = os.path.join(ROOT, "target", os.environ.get("PROFILE", "release"))
META = {
    "io.modelcontextprotocol/protocolVersion": "2026-07-28",
    "io.modelcontextprotocol/clientInfo": {"name": "site-recorder", "version": "1"},
    "io.modelcontextprotocol/clientCapabilities": {},
}


def session(server, calls, env=None, pause=0.6):
    """Run tool calls against one server process; return [(args, text)]."""
    proc = subprocess.Popen(
        [os.path.join(BIN, f"{server}-mcp")],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
        text=True, env={**os.environ, **(env or {})},
    )
    for i, (tool, args) in enumerate(calls, 1):
        req = {"jsonrpc": "2.0", "id": i, "method": "tools/call",
               "params": {"name": tool, "arguments": args, "_meta": META}}
        proc.stdin.write(json.dumps(req) + "\n")
        proc.stdin.flush()
        time.sleep(pause)
    out, _ = proc.communicate(timeout=60)
    replies = {}
    for line in out.splitlines():
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        if "id" in msg:
            replies[msg["id"]] = msg.get("result", {}).get("content", [{}])[0].get("text", "")
    return [{"tool": t, "args": a, "reply": replies.get(i, "")} for i, (t, a) in enumerate(calls, 1)]


def main():
    tmp = tempfile.mkdtemp(prefix="mcp-site-")
    fixtures = os.path.join(ROOT, "crates", "handset", "tests", "fixtures")
    transcripts = {
        "joinery": session("joinery", [
            ("design_furniture", {"kind": "bookcase", "width": 36, "height": 72, "depth": 12,
                                  "stock_thickness": 0.75, "adjustable_shelves": 4, "units": "in", "species": "mdf"}),
            ("check_shelf_sag", {"span": 900, "depth": 280, "thickness": 19, "species": "white-oak", "load_preset": "books"}),
        ]),
        "binscope": session("binscope", [
            ("overview", {"path": os.path.join(BIN, "joinery-mcp")}),
            ("diff", {"old_path": os.path.join(BIN, "joinery-mcp"), "new_path": os.path.join(BIN, "binscope-mcp")}),
        ]),
        "lighthouse": session("lighthouse", [
            ("audit_server", {"command": "sh", "args": [os.path.join(ROOT, "crates/lighthouse/tests/fixtures/legacy_server.sh")]}),
            ("audit_server", {"command": os.path.join(BIN, "joinery-mcp")}),
        ], pause=1.5),
        "vitals": session("vitals", [
            ("import_health_data", {"path": os.path.join(ROOT, "examples", "vitals-demo.csv")}),
            ("daily_summary", {"metrics": ["steps"], "from": "2026-10-24"}),
            ("readiness", {}),
        ], env={"VITALS_DB": os.path.join(tmp, "vitals.sqlite")}),
        "handset": session("handset", [
            ("snapshot", {}),
            ("act", {"action": "tap", "ref": "e3"}),
        ], env={
            "PATH": os.path.join(fixtures, "bin") + os.pathsep + os.environ.get("PATH", ""),
            "HANDSET_LOG": os.path.join(tmp, "adb.log"),
            "HANDSET_SCREEN": os.path.join(fixtures, "login.xml"),
            "HANDSET_NEXT_SCREEN": os.path.join(fixtures, "inbox.xml"),
        }),
    }
    # Paths on the recording machine are noise on a website.
    text = json.dumps(transcripts, indent=1, ensure_ascii=False).replace(ROOT + "/", "./")
    out = os.path.join(ROOT, "site", "data", "transcripts.json")
    with open(out, "w") as f:
        f.write(text + "\n")
    empty = [s for s, calls in transcripts.items() for c in calls if not c["reply"]]
    print(f"wrote {out}" + (f" (empty replies from: {', '.join(empty)})" if empty else ""))
    return 1 if empty else 0


if __name__ == "__main__":
    sys.exit(main())
