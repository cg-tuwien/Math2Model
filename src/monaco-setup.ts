import "monaco-editor/esm/vs/editor/edcore.main.js";
import "monaco-editor/esm/vs/basic-languages/wgsl/wgsl.contribution";
import "monaco-editor/esm/vs/language/json/monaco.contribution";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";
import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import zodToJsonSchema from "zod-to-json-schema";
import { SceneFileSchema, SceneFileSchemaUrl } from "./filesystem/scene-file";

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
      default:
        return new editorWorker();
    }
  },
};

monaco.languages.json.jsonDefaults.setDiagnosticsOptions({
  validate: true,
  schemas: [
    {
      uri: SceneFileSchemaUrl,
      schema: zodToJsonSchema(SceneFileSchema, "sceneSchema"),
    },
  ],
});
