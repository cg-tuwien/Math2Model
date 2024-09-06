import { createDiscreteApi } from "naive-ui";
import type { FilePath } from "./filesystem/reactive-files";

const { message, notification, dialog, loadingBar } = createDiscreteApi([
  "message",
  "dialog",
  "notification",
  "loadingBar",
]);

export function showError(
  msg: string,
  opts: { error?: any; title?: string } = {}
) {
  // TODO: show the entire error on request
  console.error(msg, opts.error);
  notification.error({
    title: opts.title ?? "Error",
    content: msg,
  });
}

export function showFileError(
  msg: string,
  file: FilePath,
  opts: { error?: any; title?: string } = {}
) {
  // TODO: Show the file path in the error message. And make it clickable to open the file.
  console.error(msg, opts.error);
  notification.error({
    title: `Error in ${file}`,
    content: msg,
  });
}

export function showInfo(msg: string) {
  console.info(msg);
  message.info(msg);
}

export {
  message as Message,
  notification as Notification,
  dialog as Dialog,
  loadingBar as LoadingBar,
};
