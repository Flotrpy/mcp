// Entry point: wires up each interactive piece once the DOM is ready.
// Every module is optional — the page is fully readable without JavaScript.

import { initReplay } from "./replay.js";
import { initReveal } from "./reveal.js";
import { initSag } from "./sag-widget.js";
import { initBuilder } from "./builder.js";
import { initPalette } from "./palette.js";
import { initScoreboard } from "./scoreboard.js";

const isMac = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);

// Show the right modifier for the palette shortcut.
for (const el of document.querySelectorAll("[data-shortcut]")) {
  el.textContent = isMac ? "⌘ K" : "Ctrl K";
}

// Show the nav's scroll-edge fade only while content is under it.
const nav = document.querySelector(".nav");
if (nav) {
  const sentinel = document.createElement("div");
  sentinel.setAttribute("aria-hidden", "true");
  document.body.prepend(sentinel);
  new IntersectionObserver(([e]) => nav.toggleAttribute("data-scrolled", !e.isIntersecting)).observe(sentinel);
}

// Interactive forms never navigate; Enter in a text field must not reload.
for (const form of document.querySelectorAll("[data-sag] form, [data-builder] form")) {
  form.addEventListener("submit", (e) => e.preventDefault());
}

initReplay();
initReveal();
initSag();
initBuilder();
initPalette();
initScoreboard();

export { isMac };
