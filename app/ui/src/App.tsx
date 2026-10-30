import { createResource, Match, Switch } from "solid-js";

import { ping } from "./ipc";

export function App() {
  const [pong] = createResource(() => ping("hello from the UI"));

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
