// Typed wrappers around Tauri commands. All UI ↔ core traffic goes through
// this module: the UI never touches the disk itself.
import { invoke } from "@tauri-apps/api/core";

/** Mirrors `PongDto` in app/src-tauri/src/lib.rs. */
export interface Pong {
  message: string;
  coreVersion: string;
  platform: string;
}

/** Scaffold-only round trip through vault-core. */
export function ping(message: string): Promise<Pong> {
  return invoke<Pong>("ping", { message });
}

/** Tells the core the UI is usable. Call once, after the first paint. */
export function appReady(): Promise<void> {
  return invoke("app_ready");
}
