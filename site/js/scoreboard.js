// Fill the landing-page scoreboard from the same audit the dashboard uses,
// so the grades shown are always the recorded ones (the HTML holds the
// last-known values as a no-JS fallback).

export async function initScoreboard(root = document.querySelector("[data-scoreboard]")) {
  if (!root) return;
  let data;
  try {
    data = await (await fetch("data/dashboard.json")).json();
  } catch {
    return;
  }
  for (const s of data.servers) {
    const li = [...root.children].find((el) => el.querySelector("span")?.textContent === s.server);
    if (!li) continue;
    li.querySelector("b").textContent = s.grade;
    li.title = `${s.points}/100 · ${s.total_tokens} tokens per turn`;
  }
}
