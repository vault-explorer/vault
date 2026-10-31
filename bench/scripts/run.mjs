#!/usr/bin/env node
// Runs the bench suites for one checkout and writes result files. See bench/README.md.
import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { isAbsolute, join, resolve } from "node:path";
import { parseArgs } from "node:util";

import { gitCommit, REPO_ROOT, stampMachine, writeResults } from "./lib.mjs";

const USAGE = `usage: node bench/scripts/run.mjs [options]

  --suite <core|app|all>  what to run (default: core on Linux and macOS, all on Windows)
  --repo <dir>            checkout to build and measure (default: this repo)
  --out <dir>             where result files go (default: <repo>/bench/results)
  --no-build              app suite: reuse binaries from a previous run in <out>/bin
  --runs <n>              app suite: measured runs per metric (default: 5)
  --settle <seconds>      app suite: idle time before reading memory (default: 30)`;

function fail(message) {
  console.error(`error: ${message}\n\n${USAGE}`);
  process.exit(2);
}

function exec(command, args, options) {
  const display = [command, ...args].join(" ");
  console.log(`> ${display}`);
  const result = spawnSync(command, args, { stdio: "inherit", ...options });
  if (result.status !== 0) {
    throw new Error(`${display} failed with status ${String(result.status)}`);
  }
}

function runCore(repo, out, commit) {
  exec("cargo", ["bench", "-p", "vault-bench", "--bench", "core"], {
    cwd: repo,
    env: { ...process.env, VAULT_BENCH_OUT: out, ...(commit && { VAULT_BENCH_COMMIT: commit }) },
  });
  stampMachine(join(out, "core.json"));
}

/** Builds with Tauri (`pnpm` needs a shell on Windows) and copies the exe out. */
function buildApp(repo, destination, { features = [], env = {} } = {}) {
  const featureArgs = features.length > 0 ? ["--features", features.join(",")] : [];
  exec(["pnpm tauri build --no-bundle", ...featureArgs].join(" "), [], {
    cwd: repo,
    shell: true,
    env: { ...process.env, ...env },
  });
  const target = process.env.CARGO_TARGET_DIR ?? join(repo, "target");
  const targetDir = isAbsolute(target) ? target : resolve(repo, target);
  copyFileSync(join(targetDir, "release", "vault.exe"), destination);
}

async function runApp(repo, out, commit, { build, runs, settleSeconds }) {
  if (process.platform !== "win32") {
    throw new Error("the app suite runs on Windows only (see bench/README.md)");
  }
  const bin = join(out, "bin");
  const exe = join(bin, "vault.exe");
  const probeExe = join(bin, "vault-probe.exe");
  if (build) {
    mkdirSync(bin, { recursive: true });
    buildApp(repo, exe);
    buildApp(repo, probeExe, {
      features: ["bench-harness"],
      env: { VITE_VAULT_BENCH: "1" },
    });
  }
  const { runAppBench } = await import("./app-bench.mjs");
  const { metrics, failures } = await runAppBench({ exe, probeExe, runs, settleSeconds });
  if (metrics.length > 0) console.log(`results: ${writeResults(out, "app", commit, metrics)}`);
  for (const failure of failures) console.error(`error: ${failure}`);
  if (failures.length > 0) process.exitCode = 1;
}

const { values } = parseArgs({
  options: {
    suite: { type: "string" },
    repo: { type: "string", default: REPO_ROOT },
    out: { type: "string" },
    "no-build": { type: "boolean", default: false },
    runs: { type: "string", default: "5" },
    settle: { type: "string", default: "30" },
  },
});

const suite = values.suite ?? (process.platform === "win32" ? "all" : "core");
if (!["core", "app", "all"].includes(suite)) fail(`unknown suite ${suite}`);
const runs = Number(values.runs);
const settleSeconds = Number(values.settle);
if (!Number.isInteger(runs) || runs < 1) fail(`bad --runs ${values.runs}`);
if (!(settleSeconds >= 0)) fail(`bad --settle ${values.settle}`);

const repo = resolve(values.repo);
const out = resolve(values.out ?? join(repo, "bench", "results"));
const commit = gitCommit(repo);

if (suite !== "app") runCore(repo, out, commit);
if (suite !== "core")
  await runApp(repo, out, commit, { build: !values["no-build"], runs, settleSeconds });
