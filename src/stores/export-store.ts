import { acceptHMRUpdate, defineStore } from "pinia";
import { ref } from "vue";

export const useExportStore = defineStore("export-store", () => {
  const isExportMode = ref(false);

  return {
    isExportMode,
  };
});
if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useExportStore, import.meta.hot));
}
