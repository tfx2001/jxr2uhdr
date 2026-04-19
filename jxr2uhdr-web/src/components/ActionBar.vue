<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { Icon } from "@iconify/vue";

const { t } = useI18n();

defineProps<{
  converting: boolean;
  hasResult: boolean;
}>();

const emit = defineEmits<{
  (e: "convert"): void;
  (e: "download"): void;
  (e: "reset"): void;
}>();
</script>

<template>
  <div class="flex items-center gap-3">
    <button
      :disabled="converting"
      class="rounded-lg bg-indigo-600 px-5 py-2 text-sm font-medium text-white transition-colors hover:bg-indigo-700 disabled:cursor-not-allowed disabled:opacity-50"
      @click="emit('convert')"
    >
      <span v-if="converting" class="flex items-center gap-2">
        <Icon icon="mdi:loading" class="animate-spin text-base" />
        {{ t('converting') }}
      </span>
      <span v-else>{{ t('convert') }}</span>
    </button>

    <button
      :disabled="!hasResult"
      class="rounded-lg border border-gray-300 bg-white px-5 py-2 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-200 dark:hover:bg-gray-700"
      @click="emit('download')"
    >
      {{ t('download') }}
    </button>

    <button
      :disabled="converting"
      class="rounded-lg border border-gray-300 bg-white px-5 py-2 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-200 dark:hover:bg-gray-700"
      @click="emit('reset')"
    >
      {{ t('reset') }}
    </button>
  </div>
</template>
