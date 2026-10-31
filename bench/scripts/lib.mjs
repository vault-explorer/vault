// Result-file helpers shared by the bench scripts. The format matches
// bench/src/results.rs; both are described in bench/README.md.
import { execFileSync } from "node:child_process";
import { mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import os from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const SCHEMA_VERSION = 1;
export const SCRIPTS_DIR = dirname(fileURLToPath(import.meta.url));
export const REPO_ROOT = join(SCRIPTS_DIR, "..", "..");

const budgets = JSON.parse(readFileSync(join(SCRIPTS_DIR, "..", "budgets.json"), "utf8")).metrics;

/** Every metric a full run should produce. */
export const EXPECTED_METRICS = Object.keys(budgets);

export function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 0 ? (sorted[mid - 1] + sorted[mid]) / 2 : sorted[mid];
}

/** Nearest-rank percentile, as in bench/src/results.rs. */
export function percentile(values, percent) {
  const sorted = [...values].sort((a, b) => a - b);
  const rank = Math.max(1, Math.ceil((sorted.length * percent) / 100));
  return sorted[rank - 1];
}

/** Builds a metric from raw samples, taking unit and budget from bench/budgets.json. */
export function metric(name, samples, cache) {
  const entry = budgets[name];
  if (!entry) throw new Error(`${name} is not in bench/budgets.json`);
  if (samples.length === 0) throw new Error(`${name} has no samples`);
  return {
    name,
    value: median(samples),
    unit: entry.unit,
    p95: percentile(samples, 95),
    samples: samples.length,
    budget: entry.budget ?? null,
    budget_op: entry.budget_op ?? null,
    scope: entry.scope ?? null,
    cache,
  };
}

export function writeResults(outDir, suite, commit, metrics) {
  mkdirSync(outDir, { recursive: true });
  const path = join(outDir, `${suite}.json`);
  const file = { schema_version: SCHEMA_VERSION, suite, commit, machine: machineInfo(), metrics };
  writeFileSync(path, `${JSON.stringify(file, null, 2)}\n`);
  return path;
}

/** Adds the machine details docs/PERFORMANCE.md asks for to a result file. */
export function stampMachine(path) {
  const file = JSON.parse(readFileSync(path, "utf8"));
  file.machine = { ...file.machine, ...machineInfo() };
  writeFileSync(path, `${JSON.stringify(file, null, 2)}\n`);
}

let cachedMachineInfo;

function machineInfo() {
  cachedMachineInfo ??= {
    os: process.platform === "win32" ? "windows" : process.platform,
    arch: process.arch === "x64" ? "x86_64" : process.arch,
    logical_cores: os.availableParallelism(),
    cpu: os.cpus()[0]?.model.trim() ?? "unknown",
    ram_gb: Math.round(os.totalmem() / 1024 ** 3),
    os_build: os.release(),
    disk: diskKind(),
  };
  return cachedMachineInfo;
}

function diskKind() {
  if (process.platform !== "win32") return "unknown";
  return powershell(
    "(Get-PhysicalDisk | ForEach-Object { $_.FriendlyName + ' (' + $_.MediaType + ')' }) -join ', '",
  ).trim();
}

export function powershell(command) {
  return execFileSync("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", command], {
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
}

export function gitCommit(repo) {
  try {
    return execFileSync("git", ["rev-parse", "HEAD"], { cwd: repo, encoding: "utf8" }).trim();
  } catch {
    return null;
  }
}

/** Reads a result file, or merges every result file in a directory. */
export function readResults(path) {
  const paths = statSync(path).isDirectory()
    ? readdirSync(path)
        .filter((name) => name.endsWith(".json"))
        .sort()
        .map((name) => join(path, name))
    : [path];
  if (paths.length === 0) throw new Error(`no result files in ${path}`);
  const files = paths.map((file) => {
    const parsed = JSON.parse(readFileSync(file, "utf8"));
    if (parsed.schema_version !== SCHEMA_VERSION) {
      throw new Error(`${file}: unsupported schema version ${String(parsed.schema_version)}`);
    }
    return parsed;
  });
  return {
    commit: files[0].commit,
    machine: files[0].machine,
    metrics: files.flatMap((file) => file.metrics),
  };
}
