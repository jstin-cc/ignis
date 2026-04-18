import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  // Tauri dev server: Vite listens on a fixed port so tauri.conf.json can point at it.
  server: {
    port: 1420,
    strictPort: true,
  },
  // Inline assets so the frameless Tauri window loads without a web server in prod.
  build: {
    assetsInlineLimit: 1_000_000,
  },
  // Required so Tauri IPC calls reach the Rust backend.
  clearScreen: false,
  envPrefix: ["VITE_", "TAURI_"],
});
