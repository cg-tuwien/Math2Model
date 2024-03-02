import { createDiscreteApi } from "naive-ui";

const { message, notification, dialog, loadingBar } = createDiscreteApi([
  "message",
  "dialog",
  "notification",
  "loadingBar",
]);

export function showError(msg: string, error: any) {
  console.error(msg, error);
  message.error(msg);
}

export {
  message as Message,
  notification as Notification,
  dialog as Dialog,
  loadingBar as LoadingBar,
};
