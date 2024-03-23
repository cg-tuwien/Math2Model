import "./assets/main.css";

import { createApp } from "vue";
import { createPinia } from "pinia";
import { Notification } from "@/notification";
import "@/monaco-setup";

import App from "./App.vue";
import router from "./router";

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

const app = createApp(App);

app.use(createPinia());
app.use(router);

app.mount("#app");
