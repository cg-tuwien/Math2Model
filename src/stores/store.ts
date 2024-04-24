import { readonly } from "vue";
import { acceptHMRUpdate, defineStore } from "pinia";
import { useDark } from "@vueuse/core";

/**
 * The main store for the app
 */
export const useStore = defineStore("store", () => {
  // Changes the theme of the app and tells TailwindCSS about it
  const isDark = useDark({
    selector: "body",
  });

  return {
    isDark: readonly(isDark),
    setIsDark: (newValue: boolean) => {
      isDark.value = newValue;
    },
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useStore, import.meta.hot));
}
