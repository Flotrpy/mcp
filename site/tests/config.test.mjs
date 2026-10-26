import { test } from "node:test";
import assert from "node:assert/strict";
import { clientConfig, installCommand } from "../js/config.js";

const opts = { binscopeRoot: "/Users/jo/My Builds", vitalsRedact: true };

test("install command lists the chosen crates", () => {
  assert.equal(installCommand(["joinery", "vitals"]), "cargo install --git https://github.com/Flotrpy/mcp joinery vitals");
  assert.equal(installCommand([]), "");
});

test("JSON clients get mcpServers with env only where needed", () => {
  const cfg = JSON.parse(clientConfig("cursor", ["joinery", "binscope", "vitals"], opts));
  assert.deepEqual(cfg.mcpServers.joinery, { command: "joinery-mcp" });
  assert.equal(cfg.mcpServers.binscope.env.BINSCOPE_ROOT, "/Users/jo/My Builds");
  assert.equal(cfg.mcpServers.vitals.env.VITALS_REDACT, "1");
});

test("Claude Code commands quote paths with spaces", () => {
  const cmd = clientConfig("claude-code", ["binscope"], opts);
  assert.equal(cmd, "claude mcp add binscope -e BINSCOPE_ROOT='/Users/jo/My Builds' -- binscope-mcp");
});

test("Codex gets TOML tables", () => {
  const toml = clientConfig("codex", ["vitals"], opts);
  assert.equal(toml, '[mcp_servers.vitals]\ncommand = "vitals-mcp"\nenv = { VITALS_REDACT = "1" }');
});
