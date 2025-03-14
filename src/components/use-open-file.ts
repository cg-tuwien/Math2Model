import type { FilePath, ReactiveFilesystem } from "@/filesystem/reactive-files";
import { computed, ref, type Reactive } from "vue";
import type { KeyedCode, Marker } from "./CodeEditor.vue";
import { MarkerSeverity } from "monaco-editor";
import { useDebounceFn, watchImmediate } from "@vueuse/core";
import { showError, showFileError } from "@/notification";
import type { WasmCompilationMessage } from "parametric-renderer-core/pkg/web";

export type EditorType = "shader" | "graph";

/** The logic for the open code file */
export function useOpenFile(
  startFile: FilePath | null,
  fs: ReactiveFilesystem,
  errorMessages: Reactive<Map<FilePath, WasmCompilationMessage[]>>
) {
  const openedFileName = ref<FilePath | null>(startFile);
  const keyedCode = ref<KeyedCode | null>(null);
  const editorType = ref<EditorType>("shader");
  const markers = computed<Marker[]>(() => {
    if (openedFileName.value === null) return [];
    const messages = errorMessages.get(openedFileName.value) ?? [];

    return messages.map((message: any) => {
      const startColumn = message.location?.line_position ?? 1;
      const endColumn = startColumn + (message.location?.length ?? 1);
      // TODO: Translate to UTF-16 ^^^
      const lineNumber = message.location?.line_number ?? 1;
      return {
        message: message.message,
        startLineNumber: lineNumber,
        startColumn,
        endColumn,
        endLineNumber: lineNumber,
        severity:
          message.message_type === "Error"
            ? MarkerSeverity.Error
            : message.message_type === "Warning"
              ? MarkerSeverity.Warning
              : MarkerSeverity.Info,
      } satisfies Marker;
    });
  });

  watchImmediate(openedFileName, (fileName) => {
    if (fileName === null) {
      keyedCode.value = null;
      return;
    }

    const id = crypto.randomUUID();
    keyedCode.value = {
      id,
      code: "",
      name: fileName,
    };

    if (fileName.endsWith(".wgsl")) {
      editorType.value = "shader";
    } else if (fileName.endsWith(".graph")) {
      editorType.value = "graph";
    } else {
      editorType.value = "shader";
    }

    // And now asynchronously load the file
    let file = fs.readTextFile(fileName);
    if (file === null) {
      showFileError("Could not read file", fileName);
      return;
    }
    file.then((v) => {
      if (keyedCode.value?.id !== id) {
        // We already opened another file
        return;
      }
      keyedCode.value = {
        id: crypto.randomUUID(),
        code: v,
        name: fileName,
      };
    });
  });

  const isReadonly = computed(() => {
    if (keyedCode.value === null) {
      return true;
    }
    return keyedCode.value.name.endsWith(".graph.wgsl");
  });

  function openFile(v: FilePath) {
    openedFileName.value = v;
  }
  function addFiles(files: Set<FilePath>) {
    files.forEach((file) => {
      if (fs.hasFile(file)) return;
      fs.writeTextFile(file, "");
    });
  }
  function renameFile(oldName: FilePath, newName: FilePath) {
    if (oldName === newName) return;
    fs.renameFile(oldName, newName);
    if (oldName === openedFileName.value) {
      openFile(newName);
    }
  }

  function deleteFiles(files: Set<FilePath>) {
    files.forEach((file) => {
      fs.deleteFile(file);
      if (file === openedFileName.value) {
        openedFileName.value = null;
      }
    });
  }

  fs.watchFromStart((change) => {
    if (change.type === "remove") {
      if (change.key === openedFileName.value) {
        openedFileName.value = null;
      }
    }
  });

  const setNewCode = useDebounceFn((newCode: () => string) => {
    const value = newCode();
    if (keyedCode.value === null) {
      showError("No file selected");
      return;
    }
    // Keeps the ID intact, but updates the code
    // This ID scheme is used to avoid triggering recursive updates (the CodeEditor has a copy of the code)
    keyedCode.value = {
      ...keyedCode.value,
      code: value,
    };
    fs.writeTextFile(keyedCode.value.name, value);
  }, 500);

  return {
    path: computed(() => openedFileName.value),
    editorType: computed(() => editorType.value),
    isReadonly,
    code: computed(() => keyedCode.value),
    markers,
    openFile,
    addFiles,
    renameFile,
    deleteFiles,
    setNewCode,
  };
}
