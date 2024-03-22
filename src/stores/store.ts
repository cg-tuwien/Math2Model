import { ref, computed } from "vue";
import { defineStore } from "pinia";
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
    isDark: computed(() => isDark.value),
    setIsDark: (newValue: boolean) => {
      isDark.value = newValue;
    },
  };
});
