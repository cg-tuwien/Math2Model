import "./assets/main.css";

import { createApp } from "vue";
import { createPinia } from "pinia";
import { showError } from "@/notification";
import "@/monaco-setup";

import App from "./App.vue";
import router from "./views/router";

if (globalThis.navigator.gpu === undefined) {
  showError(
    "WebGPU not supported\nPlease switch a more modern browser (Chrome or Firefox 141)",
    {
      error: "WebGPU not supported",
    }
  );
  throw new Error("a");
}

globalThis.addEventListener("unhandledrejection", (event) => {
  showError(event.reason, {
    title: "Unhandled Promise Rejection",
    error: event,
  });
});
globalThis.addEventListener("error", (event) => {
  if (
    event.error?.message ===
    "Using exceptions for control flow, don't mind me. This isn't actually an error!"
  ) {
    return;
  }
  showError(event.message, {
    error: event,
  });
});

const app = createApp(App);

app.use(createPinia());
app.use(router);

app.mount("#app");
