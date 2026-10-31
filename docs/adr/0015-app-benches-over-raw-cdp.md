# 0015. Measure app-level budgets over raw CDP, in bench builds only

- **Status:** Proposed
- **Date:** 2026-10-31
- **Deciders:** project maintainer

## Context

The cold-start, idle-memory and scroll budgets ([PERFORMANCE.md](../PERFORMANCE.md)) have to be measured on the real app in WebView2. The plan and backlog assumed Playwright for this. Measuring also raises three questions: how the harness knows the app is ready, how it gets into the webview, and how to keep that access out of what we ship.

We tested the approach on WebView2 156 before building on it. Setting `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=<port>` exposes the app's page. `Input.synthesizeScrollGesture` then drives a real compositor scroll, and `requestAnimationFrame` timestamps give frame times.

## Decision

- **Talk to CDP directly.** About 100 lines of Node over the built-in `fetch` and `WebSocket` (Node 22+). The harness needs a few CDP calls and none of Playwright's browser management.
- **Ready is an explicit signal.** Once the core has answered and the browser reports `first-contentful-paint`, the UI calls the `app_ready` command, and the app writes `vault:ready` to stdout. The harness times from process launch to that line. A `requestAnimationFrame` plus `setTimeout` fired about 15 ms before the real paint, so we use the paint entry itself.
- **Two binaries.** Cold start and idle memory measure the unmodified release build. Scroll uses a build with the `bench-harness` Cargo feature and `VITE_VAULT_BENCH=1`, which adds the probe page and allows remote debugging. Attaching a debugger would disturb the other two metrics, so they don't use it.
- **Normal release builds clear `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` at startup.** WebView2 honours that variable in any app, so setting it only in the harness wouldn't keep the port out of shipped builds. Verified on Windows: with the variable set, the release build opens no port and the bench build does.
- **The scroll target is a minimal probe list,** not the Details view, because the list spike hasn't happened. Its results are marked as partial coverage everywhere they appear.
- **Each launch uses a temporary WebView2 profile** (`WEBVIEW2_USER_DATA_FOLDER`), so benches never touch a developer's real app data. The first launch on a new profile isn't counted, because users pay that cost only once.
- **Dropped frames are judged against 60 fps,** not the display's refresh rate, so the result doesn't depend on the monitor.

## Alternatives considered

- **Playwright (`connectOverCDP`)** — works with WebView2 and is in the plan, but adds a large dev dependency for a few calls. The list spike can still bring it in if its scenarios need more. It remains the plan's choice for UI tests.
- **A native WebView2 `OpenDevToolsWindow` or trace API** — needs code in the app and isn't scriptable from outside it.
- **One binary for everything** — one fewer build per commit in CI, but cold start and memory would no longer measure what ships.
- **Writing a ready file from the app** — goes through the disk and needs a path passed in. Stdout needs neither, and works with the GUI subsystem when the parent pipes it.

## Consequences

- `app_ready` and its stdout line ship in release builds. They do nothing unless someone reads stdout.
- Developers can't attach to a release build through this variable; debug builds still allow it.
- Each commit needs two app builds, which is most of the bench job's build time.
- `requestAnimationFrame` measures the main thread, not the compositor, so a smooth compositor scroll with a busy main thread reads as worse than it looks. The list spike can add a DevTools trace if it needs that detail.
- In the scroll build, the variable may override the WebView2 arguments Tauri sets through its API. We haven't checked whether that changes scrolling.
- Revisit when the real list lands: the probe page should go away and `scroll.*` should measure the real view.
