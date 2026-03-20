import babel from "@rolldown/plugin-babel";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), babel({ presets: [reactCompilerPreset()] }), viteSingleFile()],
  server: {
    fs: {
      // viewer ディレクトリの外側（Rustのtargetディレクトリ）の読み取りを許可
      allow: [".."],
    },
  },
});
