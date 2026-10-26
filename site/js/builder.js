// Install builder: form state ↔ URL, live snippets, copy with feedback.

import { CLIENTS, clientConfig, installCommand } from "./config.js";

export function initBuilder(root = document.querySelector("[data-builder]")) {
  if (!root) return;
  const form = root.querySelector("form");
  const out = (k) => root.querySelector(`[data-out="${k}"]`);

  // Restore ?servers=a,b&client=x
  const q = new URLSearchParams(location.search);
  if (q.has("servers")) {
    const wanted = new Set(q.get("servers").split(","));
    for (const box of form.querySelectorAll('input[name="server"]')) box.checked = wanted.has(box.value);
  }
  const client = q.get("client");
  if (client && CLIENTS[client]) form.querySelector(`input[name="client"][value="${client}"]`).checked = true;

  const snippets = { install: "", config: "" };

  const update = (fromUser = true) => {
    const servers = [...form.querySelectorAll('input[name="server"]:checked')].map((b) => b.value);
    const chosen = form.client.value;
    const opts = { binscopeRoot: form.binscopeRoot.value.trim(), vitalsRedact: form.vitalsRedact.checked };

    // Only show options for servers that are selected.
    for (const el of form.querySelectorAll("[data-when]")) {
      el.hidden = !el.dataset.when.split(" ").some((s) => servers.includes(s));
    }

    snippets.install = installCommand(servers);
    snippets.config = clientConfig(chosen, servers, opts);
    out("install").textContent = snippets.install;
    out("config").textContent = snippets.config;
    out("file").textContent = `2. ${CLIENTS[chosen].file}`;
    out("empty").hidden = servers.length > 0;
    for (const c of root.querySelectorAll(".code")) c.hidden = servers.length === 0;

    // Only record state once the visitor has changed something.
    if (!fromUser) return;
    const next = new URLSearchParams(location.search);
    next.set("servers", servers.join(","));
    next.set("client", chosen);
    history.replaceState(null, "", `?${next}${location.hash}`);
  };
  form.addEventListener("input", () => update());

  for (const btn of root.querySelectorAll("[data-copy]")) {
    let timer = 0;
    btn.addEventListener("click", async () => {
      try {
        await navigator.clipboard.writeText(snippets[btn.dataset.copy]);
        btn.textContent = "Copied";
        btn.toggleAttribute("data-copied", true);
        out("status").textContent = "Copied to the clipboard.";
      } catch {
        out("status").textContent = "Copy is blocked here — select the text and copy it instead.";
      }
      clearTimeout(timer);
      timer = setTimeout(() => {
        btn.textContent = "Copy";
        btn.removeAttribute("data-copied");
      }, 1600);
    });
  }

  update(false);
}
