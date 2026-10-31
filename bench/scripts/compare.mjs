#!/usr/bin/env node
// Compares two bench results and prints a Markdown table. See bench/README.md.
//
// usage: node bench/scripts/compare.mjs <base> <head>
//
// Each argument is a result file or a directory of them. Exits non-zero only for
// bad input: results are report-only for now (docs/adr/0014).
import { pathToFileURL } from "node:url";

import { readResults } from "./lib.mjs";

const REGRESSION_THRESHOLD_PERCENT = 10;

function formatValue(value, unit) {
  if (value === undefined) return "–";
  const digits = Math.abs(value) >= 100 ? 0 : Math.abs(value) >= 10 ? 1 : 2;
  return `${value.toFixed(digits)} ${unit}`;
}

function formatChange(base, head) {
  if (base === undefined || head === undefined) return "–";
  if (base === 0) return head === 0 ? "0.0%" : "–";
  const percent = ((head - base) / base) * 100;
  const sign = percent >= 0 ? "+" : "";
  const flag = percent > REGRESSION_THRESHOLD_PERCENT ? " ⚠" : "";
  return `${sign}${percent.toFixed(1)}%${flag}`;
}

function meetsBudget(value, op, budget) {
  return op === "<=" ? value <= budget : value < budget;
}

function budgetCell(metric) {
  if (metric.budget === null || metric.budget === undefined) return "–";
  const op = metric.budget_op === "<=" ? "≤" : metric.budget_op;
  return `${op} ${String(metric.budget)} ${metric.unit}${metric.scope ? " *" : ""}`;
}

function resultCell(head) {
  if (!head || head.budget === null || head.budget === undefined) return "–";
  return meetsBudget(head.value, head.budget_op, head.budget) ? "pass" : "FAIL";
}

/** Builds the report for two parsed results. Pure, so it can be tested. */
export function buildReport(base, head) {
  const baseByName = new Map(base.metrics.map((metric) => [metric.name, metric]));
  const headByName = new Map(head.metrics.map((metric) => [metric.name, metric]));
  const names = [...new Set([...headByName.keys(), ...baseByName.keys()])];

  const rows = names.map((name) => {
    const baseMetric = baseByName.get(name);
    const headMetric = headByName.get(name);
    const shown = headMetric ?? baseMetric;
    return [
      name,
      formatValue(baseMetric?.value, shown.unit),
      formatValue(headMetric?.value, shown.unit),
      formatChange(baseMetric?.value, headMetric?.value),
      budgetCell(shown),
      resultCell(headMetric),
    ];
  });

  const header = ["Metric", "Base", "Head", "Change", "Budget", "Result"];
  const widths = header.map((title, column) =>
    Math.max(title.length, ...rows.map((row) => row[column].length)),
  );
  const toLine = (cells) =>
    `| ${cells.map((cell, column) => cell.padEnd(widths[column])).join(" | ")} |`;
  const lines = [
    toLine(header),
    toLine(widths.map((width) => "-".repeat(width))),
    ...rows.map(toLine),
  ];

  const scopes = [...new Set(names.map((name) => headByName.get(name)?.scope).filter(Boolean))];
  const notes = [
    `Values are medians. ⚠ marks a change above +${String(REGRESSION_THRESHOLD_PERCENT)}%. Result compares head with the budget.`,
    ...(scopes.length > 0 ? [`\\* Partial coverage of the budget: ${scopes.join("; ")}.`] : []),
  ];

  return [
    `Base: \`${base.commit ?? "unknown"}\`  `,
    `Head: \`${head.commit ?? "unknown"}\`  `,
    `Machine: ${describeMachine(head.machine)}`,
    "",
    ...lines,
    "",
    ...notes,
  ].join("\n");
}

function describeMachine(machine) {
  if (!machine) return "unknown";
  const parts = [
    machine.cpu,
    machine.ram_gb && `${String(machine.ram_gb)} GB RAM`,
    machine.disk,
    machine.os_build ? `${machine.os} ${machine.os_build}` : machine.os,
  ];
  return parts.filter(Boolean).join(", ");
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [basePath, headPath, ...rest] = process.argv.slice(2);
  if (!basePath || !headPath || rest.length > 0) {
    console.error("usage: node bench/scripts/compare.mjs <base> <head>");
    process.exit(2);
  }
  try {
    console.log(buildReport(readResults(basePath), readResults(headPath)));
  } catch (error) {
    console.error(`error: ${error.message}`);
    process.exit(2);
  }
}
