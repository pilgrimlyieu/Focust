import { fileURLToPath } from "node:url";
import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  test: {
    coverage: {
      exclude: [
        "src/**/*.d.ts",
        "src/types/generated/**",
        "src/test/**",
        "src/main.ts",
        "src/vite-env.d.ts",
      ],
      include: ["src/**/*.{ts,vue}"],
      provider: "v8",
      reporter: ["text", "json", "html"],
    },
    globals: true,
    include: ["src/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}"],
    setupFiles: ["./src/test/setup.ts"],
  },
});
