import { defineConfig } from "vite"
import react from "@vitejs/plugin-react"
import tsconfigPaths from "vite-tsconfig-paths"
import { resolve } from "path"
import pkg from "./package.json"

const tauriConfig = {
  identifier: "app.whispo",
  productName: "Whispo",
}

const define = {
  "process.env.APP_ID": JSON.stringify(tauriConfig.identifier),
  "process.env.PRODUCT_NAME": JSON.stringify(tauriConfig.productName),
  "process.env.APP_VERSION": JSON.stringify(pkg.version),
  "process.env.IS_MAC": JSON.stringify(process.platform === "darwin"),
}

export default defineConfig({
  plugins: [tsconfigPaths(), react()],
  define,
  root: "./src/renderer",
  build: {
    outDir: "../../out/renderer",
    emptyOutDir: true,
  },
  server: {
    port: 5173,
    strictPort: true,
  },
  clearScreen: false,
  envPrefix: ["VITE_", "TAURI_"],
})
