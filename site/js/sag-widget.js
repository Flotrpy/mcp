// The interactive shelf. Controls are native inputs (keyboard and screen
// reader friendly); the right-hand support can also be dragged 1:1, with
// the grab offset respected. The drawn sag follows a spring.

import { check, presetLoadKg, SPECIES } from "./sag.js";
import { Spring } from "./spring.js";

const DEPTH_MM = 280;
const EXAGGERATE = 40; // sag drawn ×40 so millimetres are visible
const X0 = 50;
const X_PER_MM = 900 / 1600; // full width is a 1600 mm span
const Y = 120;

const fmt = (n, d = 1) => n.toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });

export function initSag(root = document.querySelector("[data-sag]")) {
  if (!root) return;
  const form = root.querySelector("form");
  const shelf = root.querySelector("[data-shelf]");
  const pin = root.querySelector('[data-pin="right"]');
  const svg = root.querySelector("svg");
  const out = (k) => root.querySelector(`[data-out="${k}"]`);

  const select = root.querySelector("[data-species]");
  for (const s of SPECIES) select.append(new Option(s.name, s.id, s.id === "birch-ply", s.id === "birch-ply"));

  // Restore from the URL so a configuration can be shared.
  const params = new URLSearchParams(location.search);
  if (params.has("span")) form.span.value = params.get("span");
  for (const name of ["thickness", "load"]) {
    const v = params.get(name);
    const input = v && form.querySelector(`input[name="${name}"][value="${CSS.escape(v)}"]`);
    if (input) input.checked = true;
  }
  if (params.has("species") && SPECIES.some((s) => s.id === params.get("species"))) select.value = params.get("species");

  let spanMm = Number(form.span.value);
  let sagUnits = 0;
  const draw = () => {
    const x1 = X0 + spanMm * X_PER_MM;
    // A quadratic Bézier reaches half its control offset at the midpoint.
    shelf.setAttribute("d", `M${X0} ${Y} Q${(X0 + x1) / 2} ${Y + 2 * sagUnits} ${x1} ${Y}`);
    pin.setAttribute("transform", `translate(${x1 - 950} 0)`);
  };
  const spring = new Spring(0, (v) => {
    sagUnits = v;
    draw();
  });

  let urlTimer = 0;
  const update = (fromUser = true) => {
    spanMm = Number(form.span.value);
    const thicknessMm = Number(form.thickness.value);
    const preset = form.load.value;
    const species = SPECIES.find((s) => s.id === select.value);
    const r = check({ spanMm, depthMm: DEPTH_MM, thicknessMm, loadKg: presetLoadKg(preset, spanMm) }, species);

    out("span").textContent = `${spanMm} mm`;
    out("sag").textContent = `${fmt(r.longTermMm, 2)} mm`;
    out("limit").textContent = `${fmt(r.limitMm, 2)} mm`;
    out("util").textContent = `${Math.round(r.utilisation * 100)}%`;
    const tone = r.utilisation > 1 ? "bad" : r.utilisation > 0.75 ? "warn" : "ok";
    root.dataset.state = tone;
    const verdict = out("verdict");
    const label = Object.assign(document.createElement("strong"), { textContent: r.passes ? "Passes." : "Sags visibly." });
    label.dataset.tone = tone;
    const advice = r.passes
      ? ` ${species.name} at ${thicknessMm} mm holds ${preset} across ${spanMm} mm.`
      : ` Shorten the span to ${Math.floor(r.maxSpanMm / 10) * 10} mm or use ${fmt(Math.ceil(r.minThicknessMm * 10) / 10)} mm stock.`;
    verdict.replaceChildren(label, advice);

    // True to the label: millimetres × EXAGGERATE in the same scale as the span.
    spring.set(Math.min(r.longTermMm * EXAGGERATE * X_PER_MM, 110));
    draw();

    if (!fromUser) return;
    clearTimeout(urlTimer);
    urlTimer = setTimeout(() => {
      const q = new URLSearchParams(location.search);
      q.set("span", String(spanMm));
      q.set("thickness", String(thicknessMm));
      q.set("species", species.id);
      q.set("load", preset);
      history.replaceState(null, "", `?${q}${location.hash}`);
    }, 250);
  };
  form.addEventListener("input", () => update());

  // Drag the right support. Pointer capture keeps tracking outside the SVG.
  let grab = 0;
  const toSvgX = (e) => {
    const box = svg.getBoundingClientRect();
    return ((e.clientX - box.left) / box.width) * 1000;
  };
  pin.addEventListener("pointerdown", (e) => {
    pin.setPointerCapture(e.pointerId);
    pin.toggleAttribute("data-dragging", true);
    grab = toSvgX(e) - (X0 + spanMm * X_PER_MM);
  });
  pin.addEventListener("pointermove", (e) => {
    if (!pin.hasPointerCapture(e.pointerId)) return;
    const mm = Math.round((toSvgX(e) - grab - X0) / X_PER_MM / 10) * 10;
    form.span.value = String(Math.max(Number(form.span.min), Math.min(Number(form.span.max), mm)));
    update();
  });
  const release = (e) => {
    if (pin.hasPointerCapture(e.pointerId)) pin.releasePointerCapture(e.pointerId);
    pin.removeAttribute("data-dragging");
  };
  pin.addEventListener("pointerup", release);
  pin.addEventListener("pointercancel", release);

  update(false);
}
