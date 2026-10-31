// Minimal Chrome DevTools Protocol client over Node's built-in fetch and WebSocket.

const POLL_INTERVAL_MS = 50;

export async function pollUntil(description, timeoutMs, attempt) {
  const deadline = performance.now() + timeoutMs;
  for (;;) {
    const value = await attempt();
    if (value) return value;
    if (performance.now() > deadline) {
      const what = typeof description === "function" ? description() : description;
      throw new Error(`timed out waiting for ${what}`);
    }
    await new Promise((resolve) => setTimeout(resolve, POLL_INTERVAL_MS));
  }
}

/** Waits for a page target on `port` and connects to it. */
export async function connectToPage(port, timeoutMs) {
  let lastSeen = "nothing";
  const target = await pollUntil(
    () => `a page target on port ${String(port)}; last saw ${lastSeen}`,
    timeoutMs,
    async () => {
      try {
        const response = await fetch(`http://127.0.0.1:${String(port)}/json/list`);
        const targets = await response.json();
        lastSeen = `targets ${JSON.stringify(targets.map(({ type, url }) => ({ type, url })))}`;
        return targets.find((candidate) => candidate.type === "page");
      } catch (error) {
        lastSeen = `${error.message} (${String(error.cause?.code)})`; // Not listening yet.
        return undefined;
      }
    },
  );

  const socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });
  return new CdpSession(socket);
}

class CdpSession {
  #socket;
  #nextId = 1;
  #pending = new Map();

  constructor(socket) {
    this.#socket = socket;
    socket.addEventListener("message", (event) => {
      const message = JSON.parse(event.data);
      const pending = this.#pending.get(message.id);
      if (!pending) return;
      this.#pending.delete(message.id);
      if (message.error) {
        pending.reject(new Error(`${pending.method}: ${message.error.message}`));
      } else {
        pending.resolve(message.result);
      }
    });
    socket.addEventListener("close", () => {
      for (const pending of this.#pending.values()) {
        pending.reject(new Error(`${pending.method}: connection closed`));
      }
      this.#pending.clear();
    });
  }

  send(method, params = {}) {
    return new Promise((resolve, reject) => {
      const id = this.#nextId++;
      this.#pending.set(id, { method, resolve, reject });
      this.#socket.send(JSON.stringify({ id, method, params }));
    });
  }

  /** Evaluates `expression` in the page and returns its value. */
  async evaluate(expression) {
    const { result, exceptionDetails } = await this.send("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
    });
    if (exceptionDetails) {
      throw new Error(`page threw: ${exceptionDetails.exception?.description ?? expression}`);
    }
    return result.value;
  }

  close() {
    this.#socket.close();
  }
}
