#!/bin/sh
# A deliberately imperfect MCP server for lighthouse's tests:
# - speaks only the 2025-11-25 handshake (no server/discover),
# - logs a line to stdout (a classic bug),
# - takes its single tool's description from $FAKE_DESC so tests can
#   simulate drift between pin and verify.
echo "fake server starting"
desc="${FAKE_DESC:-Echo the given text back unchanged.}"
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  [ -z "$id" ] && continue
  case "$line" in
    *'"method":"initialize"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"fake","version":"0.0.1"}}}\n' "$id" ;;
    *'"method":"tools/list"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"tools":[{"name":"echo","description":"%s","inputSchema":{"type":"object","properties":{"text":{"type":"string"}},"required":["text"]}}]}}\n' "$id" "$desc" ;;
    *)
      printf '{"jsonrpc":"2.0","id":%s,"error":{"code":-32601,"message":"Method not found"}}\n' "$id" ;;
  esac
done
