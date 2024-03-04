import "./assets/main.css";

import { createApp } from "vue";
import { createPinia } from "pinia";
import { Notification } from "@/notification";

import App from "./App.vue";
import router from "./router";

import "monaco-editor/esm/vs/editor/editor.all.js";
import "monaco-editor/esm/vs/basic-languages/wgsl/wgsl.contribution";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";
import tsWorker from "monaco-editor/esm/vs/language/typescript/ts.worker?worker";

globalThis.addEventListener("unhandledrejection", (event) => {
  Notification.error({
    title: "Unhandled Promise Rejection",
    content: event.reason + "",
  });
  console.error(event);
});
globalThis.addEventListener("error", (event) => {
  Notification.error({
    title: "Unhandled Error",
    content: event.message + "",
  });
  console.error(event);
});

if (self.MonacoEnvironment) {
  console.error(
    "Monaco environment shouldn't exist yet ",
    self.MonacoEnvironment
  );
}
self.MonacoEnvironment = {
  getWorker: function (_, label) {
    switch (label) {
      case "json":
        return new jsonWorker();
      case "typescript":
      case "javascript":
        return new tsWorker();
      default:
        return new editorWorker();
    }
  },
};

const app = createApp(App);

app.use(createPinia());
app.use(router);

app.mount("#app");
