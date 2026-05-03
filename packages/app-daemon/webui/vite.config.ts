import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";
import { viteSingleFile } from "vite-plugin-singlefile";
import path from "path";

export default defineConfig({
  root: path.resolve(__dirname, "src"),
  plugins: [tailwindcss(), viteSingleFile()],
  build: {
    outDir: path.resolve(__dirname, "dist"),
    target: "es2022",
    cssCodeSplit: false,
    assetsInlineLimit: Infinity,
    rollupOptions: {
      output: {
        inlineDynamicImports: true,
      },
    },
  },
});
