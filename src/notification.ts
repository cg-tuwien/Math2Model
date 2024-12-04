import {
  createDiscreteApi,
  type DialogApi,
  type NotificationApi,
} from "naive-ui";
import type { FilePath } from "./filesystem/reactive-files";
import type { MessageApiInjection } from "naive-ui/es/message/src/MessageProvider";
import type { LoadingBarInst } from "naive-ui/es/loading-bar/src/LoadingBarProvider";

const dest = createDiscreteApi([
  "message",
  "dialog",
  "notification",
  "loadingBar",
]);
export const Message: MessageApiInjection = dest.message;
export const Notification: NotificationApi = dest.notification;
export const Dialog: DialogApi = dest.dialog;
export const LoadingBar: LoadingBarInst = dest.loadingBar;

export function showError(
  msg: string,
  opts: { error?: any; title?: string } = {}
): void {
  // TODO: show the entire error on request
  console.trace(msg, opts.error);
  Notification.error({
    title: opts.title ?? "Error",
    content: msg,
  });
}

export function showFileError(
  msg: string,
  file: FilePath,
  opts: { error?: any; title?: string } = {}
): void {
  // TODO: Show the file path in the error message. And make it clickable to open the file.
  console.trace(msg, opts.error);
  Notification.error({
    title: `Error in ${file}`,
    content: msg,
  });
}

export function showInfo(msg: string): void {
  console.info(msg);
  Message.info(msg);
}
