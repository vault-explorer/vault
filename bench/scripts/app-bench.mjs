// App-level benches on Windows: cold start, idle memory and scroll frame times.
//
// Cold start and idle memory measure the unmodified release binary. Scroll needs a
// bench build, with remote debugging kept available and the probe list included.
import { spawn } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createInterface } from "node:readline";
import { setTimeout as sleep } from "node:timers/promises";

import { connectToPage, pollUntil } from "./cdp.mjs";
import { metric, percentile, powershell } from "./lib.mjs";

const READY_LINE = "vault:ready";
const READY_TIMEOUT_MS = 60_000;
const CDP_TIMEOUT_MS = 30_000;
// The first launch on a new WebView2 profile creates the profile, which real users
// pay only once, so it isn't part of the cold-start number.
const PROFILE_WARMUP_LAUNCHES = 1;
// Warm-up pass for the scroll bench: first-time layout and JIT aren't the steady state.
const SCROLL_WARMUP_RUNS = 1;
const SCROLL_DISTANCE_PX = 30_000;
const SCROLL_SPEED_PX_PER_S = 6_000;
// A frame counts as dropped if it missed the 60 fps cadence by half a frame or more.
// Judged against 60 fps, not the display, so a 240 Hz monitor isn't held to 4 ms.
const DROPPED_FRAME_MS = 1000 / 60 + 1000 / 120;
const BYTES_PER_MB = 1024 ** 2; // Same "MB" as Task Manager.

const FRAME_RECORDER = `(() => {
  const frames = [];
  const list = document.querySelector('[data-testid="scroll-probe"]');
  window.__vaultFrames = frames;
  window.__vaultRecording = true;
  const record = (timestamp) => {
    if (!window.__vaultRecording) return;
    frames.push({
      timestamp,
      scrollTop: list.scrollTop,
      rows: list.querySelectorAll("[data-row]").length,
    });
    requestAnimationFrame(record);
  };
  requestAnimationFrame(record);
})()`;

function launch(exe, profileDir, extraEnv = {}) {
  const startedAt = performance.now();
  const child = spawn(exe, [], {
    env: { ...process.env, WEBVIEW2_USER_DATA_FOLDER: profileDir, ...extraEnv },
    stdio: ["ignore", "pipe", "ignore"],
    windowsHide: false,
  });
  const ready = new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      reject(new Error(`no "${READY_LINE}" within ${String(READY_TIMEOUT_MS)} ms`));
    }, READY_TIMEOUT_MS);
    child.once("error", reject);
    child.once("exit", (code) => {
      clearTimeout(timeout);
      reject(new Error(`${exe} exited with code ${String(code)} before it was ready`));
    });
    createInterface({ input: child.stdout }).on("line", (line) => {
      if (line.trim() === READY_LINE) {
        clearTimeout(timeout);
        resolve(performance.now() - startedAt);
      }
    });
  });
  return { child, ready };
}

/** Kills the app and its WebView2 processes and waits for the app to be gone. */
async function terminate(child) {
  if (child.exitCode !== null) return;
  const exited = new Promise((resolve) => child.once("exit", resolve));
  const killer = spawn("taskkill", ["/T", "/F", "/PID", String(child.pid)], { stdio: "ignore" });
  await new Promise((resolve) => killer.once("exit", resolve));
  await exited;
}

function withProfile(run) {
  const profileDir = mkdtempSync(join(tmpdir(), "vault-bench-profile-"));
  return run(profileDir).finally(() => {
    // WebView2 children can hold the profile for a moment after the kill.
    rmSync(profileDir, { recursive: true, force: true, maxRetries: 20, retryDelay: 250 });
  });
}

async function measureColdStart(exe, runs) {
  return withProfile(async (profileDir) => {
    const samples = [];
    for (let i = 0; i < PROFILE_WARMUP_LAUNCHES + runs; i++) {
      const { child, ready } = launch(exe, profileDir);
      try {
        const elapsedMs = await ready;
        if (i >= PROFILE_WARMUP_LAUNCHES) samples.push(elapsedMs);
      } finally {
        await terminate(child);
      }
    }
    return [metric("cold_start", samples, "warm")];
  });
}

/** Sums memory over a process and all its descendants, in bytes. */
function processTreeMemory(rootPid) {
  const raw = powershell(`
    $processes = Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId
    $counters = Get-CimInstance Win32_PerfRawData_PerfProc_Process |
      Select-Object IDProcess, WorkingSetPrivate, WorkingSet
    @{ processes = @($processes); counters = @($counters) } | ConvertTo-Json -Depth 3 -Compress`);
  const { processes, counters } = JSON.parse(raw);

  const tree = new Set([rootPid]);
  for (let grew = true; grew;) {
    grew = false;
    for (const { ProcessId, ParentProcessId } of processes) {
      if (tree.has(ParentProcessId) && !tree.has(ProcessId)) {
        tree.add(ProcessId);
        grew = true;
      }
    }
  }
  const members = counters.filter((counter) => tree.has(counter.IDProcess));
  if (members.length === 0) throw new Error(`no memory counters for process ${String(rootPid)}`);
  const sum = (field) => members.reduce((total, counter) => total + Number(counter[field]), 0);
  return { privateBytes: sum("WorkingSetPrivate"), workingSetBytes: sum("WorkingSet") };
}

async function measureIdleMemory(exe, runs, settleSeconds) {
  return withProfile(async (profileDir) => {
    const privateMb = [];
    const workingSetMb = [];
    for (let i = 0; i < PROFILE_WARMUP_LAUNCHES + runs; i++) {
      const { child, ready } = launch(exe, profileDir);
      try {
        await ready;
        if (i < PROFILE_WARMUP_LAUNCHES) continue;
        await sleep(settleSeconds * 1000);
        const { privateBytes, workingSetBytes } = processTreeMemory(child.pid);
        privateMb.push(privateBytes / BYTES_PER_MB);
        workingSetMb.push(workingSetBytes / BYTES_PER_MB);
      } finally {
        await terminate(child);
      }
    }
    return [
      metric("idle_memory.private_working_set", privateMb, "warm"),
      metric("idle_memory.working_set", workingSetMb, "warm"),
    ];
  });
}

function freePort() {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const { port } = server.address();
      server.close(() => {
        resolve(port);
      });
    });
  });
}

async function openScrollProbe(session) {
  const origin = await session.evaluate("location.origin");
  await session.send("Page.enable");
  await session.send("Page.navigate", { url: `${origin}/?bench=scroll` });
  await pollUntil("the scroll probe to render", CDP_TIMEOUT_MS, async () => {
    try {
      return await session.evaluate(`document.querySelectorAll("[data-row]").length > 0`);
    } catch {
      return false; // The page is mid-navigation.
    }
  });
  if (await session.evaluate("document.hidden")) {
    throw new Error("the window is hidden, so frame times would be meaningless");
  }
}

/** Frame intervals (ms) and peak DOM rows while the list was actually moving. */
export function analyzeFrames(frames) {
  const initial = frames[0]?.scrollTop;
  const final = frames.at(-1)?.scrollTop;
  const first = frames.findIndex((frame) => frame.scrollTop !== initial);
  const last = frames.findIndex((frame) => frame.scrollTop === final);
  if (first < 0 || last <= first) throw new Error("the list did not scroll");
  const moving = frames.slice(first - 1, last + 1);
  const intervals = moving.slice(1).map((frame, i) => frame.timestamp - moving[i].timestamp);
  return {
    intervals,
    domRowsMax: Math.max(...frames.map((frame) => frame.rows)),
    droppedPercent:
      (100 * intervals.filter((interval) => interval > DROPPED_FRAME_MS).length) / intervals.length,
  };
}

async function scrollOnce(session) {
  await session.evaluate(`document.querySelector('[data-testid="scroll-probe"]').scrollTop = 0`);
  await session.evaluate(FRAME_RECORDER);
  await session.send("Input.synthesizeScrollGesture", {
    x: 400,
    y: 300,
    yDistance: -SCROLL_DISTANCE_PX,
    speed: SCROLL_SPEED_PX_PER_S,
    gestureSourceType: "mouse",
  });
  return session.evaluate(`(() => {
    window.__vaultRecording = false;
    return window.__vaultFrames;
  })()`);
}

async function measureScroll(exe, runs) {
  return withProfile(async (profileDir) => {
    const port = await freePort();
    const { child, ready } = launch(exe, profileDir, {
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${String(port)}`,
    });
    let session;
    try {
      await ready;
      session = await connectToPage(port, CDP_TIMEOUT_MS);
      await openScrollProbe(session);

      const passes = [];
      for (let i = 0; i < SCROLL_WARMUP_RUNS + runs; i++) {
        const frames = analyzeFrames(await scrollOnce(session));
        if (i >= SCROLL_WARMUP_RUNS) passes.push(frames);
      }
      const perPass = (statistic) => passes.map(statistic);
      return [
        metric(
          "scroll.frame_time.p50",
          perPass((p) => percentile(p.intervals, 50)),
          "warm",
        ),
        metric(
          "scroll.frame_time.p95",
          perPass((p) => percentile(p.intervals, 95)),
          "warm",
        ),
        metric(
          "scroll.frame_time.p99",
          perPass((p) => percentile(p.intervals, 99)),
          "warm",
        ),
        metric(
          "scroll.dropped_frames",
          perPass((p) => p.droppedPercent),
          "warm",
        ),
        metric(
          "scroll.dom_rows_max",
          perPass((p) => p.domRowsMax),
          "warm",
        ),
      ];
    } finally {
      session?.close();
      await terminate(child);
    }
  });
}

export async function runAppBench({ exe, probeExe, runs, settleSeconds }) {
  return [
    ...(await measureColdStart(exe, runs)),
    ...(await measureIdleMemory(exe, runs, settleSeconds)),
    ...(await measureScroll(probeExe, runs)),
  ];
}
