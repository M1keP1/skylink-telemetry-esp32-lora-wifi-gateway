import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { viteStaticCopy } from "vite-plugin-static-copy";

const cesiumBaseUrl = "cesium";

export default defineConfig({
  // Helpful for Tauri later + local file hosting; also works fine in dev
  base: "./", // Vite base path behavior  [oai_citation:1‡vitejs](https://vite.dev/guide/build?utm_source=chatgpt.com)

  plugins: [
    react(),
    viteStaticCopy({
      targets: [
        { src: "node_modules/cesium/Build/Cesium/Workers", dest: cesiumBaseUrl },
        { src: "node_modules/cesium/Build/Cesium/Assets", dest: cesiumBaseUrl },
        { src: "node_modules/cesium/Build/Cesium/ThirdParty", dest: cesiumBaseUrl },
        { src: "node_modules/cesium/Build/Cesium/Widgets", dest: cesiumBaseUrl },
      ],
    }),
  ],

  define: {
    // Fixes: “Unable to determine Cesium base URL automatically…”
    CESIUM_BASE_URL: JSON.stringify(`./${cesiumBaseUrl}`),
  },
});