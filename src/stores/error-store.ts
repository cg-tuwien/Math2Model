import type { FilePath } from "@/filesystem/reactive-files";
import { sceneFilesPromise } from "@/globals";
import type { WasmCompilationMessage } from "parametric-renderer-core/pkg/web";
import { acceptHMRUpdate, defineStore } from "pinia";
import { computed, reactive } from "vue";

export const useErrorStore = defineStore("error-store", () => {
  const fs = sceneFilesPromise;
  const errors = reactive(new Map<FilePath, WasmCompilationMessage[]>());
  fs.then((v) => {
    v.watchFromStart((change) => {
      // Potential future task: Move the errors around on edits
      errors.delete(change.key);
    });
  });

  function setErrors(file: FilePath, error: WasmCompilationMessage[]) {
    if (error.length === 0) {
      errors.delete(file);
    } else {
      errors.set(file, error);
    }
  }

  function clearErrors(file: FilePath) {
    setErrors(file, []);
  }

  return {
    setErrors,
    clearErrors,
    errors: computed(() => errors),
  };
});
if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useErrorStore, import.meta.hot));
}
