// Fuzzy ranking for the command palette (tested in tests/search.test.mjs).

/**
 * Score how well `query` matches `text`: every query character must appear
 * in order. Contiguous runs, word starts and an early first match score
 * higher. Returns -1 for no match.
 */
export function score(query, text) {
  const q = query.trim().toLowerCase();
  if (!q) return 0;
  const t = text.toLowerCase();
  // A contiguous match beats any scattered one; best at a word start.
  let best = -1;
  for (let at = t.indexOf(q); at !== -1; at = t.indexOf(q, at + 1)) {
    const wordStart = at === 0 || /[\s_\-/.]/.test(t[at - 1]);
    best = Math.max(best, 100 + (wordStart ? 50 : 0) + q.length * 2 - at * 0.1);
  }
  if (best >= 0) return best;

  let s = 0;
  let ti = 0;
  let run = 0;
  let first = -1;
  for (const ch of q) {
    if (ch === " ") continue;
    const at = t.indexOf(ch, ti);
    if (at === -1) return -1;
    if (first === -1) first = at;
    run = at === ti ? run + 1 : 1;
    const wordStart = at === 0 || /[\s_\-/.]/.test(t[at - 1]);
    s += 1 + run * 2 + (wordStart ? 4 : 0);
    ti = at + 1;
  }
  return s - first * 0.5 - (t.length - q.length) * 0.05;
}

/** Items matching `query`, best first; all items in order when empty. */
export function rank(items, query, key = (i) => `${i.title} ${i.hint ?? ""}`) {
  if (!query.trim()) return items.slice();
  return items
    .map((item, i) => ({ item, i, s: score(query, key(item)) }))
    .filter((x) => x.s >= 0)
    .sort((a, b) => b.s - a.s || a.i - b.i)
    .map((x) => x.item);
}
