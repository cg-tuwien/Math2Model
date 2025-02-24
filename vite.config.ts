import { fileURLToPath, URL } from "node:url";
import Components from "unplugin-vue-components/vite";
import { NaiveUiResolver } from "unplugin-vue-components/resolvers";
import Icons from "unplugin-icons/vite";
import IconsResolver from "unplugin-icons/resolver";
import checker from "vite-plugin-checker";

import { defineConfig, UserConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// https://vitejs.dev/config/
const config: UserConfig = defineConfig({
  base: "./",
  plugins: [
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
  css: {
    preprocessorOptions: {
      scss: {
        api: "modern-compiler", // or "modern"
      },
    },
  },
});

export default config;
