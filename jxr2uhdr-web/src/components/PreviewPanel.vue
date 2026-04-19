<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { Icon } from "@iconify/vue";
import type { ConvertResult } from "../composables/useWasm";

const { t } = useI18n();

defineProps<{
  previewUrl: string | null;
  result: ConvertResult | null;
}>();

const fullscreen = ref(false);
</script>

<template>
  <div class="flex flex-col items-center gap-3">
    <div
      v-if="previewUrl"
      class="relative w-full cursor-zoom-in overflow-hidden rounded-lg border border-gray-200 dark:border-gray-700"
      @click="fullscreen = true"
    >
      <img
        :src="previewUrl"
        :alt="t('previewAlt')"
        class="mx-auto max-h-[60vh] w-auto object-contain"
      />
    </div>
    <p
      v-if="result"
      class="text-xs text-gray-500 dark:text-gray-400"
    >
      {{ t('imageInfo', { width: result.width, height: result.height, size: (result.jpeg.length / 1024).toFixed(1) }) }}
    </p>

    <!-- Fullscreen overlay -->
    <Teleport to="body">
      <div
        v-if="fullscreen && previewUrl"
        role="dialog"
        aria-modal="true"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/90 backdrop-blur-sm"
        @click="fullscreen = false"
        @keydown.escape="fullscreen = false"
      >
        <img
          :src="previewUrl"
          :alt="t('previewAltFull')"
          class="max-h-screen max-w-full object-contain"
        />
        <button
          class="absolute right-4 top-4 flex h-10 w-10 items-center justify-center rounded-full bg-white/10 text-white transition-colors hover:bg-white/20"
          :aria-label="t('closePreview')"
          @click.stop="fullscreen = false"
        >
          <Icon icon="mdi:close" class="text-2xl" />
        </button>
      </div>
    </Teleport>
  </div>
</template>
