import { test } from "node:test";
import assert from "node:assert/strict";
import { rank, score } from "../js/search.js";

const items = [
  { title: "joinery", hint: "design_furniture check_shelf_sag nest_parts" },
  { title: "binscope", hint: "overview inspect hexdump diff" },
  { title: "Install", hint: "config builder" },
  { title: "Dashboard", hint: "lighthouse grades" },
];

test("characters must appear in order", () => {
  assert.equal(score("xyz", "joinery"), -1);
  assert.ok(score("jnr", "joinery") > 0);
});

test("word starts and prefixes win", () => {
  assert.equal(rank(items, "sag")[0].title, "joinery");
  assert.equal(rank(items, "dash")[0].title, "Dashboard");
  assert.equal(rank(items, "hex")[0].title, "binscope");
});

test("empty query keeps the original order", () => {
  assert.deepEqual(rank(items, "  ").map((i) => i.title), items.map((i) => i.title));
});
