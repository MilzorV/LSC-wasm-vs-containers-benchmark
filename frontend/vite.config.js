import { resolve } from "node:path";
import { defineConfig } from "vite";

export default defineConfig({
  base: "./",
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        demo: resolve(__dirname, "demo/index.html"),
      },
    },
  },
  server: {
    port: 5173,
    proxy: {
      "/health": "http://127.0.0.1:8080",
      "/version": "http://127.0.0.1:8080",
      "/stats": "http://127.0.0.1:8080",
      "/movies": "http://127.0.0.1:8080",
      "/search": "http://127.0.0.1:8080",
    },
  },
});
