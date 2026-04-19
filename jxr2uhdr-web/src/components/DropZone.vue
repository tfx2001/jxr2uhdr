<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { Icon } from "@iconify/vue";

const { t } = useI18n();

const emit = defineEmits<{
  (e: "select", file: File): void;
}>();

const dragging = ref(false);
const fileInput = ref<HTMLInputElement | null>(null);

function onDragOver(event: DragEvent) {
  event.preventDefault();
  dragging.value = true;
}

function onDragLeave() {
  dragging.value = false;
}

function onDrop(event: DragEvent) {
  event.preventDefault();
  dragging.value = false;
  const file = event.dataTransfer?.files[0];
  if (file && file.name.toLowerCase().endsWith(".jxr")) {
    emit("select", file);
  }
}

function onFileChange(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (file && file.name.toLowerCase().endsWith(".jxr")) {
    emit("select", file);
  }
  input.value = "";
}

function onKeyDown(event: KeyboardEvent) {
  if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    openFilePicker();
  }
}

function openFilePicker() {
  fileInput.value?.click();
}
</script>

<template>
  <div
    :class="[
      'flex flex-col items-center justify-center gap-3 rounded-xl border-2 border-dashed p-10 transition-colors cursor-pointer',
      dragging
        ? 'border-indigo-400 bg-indigo-50 dark:bg-indigo-950/30'
        : 'border-gray-300 dark:border-gray-600 hover:border-indigo-300 dark:hover:border-indigo-500',
    ]"
    tabindex="0"
    role="button"
    :aria-label="t('dropzone', { ext: '.jxr' })"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
    @drop="onDrop"
    @click="openFilePicker"
    @keydown="onKeyDown"
  >
    <Icon icon="mdi:cloud-upload-outline" class="text-4xl text-gray-400" />
    <p class="text-sm text-gray-500 dark:text-gray-400">
      {{ t('dropzone', { ext: '.jxr' }) }}
    </p>
    <input
      ref="fileInput"
      type="file"
      accept=".jxr"
      class="hidden"
      @change="onFileChange"
    />
  </div>
</template>
