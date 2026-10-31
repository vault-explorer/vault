// Scroll target for the frame-time bench. Bench builds only; never shipped.
//
// This is deliberately not the Details view. The 100k-row list spike
// (docs/backlog/phase-0.md) replaces it, and until then its numbers show that the
// harness and WebView2 work, not that Vault's real list meets the scroll budget.
import { createMemo, createSignal, Index } from "solid-js";

import "./scroll-probe.css";

const ROW_COUNT = 100_000;
const ROW_HEIGHT_PX = 24;
const OVERSCAN_ROWS = 5;
const EXTENSIONS = ["pdf", "docx", "ts", "rs", "jpg", "png", "mp3", "zip"] as const;

function rowLabel(index: number): string {
  const extension = EXTENSIONS[index % EXTENSIONS.length] ?? "";
  return `file-${String(index).padStart(6, "0")}.${extension}`;
}

export function ScrollProbe() {
  const [scrollTop, setScrollTop] = createSignal(0);
  const [viewportHeight, setViewportHeight] = createSignal(window.innerHeight);

  const firstRow = createMemo(() =>
    Math.max(0, Math.floor(scrollTop() / ROW_HEIGHT_PX) - OVERSCAN_ROWS),
  );
  const visibleRows = createMemo(() => {
    const count = Math.ceil(viewportHeight() / ROW_HEIGHT_PX) + 2 * OVERSCAN_ROWS;
    const first = firstRow();
    return Array.from({ length: Math.min(count, ROW_COUNT - first) }, (_, i) => first + i);
  });

  return (
    <div
      class="scroll-probe"
      data-testid="scroll-probe"
      ref={(element) => {
        queueMicrotask(() => setViewportHeight(element.clientHeight));
      }}
      onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
    >
      <div
        class="scroll-probe__spacer"
        style={{ height: `${String(ROW_COUNT * ROW_HEIGHT_PX)}px` }}
      >
        <Index each={visibleRows()}>
          {(row) => (
            <div
              class="scroll-probe__row"
              data-row
              style={{ transform: `translateY(${String(row() * ROW_HEIGHT_PX)}px)` }}
            >
              <span>{rowLabel(row())}</span>
              <span>{(row() * 37) % 9973} KB</span>
              <span>2026-01-{String((row() % 28) + 1).padStart(2, "0")}</span>
            </div>
          )}
        </Index>
      </div>
    </div>
  );
}
