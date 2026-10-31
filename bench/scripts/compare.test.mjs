import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { analyzeFrames } from "./app-bench.mjs";
import { buildReport } from "./compare.mjs";
import { median, metric, percentile, readResults, SCHEMA_VERSION } from "./lib.mjs";

const measured = (name, value, extra = {}) => ({
  name,
  value,
  unit: "ms",
  budget: null,
  budget_op: null,
  scope: null,
  ...extra,
});
const result = (metrics, commit = "abc") => ({ commit, machine: { os: "windows" }, metrics });
const rowFor = (report, name) => report.split("\n").find((line) => line.startsWith(`| ${name} `));
const cells = (row) =>
  row
    .split("|")
    .slice(1, -1)
    .map((cell) => cell.trim());

test("reports change and budget result", () => {
  const budgeted = { budget: 50, budget_op: "<" };
  const report = buildReport(
    result([measured("a", 10, budgeted)]),
    result([measured("a", 12, budgeted)]),
  );
  assert.deepEqual(cells(rowFor(report, "a")), [
    "a",
    "10.0 ms",
    "12.0 ms",
    "+20.0% ⚠",
    "< 50 ms",
    "pass",
  ]);
});

test("fails a metric over budget, and honours strict and inclusive limits", () => {
  const strict = { budget: 50, budget_op: "<" };
  const inclusive = { budget: 60, budget_op: "<=" };
  const report = buildReport(
    result([
      measured("over", 40, strict),
      measured("strict", 40, strict),
      measured("incl", 40, inclusive),
    ]),
    result([
      measured("over", 70, strict),
      measured("strict", 50, strict),
      measured("incl", 60, inclusive),
    ]),
  );
  assert.equal(cells(rowFor(report, "over"))[5], "FAIL");
  assert.equal(cells(rowFor(report, "strict"))[5], "FAIL");
  assert.equal(cells(rowFor(report, "incl"))[5], "pass");
});

test("does not flag improvements or changes within the threshold", () => {
  const report = buildReport(
    result([measured("faster", 10), measured("noise", 10)]),
    result([measured("faster", 5), measured("noise", 10.5)]),
  );
  assert.equal(cells(rowFor(report, "faster"))[3], "-50.0%");
  assert.equal(cells(rowFor(report, "noise"))[3], "+5.0%");
});

test("handles metrics present on only one side", () => {
  const report = buildReport(result([measured("gone", 1)]), result([measured("new", 2)]));
  assert.deepEqual(cells(rowFor(report, "new")).slice(1, 4), ["–", "2.00 ms", "–"]);
  assert.deepEqual(cells(rowFor(report, "gone")).slice(1, 4), ["1.00 ms", "–", "–"]);
  assert.equal(cells(rowFor(report, "gone"))[5], "–");
});

test("marks partial budgets and explains them", () => {
  const partial = { budget: 50, budget_op: "<", scope: "core only" };
  const report = buildReport(
    result([measured("a", 1, partial)]),
    result([measured("a", 1, partial)]),
  );
  assert.equal(cells(rowFor(report, "a"))[4], "< 50 ms *");
  assert.match(report, /Partial coverage of the budget: core only/);
});

test("zero base doesn't divide by zero", () => {
  const report = buildReport(
    result([measured("a", 0), measured("b", 0)]),
    result([measured("a", 0), measured("b", 3)]),
  );
  assert.equal(cells(rowFor(report, "a"))[3], "0.0%");
  assert.equal(cells(rowFor(report, "b"))[3], "–");
});

test("reads a file, or merges a directory of files", () => {
  const dir = mkdtempSync(join(tmpdir(), "vault-compare-"));
  try {
    const write = (name, metrics) => {
      const file = { schema_version: SCHEMA_VERSION, commit: "c", machine: {}, metrics };
      writeFileSync(join(dir, name), JSON.stringify(file));
    };
    write("app.json", [measured("a", 1)]);
    write("core.json", [measured("b", 2)]);
    assert.equal(readResults(dir).metrics.length, 2);
    assert.equal(readResults(join(dir, "core.json")).metrics.length, 1);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("rejects other schema versions", () => {
  const dir = mkdtempSync(join(tmpdir(), "vault-compare-"));
  try {
    const path = join(dir, "x.json");
    writeFileSync(path, JSON.stringify({ schema_version: 999, metrics: [] }));
    assert.throws(() => readResults(path), /unsupported schema version/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("statistics match bench/src/results.rs", () => {
  assert.equal(median([5, 1, 3]), 3);
  assert.equal(median([1, 2, 3, 10]), 2.5);
  assert.equal(
    percentile(
      Array.from({ length: 100 }, (_, i) => i + 1),
      95,
    ),
    95,
  );
  assert.equal(percentile([7], 95), 7);
});

test("metric takes unit and budget from budgets.json", () => {
  const m = metric("cold_start", [100, 300, 200], "warm");
  assert.deepEqual(
    [m.value, m.p95, m.samples, m.unit, m.budget, m.budget_op],
    [200, 300, 3, "ms", 400, "<"],
  );
  assert.throws(() => metric("nope", [1], "warm"), /not in bench\/budgets.json/);
  assert.throws(() => metric("cold_start", [], "warm"), /no samples/);
});

test("analyzeFrames keeps only frames while the list moved", () => {
  const frame = (timestamp, scrollTop, rows = 40) => ({ timestamp, scrollTop, rows });
  const { intervals, domRowsMax, droppedPercent } = analyzeFrames([
    frame(0, 0),
    frame(500, 0), // Idle before the gesture: not a dropped frame.
    frame(516, 100),
    frame(532, 200, 44),
    frame(582, 300),
    frame(598, 400),
    frame(1500, 400), // Idle after the gesture.
  ]);
  assert.deepEqual(intervals, [16, 16, 50, 16]);
  assert.equal(domRowsMax, 44);
  assert.equal(droppedPercent, 25);
});

test("analyzeFrames rejects a list that never moved", () => {
  const still = [0, 16, 32].map((timestamp) => ({ timestamp, scrollTop: 0, rows: 40 }));
  assert.throws(() => analyzeFrames(still), /did not scroll/);
});
