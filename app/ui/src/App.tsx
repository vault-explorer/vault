import { createEffect, createResource, Match, on, Switch } from "solid-js";

import { appReady, ping } from "./ipc";

/**
 * Runs `callback` once the browser has really painted content. A
 * `requestAnimationFrame` plus `setTimeout` fires about 15 ms too early on WebView2.
 */
function afterFirstContentfulPaint(callback: () => void): void {
  const observer = new PerformanceObserver((entries) => {
    if (entries.getEntriesByName("first-contentful-paint").length > 0) {
      observer.disconnect();
      callback();
    }
  });
  observer.observe({ type: "paint", buffered: true });
}

export function App() {
  const [pong] = createResource(() => ping("hello from the UI"));

  // "Usable" means the first real content is on screen and the core answered.
  // If the core is unreachable we never signal, and the cold-start bench fails
  // loudly rather than timing an error page.
  createEffect(
    on(pong, (reply) => {
      if (reply) {
        afterFirstContentfulPaint(() => void appReady());
      }
    }),
  );

  return (
    <main class="app">
      <Switch>
        <Match when={pong.error as unknown}>
          <p class="status status--error">Core unreachable: {String(pong.error)}</p>
        </Match>
        <Match when={pong()}>
          {(reply) => (
            <p class="status" data-testid="core-status">
              vault-core {reply().coreVersion} on {reply().platform} · echoed “{reply().message}”
            </p>
          )}
        </Match>
      </Switch>
    </main>
  );
}
