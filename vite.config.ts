import { resolve } from "node:path";
import tailwindcss from "@tailwindcss/vite";
import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";
import vueDevTools from "vite-plugin-vue-devtools";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  // Multi-page application setup for separate settings window
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        settings: resolve(__dirname, "settings.html"),
      },
    },
  },
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  plugins: [vue(), tailwindcss(), vueDevTools()],

  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    hmr: host
      ? {
          host,
          port: 1421,
          protocol: "ws",
        }
      : undefined,
    host: host || false,
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
