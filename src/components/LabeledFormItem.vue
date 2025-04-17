<script setup lang="ts">
const props = defineProps<{
  tooltip?: string;
}>();
</script>
<template>
  <div>
    <span class="pr-1 mb-1 inline-block"><slot name="label"></slot></span>
    <n-tooltip trigger="hover" placement="bottom" v-if="props.tooltip">
      <template #trigger>
        <n-button text v-bind="$attrs" size="tiny">
          <template #icon> <mdi-info-outline /> </template>
        </n-button>
      </template>
      {{ props.tooltip }}
    </n-tooltip>
    <br />
    <slot></slot>
  </div>
</template>
