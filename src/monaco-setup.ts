import "monaco-editor/esm/vs/editor/edcore.main.js";
import "monaco-editor/esm/vs/basic-languages/wgsl/wgsl.contribution";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";
import tsWorker from "monaco-editor/esm/vs/language/typescript/ts.worker?worker";

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
