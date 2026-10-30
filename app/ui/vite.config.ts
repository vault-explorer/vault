import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

// Tauri expects a fixed dev-server port and serves the built files itself.
// https://v2.tauri.app/start/frontend/vite/
export default defineConfig({
  plugins: [solid()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  envPrefix: ["VITE_", "TAURI_ENV_"],
  build: {
    // WebView2 on Windows 10 22H2+ is Chromium-based and evergreen.
    target: "chrome120",
  },
});
