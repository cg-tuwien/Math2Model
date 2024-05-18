<script setup lang="ts">
import { useStore } from "@/stores/store";
import type { DropdownOption } from "naive-ui/es/dropdown/src/interface";
import { computed, h, ref } from "vue";
import { RouterLink } from "vue-router";
import IconMoon from "~icons/mdi/moon-and-stars";
import IconSun from "~icons/mdi/white-balance-sunny";
import { assertUnreachable } from "@stefnotch/typestef/assert";

const store = useStore();

type MyDropdownOption = DropdownOption & {
  key: "switch-to-dark" | "switch-to-light";
};

const options = computed<MyDropdownOption[]>(() => {
  return [
    store.isDark
      ? {
          label: "Light",
          key: "switch-to-light",
          icon: () => h(IconSun),
        }
      : {
          label: "Dark",
          key: "switch-to-dark",
          icon: () => h(IconMoon),
        },
  ];
});

function handleHamburger(key: MyDropdownOption["key"]) {
  if (key === "switch-to-dark") {
    store.setIsDark(true);
  } else if (key === "switch-to-light") {
    store.setIsDark(false);
  } else {
    assertUnreachable(key);
  }
}
</script>

<template>
  <n-page-header class="px-2 border-b border-gray-200">
    <template #title>
      <RouterLink to="/" class="mx-2">Home</RouterLink>
      <RouterLink to="/wgpu" class="mx-2">Wgpu</RouterLink>
    </template>

    <template #extra>
      <n-dropdown
        :options="options"
        trigger="click"
        placement="bottom-start"
        @select="handleHamburger"
      >
        <n-button :bordered="false" style="padding: 0 4px">
          <mdi-hamburger-menu></mdi-hamburger-menu>
        </n-button>
      </n-dropdown>
    </template>
  </n-page-header>
</template>

<style scoped></style>
