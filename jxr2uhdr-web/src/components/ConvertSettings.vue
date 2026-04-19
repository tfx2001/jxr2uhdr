<script setup lang="ts">
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const quality = defineModel<number>({ required: true });
</script>

<template>
  <div class="flex items-center gap-4">
    <label for="quality" class="text-sm font-medium text-gray-700 dark:text-gray-300 whitespace-nowrap">
      {{ t('quality') }}
    </label>
    <input
      id="quality"
      v-model.number="quality"
      type="range"
      min="1"
      max="100"
      step="1"
      class="h-2 w-full cursor-pointer appearance-none rounded-lg bg-gray-200 accent-indigo-500 dark:bg-gray-700"
    />
    <span class="min-w-[3ch] text-right text-sm font-mono text-gray-600 dark:text-gray-400">
      {{ quality }}
    </span>
  </div>
</template>
