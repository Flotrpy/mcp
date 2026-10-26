// Renders site/data/dashboard.json (written by tools/record.py).

const ACCENT = { joinery: "--joinery", binscope: "--binscope", lighthouse: "--lighthouse", vitals: "--vitals", handset: "--handset" };
const POINTS = { A: 4, B: 3, C: 2, D: 1, F: 0 };
const nf = new Intl.NumberFormat();

const h = (tag, props = {}, ...kids) => {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(props)) {
    if (k === "style") for (const [p, val] of Object.entries(v)) el.style.setProperty(p, val);
    else if (k.startsWith("data-") || k.startsWith("aria-") || k === "role") el.setAttribute(k, v);
    else el[k] = v;
  }
  el.append(...kids.filter((x) => x != null));
  return el;
};

const svg = (tag, attrs = {}, ...kids) => {
  const el = document.createElementNS("http://www.w3.org/2000/svg", tag);
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, String(v));
  el.append(...kids);
  return el;
};

function ring(points, grade) {
  const r = 34;
  const c = 2 * Math.PI * r;
  return svg(
    "svg",
    { class: "ring", viewBox: "0 0 80 80", role: "img", "aria-label": `Grade ${grade}, ${points} out of 100` },
    svg("circle", { class: "ring__track", cx: 40, cy: 40, r }),
    svg("circle", {
      class: "ring__value", cx: 40, cy: 40, r,
      "stroke-dasharray": c, "stroke-dashoffset": c * (1 - points / 100), style: `--circumference: ${c}`,
    }),
    svg("text", { class: "ring__grade", x: 40, y: 41, "text-anchor": "middle", "dominant-baseline": "middle" }, grade),
  );
}

const badge = (tone, text) => h("span", { className: "badge", "data-tone": tone }, text);

function card(s) {
  const migration = {
    ready: ["ok", "Both protocols"],
    "legacy-only": ["warn", "Legacy protocol only"],
    "stateless-only": ["warn", "Stateless only"],
    unreachable: ["bad", "Unreachable"],
  }[s.migration] ?? ["warn", s.migration];
  return h(
    "li",
    { className: "card", style: { "--accent": `var(${ACCENT[s.server] ?? "--link"})` } },
    ring(s.points, s.grade),
    h("p", { className: "card__name", translate: false }, s.server, h("span", { className: "card__version" }, s.version ?? "")),
    h(
      "p",
      { className: "card__meta tabular" },
      `${s.tools.length} tools · ${nf.format(s.total_tokens)} tokens · ${(s.binary_bytes / 1e6).toFixed(1)} MB · ${s.handshake_ms.stateless ?? "–"} ms`,
    ),
    h("div", { className: "badges" }, badge(...migration), s.lock.clean ? badge("ok", "Matches mcp.lock") : badge("bad", "Drift from mcp.lock")),
  );
}

function costChart(servers) {
  const max = Math.max(...servers.flatMap((s) => s.tools.map((t) => t.tokens)));
  return servers.map((s) => {
    const heaviest = s.tools[0];
    const sentence = `${s.server}: ${s.tools.length} tools totalling ${s.total_tokens} tokens; heaviest is ${heaviest.tool} at ${heaviest.tokens}.`;
    return h(
      "div",
      { className: "cost__server", style: { "--accent": `var(${ACCENT[s.server]})` } },
      h("p", { className: "cost__head" }, h("b", { translate: false }, s.server), h("span", { className: "tabular" }, `${nf.format(s.total_tokens)} tokens`)),
      h(
        "div",
        { className: "cost__bars", role: "img", "aria-label": sentence },
        ...s.tools.map((t, i) =>
          h(
            "div",
            { className: "bar", "aria-hidden": "true" },
            h("span", { className: "bar__label", translate: false }, t.tool),
            h("span", { className: "bar__track" }, h("span", { className: "bar__fill", style: { width: `${(100 * t.tokens) / max}%`, "--i": String(i) } })),
            h("span", { className: "bar__value tabular" }, nf.format(t.tokens)),
          ),
        ),
      ),
      h(
        "details",
        {},
        h("summary", {}, "Data table"),
        h(
          "table",
          {},
          h("thead", {}, h("tr", {}, h("th", { scope: "col" }, "Tool"), h("th", { scope: "col" }, "Tokens"))),
          h("tbody", {}, ...s.tools.map((t) => h("tr", {}, h("td", { translate: false }, t.tool), h("td", { className: "tabular" }, String(t.tokens))))),
        ),
      ),
    );
  });
}

async function main() {
  const root = document.querySelector("[data-dashboard]");
  const out = (k) => root.querySelector(`[data-out="${k}"]`);
  let data;
  try {
    data = await (await fetch("data/dashboard.json")).json();
  } catch {
    out("headline").textContent = "The audit data could not be loaded — run site/tools/record.py and reload.";
    return;
  }
  const s = data.servers;
  const time = out("generated");
  time.dateTime = data.generated;
  time.textContent = new Date(`${data.generated}T12:00:00`).toLocaleDateString(undefined, { dateStyle: "medium" });

  const drift = s.filter((x) => !x.lock.clean).length;
  const belowA = s.filter((x) => x.grade !== "A").length;
  out("headline").textContent =
    drift === 0 && belowA === 0
      ? `All ${s.length} servers grade A and match their lockfiles.`
      : `${belowA} server(s) below an A, ${drift} drifting from their lockfile.`;

  const avg = s.reduce((n, x) => n + POINTS[x.grade], 0) / s.length;
  out("avg").textContent = Object.entries(POINTS).find(([, p]) => p <= Math.round(avg))?.[0] ?? "–";
  out("tools").textContent = nf.format(s.reduce((n, x) => n + x.tools.length, 0));
  out("tokens").textContent = nf.format(s.reduce((n, x) => n + x.total_tokens, 0));
  out("locks").textContent = `${s.length - drift}/${s.length}`;

  out("cards").replaceChildren(...s.map(card));
  out("cost").replaceChildren(...costChart(s));
}

main();
