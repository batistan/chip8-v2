import { defineConfig } from "vitest/config";
import solid from "vite-plugin-solid";
import wasm from "vite-plugin-wasm";

export default defineConfig({
  plugins: [solid({ hot: true }), wasm()],
  test: {
    environment: "jsdom" // default is overridden by the solid plugin; need to specify manually
  },
  server: {
    fs: {
      allow: [".."],
    },
  },
});
