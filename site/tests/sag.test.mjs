// Run with: npm test (in site/) or node --test site/tests/*.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { check, SPECIES, MM_PER_INCH, LB_PER_KG, presetLoadKg } from "../js/sag.js";

const oak = SPECIES.find((s) => s.id === "red-oak");

test("matches the hand calculation used in the Rust tests", () => {
  // 36" × 10" × 0.75" red oak with 60 lb: δ ≈ 0.0569"
  const r = check({ spanMm: 36 * MM_PER_INCH, depthMm: 10 * MM_PER_INCH, thicknessMm: 0.75 * MM_PER_INCH, loadKg: 60 / LB_PER_KG }, oak);
  assert.ok(Math.abs(r.deflectionMm / MM_PER_INCH - 0.0569) < 0.0005, String(r.deflectionMm));
});

test("suggested span and thickness land exactly on the limit", () => {
  const base = { spanMm: 1000, depthMm: 280, thicknessMm: 18, loadKg: presetLoadKg("books", 1000) };
  const r = check(base, oak);
  const atThickness = check({ ...base, thicknessMm: r.minThicknessMm }, oak);
  assert.ok(Math.abs(atThickness.utilisation - 1) < 1e-9);
});

test("species table matches crates/joinery/src/model.rs", () => {
  const rust = readFileSync(fileURLToPath(new URL("../../crates/joinery/src/model.rs", import.meta.url)), "utf8");
  const rows = [...rust.matchAll(/id: "([a-z-]+)",[\s\S]*?moe_psi: ([\d_]+)\.0,\s*creep_factor: ([\d.]+)/g)];
  assert.equal(rows.length, SPECIES.length);
  for (const [, id, moe, creep] of rows) {
    const js = SPECIES.find((s) => s.id === id);
    assert.ok(js, `missing ${id}`);
    assert.equal(js.moe, Number(moe.replaceAll("_", "")), id);
    assert.equal(js.creep, Number(creep), id);
  }
});
