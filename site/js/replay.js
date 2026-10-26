// Replays recorded MCP sessions in the hero terminal.
//
// Input-driven: a session plays when its tab is chosen (and once when the
// terminal first scrolls into view) — it never loops on its own. Lines fade in
// with a short CSS stagger; reduced-motion users get them instantly.

const SERVERS = ["joinery", "binscope", "lighthouse", "vitals", "handset"];

/** Compact `name(key: value, …)` view of a tool call, trimmed for the screen. */
export function describeCall(tool, args) {
  const parts = Object.entries(args).map(([k, v]) => {
    let s = typeof v === "string" ? JSON.stringify(v) : JSON.stringify(v);
    if (s.length > 28) s = s.slice(0, 26) + "…\"";
    return `${k}: ${s}`;
  });
  let inner = parts.join(", ");
  if (inner.length > 70) inner = inner.slice(0, 68) + "…";
  return `${tool}(${inner})`;
}

/** Classify a reply line for colouring (diff-style +/-). */
export function lineClass(line) {
  if (line.startsWith("+ ")) return "add";
  if (line.startsWith("- ")) return "del";
  return "";
}

function el(tag, cls, text) {
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  if (text != null) n.textContent = text;
  return n;
}

function render(screen, server, calls) {
  const frag = document.createDocumentFragment();
  let i = 0;
  const push = (node) => {
    node.style.setProperty("--i", String(i++));
    frag.append(node);
  };
  for (const call of calls) {
    const req = el("span", "ln req");
    req.append("› ", el("b", null, server), " " + describeCall(call.tool, call.args));
    push(req);
    for (const line of call.reply.split("\n")) {
      push(el("span", `ln ${lineClass(line)}`.trim(), line || " "));
    }
    push(el("span", "ln gap"));
  }
  screen.replaceChildren(frag);
  screen.parentElement.scrollTop = 0;
}

export async function initReplay(root = document.querySelector("[data-replay]")) {
  if (!root) return;
  const screen = root.querySelector("[data-replay-screen]");
  const tabs = [...root.querySelectorAll('[role="tab"]')];
  let data;
  try {
    data = await (await fetch("data/transcripts.json")).json();
  } catch {
    screen.textContent = "Recorded sessions could not be loaded — see the examples on GitHub.";
    return;
  }
  let current = "joinery";
  const select = (server, focus = false) => {
    current = server;
    for (const t of tabs) {
      const on = t.dataset.server === server;
      t.setAttribute("aria-selected", String(on));
      t.tabIndex = on ? 0 : -1;
      if (on && focus) t.focus();
    }
    render(screen, server, data[server] || []);
  };

  for (const t of tabs) t.addEventListener("click", () => select(t.dataset.server));
  // Arrow keys move between tabs (WAI-ARIA tabs pattern).
  root.querySelector('[role="tablist"]').addEventListener("keydown", (e) => {
    const i = SERVERS.indexOf(current);
    const next = { ArrowRight: i + 1, ArrowLeft: i - 1, Home: 0, End: SERVERS.length - 1 }[e.key];
    if (next === undefined) return;
    e.preventDefault();
    select(SERVERS[(next + SERVERS.length) % SERVERS.length], true);
  });
  root.querySelector("[data-replay-again]").addEventListener("click", () => select(current));

  // Play the first session once, when the terminal becomes visible.
  const io = new IntersectionObserver((entries) => {
    if (entries.some((e) => e.isIntersecting)) {
      select(current);
      io.disconnect();
    }
  }, { threshold: 0.25 });
  io.observe(root);
}
