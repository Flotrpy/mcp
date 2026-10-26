#!/usr/bin/env python3
"""Record real MCP exchanges from the built servers into site/data/transcripts.json,
and audit every server with lighthouse into site/data/dashboard.json.

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


SERVERS = ["joinery", "binscope", "lighthouse", "vitals", "handset"]


def dashboard(tmp):
    """Audit every server with lighthouse and check its committed mcp.lock."""
    lh = os.path.join(BIN, "lighthouse-mcp")
    env = {**os.environ, "VITALS_DB": os.path.join(tmp, "dash.sqlite")}
    rows = []
    for s in SERVERS:
        exe = os.path.join(BIN, f"{s}-mcp")
        audit = json.loads(subprocess.run([lh, "audit", "--json", "--", exe], capture_output=True, text=True, env=env, timeout=120).stdout)
        verify = subprocess.run([lh, "verify", "--lock", os.path.join(ROOT, "locks", f"{s}.lock"), "--", exe],
                                capture_output=True, text=True, env=env, timeout=120)
        rows.append({
            "server": s,
            "version": audit.get("server_version"),
            "grade": audit["score"]["grade"],
            "points": audit["score"]["points"],
            "deductions": audit["score"]["deductions"],
            "migration": audit["migration"],
            "handshake_ms": {"stateless": audit["stateless"].get("handshake_ms"), "legacy": audit["legacy"].get("handshake_ms")},
            "tools": [{"tool": t["tool"], "tokens": t["tokens"]} for t in audit["cost"]["tools"]],
            "total_tokens": audit["cost"]["total_tokens"],
            "findings": len(audit["findings"]),
            "lock": {"clean": verify.returncode == 0, "summary": verify.stdout.strip().splitlines()[0] if verify.stdout else ""},
            "binary_bytes": os.path.getsize(exe),
        })
    return {"generated": time.strftime("%Y-%m-%d"), "servers": rows}


def cut_diagram():
    """The SVG joinery draws for the transcript's bookcase, for the joinery tile."""
    parts = [
        {"name": "side", "length": 1829, "width": 305, "quantity": 2},
        {"name": "top", "length": 889, "width": 298, "quantity": 1},
        {"name": "bottom", "length": 889, "width": 298, "quantity": 1},
        {"name": "shelf", "length": 874, "width": 298, "quantity": 4},
        {"name": "back", "length": 1829, "width": 895, "quantity": 1, "grain_locked": False},
    ]
    proc = subprocess.run(
        [os.path.join(BIN, "joinery-mcp")],
        input=json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {
            "name": "nest_parts", "arguments": {"parts": parts}, "_meta": META}}) + "\n",
        capture_output=True, text=True, timeout=30,
    )
    result = json.loads(proc.stdout.splitlines()[0])["result"]
    return result["structuredContent"]["svg"], result["content"][0]["text"]


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
    svg, caption = cut_diagram()
    with open(os.path.join(ROOT, "site", "data", "joinery-layout.svg"), "w") as f:
        f.write(svg + "\n")
    print("cut diagram:", caption)
    dash = dashboard(tmp)
    with open(os.path.join(ROOT, "site", "data", "dashboard.json"), "w") as f:
        f.write(json.dumps(dash, indent=1) + "\n")
    empty = [s for s, calls in transcripts.items() for c in calls if not c["reply"]]
    print(f"wrote {out}" + (f" (empty replies from: {', '.join(empty)})" if empty else ""))
    return 1 if empty else 0


if __name__ == "__main__":
    sys.exit(main())
