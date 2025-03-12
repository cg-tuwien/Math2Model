import { fileURLToPath, URL } from "node:url";
import Components from "unplugin-vue-components/vite";
import { NaiveUiResolver } from "unplugin-vue-components/resolvers";
import Icons from "unplugin-icons/vite";
import IconsResolver from "unplugin-icons/resolver";
import checker from "vite-plugin-checker";
import tailwindcss from "@tailwindcss/vite";

import { defineConfig, UserConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// https://vitejs.dev/config/
const config: UserConfig = defineConfig({
  base: "./",
  plugins: [
    tailwindcss(),
    vue(),
    Icons(),
    Components({
      resolvers: [
        NaiveUiResolver(),
        IconsResolver({ prefix: false, enabledCollections: ["mdi"] }),
      ],
    }),
    checker({
      vueTsc: true,
    }),
  ],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  build: {
    target: "esnext",
  },
  server: {
    watch: {
      ignored: [
        "**/parametric-renderer-core/target/**",
        "**/parametric-renderer-core/desktop/**",
        "**/parametric-renderer-core/renderer-core/**",
        "**/parametric-renderer-core/wasm/**",
      ],
    },
  },
});

export default config;
