// Pure config generators for the install builder (tested in tests/config.test.mjs).

export const SERVERS = ["joinery", "binscope", "lighthouse", "vitals", "handset"];

export const CLIENTS = {
  "claude-code": { label: "Claude Code", file: "Run in your terminal" },
  "claude-desktop": { label: "Claude Desktop", file: "claude_desktop_config.json" },
  cursor: { label: "Cursor", file: ".cursor/mcp.json" },
  codex: { label: "Codex", file: "~/.codex/config.toml" },
};

/** Environment variables a server needs, from the builder's options. */
export function envFor(server, opts) {
  const env = {};
  if (server === "binscope" && opts.binscopeRoot) env.BINSCOPE_ROOT = opts.binscopeRoot;
  if (server === "vitals" && opts.vitalsRedact) env.VITALS_REDACT = "1";
  return env;
}

export function installCommand(servers) {
  if (!servers.length) return "";
  return `cargo install --git https://github.com/Flotrpy/mcp ${servers.join(" ")}`;
}

const shellQuote = (s) => (/^[\w./:@=-]+$/.test(s) ? s : `'${s.replaceAll("'", "'\\''")}'`);
const tomlString = (s) => JSON.stringify(s);

export function clientConfig(client, servers, opts = {}) {
  if (!servers.length) return "";
  switch (client) {
    case "claude-code":
      return servers
        .map((s) => {
          const env = Object.entries(envFor(s, opts)).map(([k, v]) => ` -e ${k}=${shellQuote(v)}`).join("");
          return `claude mcp add ${s}${env} -- ${s}-mcp`;
        })
        .join("\n");
    case "codex":
      return servers
        .map((s) => {
          const env = Object.entries(envFor(s, opts));
          const lines = [`[mcp_servers.${s}]`, `command = ${tomlString(`${s}-mcp`)}`];
          if (env.length) lines.push(`env = { ${env.map(([k, v]) => `${k} = ${tomlString(v)}`).join(", ")} }`);
          return lines.join("\n");
        })
        .join("\n\n");
    default: {
      const mcpServers = {};
      for (const s of servers) {
        const env = envFor(s, opts);
        mcpServers[s] = Object.keys(env).length ? { command: `${s}-mcp`, env } : { command: `${s}-mcp` };
      }
      return JSON.stringify({ mcpServers }, null, 2);
    }
  }
}
