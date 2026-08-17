import { defineConfig } from "vite";

export default defineConfig({
  root: "app",
  clearScreen: false,
  server: { strictPort: true },
  build: { outDir: "../dist", emptyOutDir: true },
});
