// ⌘K command palette: native <dialog>, combobox + listbox semantics.
//
// Opening from the keyboard is instant (a frequent keyboard action should
// never wait on an animation); opening by pointer grows the panel out of the
// search button and closes it back along the same path.

import { rank } from "./search.js";

const ITEMS = [
  { title: "Servers", hint: "overview of all five", href: "#servers" },
  { title: "Try joinery", hint: "shelf sag calculator", href: "#sag" },
  { title: "Install", hint: "config builder for your client", href: "#install" },
  { title: "Dashboard", hint: "lighthouse grades, token cost, lockfiles", href: "dashboard.html" },
  { title: "joinery", hint: "design_furniture check_shelf_sag nest_parts list_species", href: "#joinery", accent: "--joinery" },
  { title: "binscope", hint: "overview inspect hexdump diff", href: "#binscope", accent: "--binscope" },
  { title: "lighthouse", hint: "audit_server pin_server verify_server lint_tools", href: "#lighthouse", accent: "--lighthouse" },
  { title: "vitals", hint: "import_health_data data_overview daily_summary readiness trend", href: "#vitals", accent: "--vitals" },
  { title: "handset", hint: "list_devices snapshot act wait_for", href: "#handset", accent: "--handset" },
  { title: "Source on GitHub", hint: "Flotrpy/mcp", href: "https://github.com/Flotrpy/mcp" },
  { title: "Design decisions", hint: "docs/DESIGN.md", href: "https://github.com/Flotrpy/mcp/blob/claude/vibrant-gauss-ej6mfz/docs/DESIGN.md" },
];

export function initPalette(dialog = document.querySelector("[data-palette]")) {
  if (!dialog) return;
  const input = dialog.querySelector("input");
  const list = dialog.querySelector('[role="listbox"]');
  const empty = dialog.querySelector(".palette__empty");
  let results = ITEMS;
  let active = 0;

  const render = () => {
    results = rank(ITEMS, input.value);
    active = Math.min(active, Math.max(results.length - 1, 0));
    list.replaceChildren(
      ...results.map((item, i) => {
        const li = document.createElement("li");
        li.className = "palette__item";
        li.id = `palette-opt-${i}`;
        li.setAttribute("role", "option");
        li.setAttribute("aria-selected", String(i === active));
        if (item.accent) li.style.setProperty("--accent", `var(${item.accent})`);
        const dot = Object.assign(document.createElement("span"), { className: "palette__dot" });
        const title = Object.assign(document.createElement("span"), { className: "palette__title", textContent: item.title });
        const hint = Object.assign(document.createElement("span"), { className: "palette__hint", textContent: item.hint });
        li.append(dot, title, hint);
        li.addEventListener("pointerdown", (e) => e.preventDefault()); // keep focus in the input
        li.addEventListener("click", () => go(item));
        return li;
      }),
    );
    empty.hidden = results.length > 0;
    input.setAttribute("aria-activedescendant", results.length ? `palette-opt-${active}` : "");
  };

  const move = (delta) => {
    if (!results.length) return;
    active = (active + delta + results.length) % results.length;
    for (const [i, li] of [...list.children].entries()) li.setAttribute("aria-selected", String(i === active));
    list.children[active].scrollIntoView({ block: "nearest" });
    input.setAttribute("aria-activedescendant", `palette-opt-${active}`);
  };

  const close = () => {
    if (!dialog.open) return;
    if (!dialog.hasAttribute("data-animate") || matchMedia("(prefers-reduced-motion: reduce)").matches) {
      dialog.close();
      return;
    }
    dialog.toggleAttribute("data-closing", true);
    dialog.addEventListener(
      "animationend",
      () => {
        dialog.removeAttribute("data-closing");
        dialog.close();
      },
      { once: true },
    );
  };

  const go = (item) => {
    close();
    if (item.href.startsWith("#")) {
      document.querySelector(item.href)?.scrollIntoView({ block: "start" });
      history.replaceState(null, "", item.href);
    } else {
      location.href = item.href;
    }
  };

  const open = (fromPointer, trigger) => {
    if (dialog.open) return;
    dialog.toggleAttribute("data-animate", fromPointer);
    if (fromPointer && trigger) {
      // Anchor the motion to the button that opened it.
      const b = trigger.getBoundingClientRect();
      dialog.style.transformOrigin = `${b.left + b.width / 2 - (innerWidth - Math.min(innerWidth - 32, 608)) / 2}px -4rem`;
    }
    input.value = "";
    active = 0;
    render();
    dialog.showModal();
    input.focus();
  };

  input.addEventListener("input", () => {
    active = 0;
    render();
  });
  input.addEventListener("keydown", (e) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      move(1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      move(-1);
    } else if (e.key === "Enter" && results[active]) {
      e.preventDefault();
      go(results[active]);
    }
  });
  dialog.addEventListener("cancel", (e) => {
    e.preventDefault();
    close();
  });
  dialog.addEventListener("click", (e) => {
    if (e.target === dialog) close(); // click on the backdrop
  });
  for (const btn of document.querySelectorAll("[data-palette-open]")) {
    btn.addEventListener("click", (e) => open(e.detail > 0, btn));
  }
  addEventListener("keydown", (e) => {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      dialog.open ? close() : open(false);
    } else if (e.key === "/" && !dialog.open && !/input|textarea|select/i.test(document.activeElement?.tagName ?? "")) {
      e.preventDefault();
      open(false);
    }
  });
}
