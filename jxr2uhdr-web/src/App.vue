<script setup lang="ts">
import { ref } from "vue";
import DropZone from "./components/DropZone.vue";
import ConvertSettings from "./components/ConvertSettings.vue";
import PreviewPanel from "./components/PreviewPanel.vue";
import ActionBar from "./components/ActionBar.vue";
import { useConverter } from "./composables/useConverter";
import { setLocale } from "./i18n";
import { useI18n } from "vue-i18n";

const { t, locale } = useI18n();

const {
  converting,
  error,
  result,
  previewUrl,
  fileName,
  quality,
  convert,
  download,
  reset,
} = useConverter();

const selectedFile = ref<File | null>(null);

function onFileSelect(file: File) {
  selectedFile.value = file;
  convert(file);
}

function onConvert() {
  if (selectedFile.value) {
    convert(selectedFile.value);
  }
}

function onReset() {
  reset();
  selectedFile.value = null;
}

function toggleLocale() {
  const next = locale.value === "en" ? "zh-CN" : "en";
  setLocale(next);
}
</script>

<template>
  <div class="flex min-h-screen flex-col items-center bg-white dark:bg-gray-900">
    <header class="w-full border-b border-gray-200 dark:border-gray-700">
      <div class="mx-auto flex max-w-3xl items-center gap-3 px-6 py-4">
        <h1 class="text-lg font-semibold text-gray-900 dark:text-white">
          {{ t('title') }}
        </h1>
        <span class="rounded-full bg-indigo-100 px-2 py-0.5 text-xs font-medium text-indigo-700 dark:bg-indigo-900 dark:text-indigo-300">
          {{ t('wasm') }}
        </span>
        <div class="flex-1" />
        <a
          href="https://github.com/tfx2001/jxr2uhdr"
          target="_blank"
          rel="noopener noreferrer"
          class="inline-flex items-center"
        >
          <img
            alt="GitHub Repo stars"
            src="https://img.shields.io/github/stars/tfx2001/jxr2uhdr"
          />
        </a>
        <button
          class="rounded-md px-2 py-1 text-xs font-medium text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-700 dark:text-gray-400 dark:hover:bg-gray-800 dark:hover:text-gray-200"
          @click="toggleLocale"
        >
          {{ t('langSwitch') }}
        </button>
      </div>
    </header>

    <main class="mx-auto flex w-full max-w-3xl flex-1 flex-col gap-6 px-6 py-8">
      <!-- Upload -->
      <DropZone @select="onFileSelect" />

      <!-- Selected file info -->
      <p v-if="fileName" class="text-sm text-gray-600 dark:text-gray-400">
        {{ t('selected', { name: fileName }) }}
      </p>

      <!-- Error -->
      <div
        v-if="error"
        class="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-800 dark:bg-red-950/50 dark:text-red-400"
      >
        {{ error }}
      </div>

      <!-- Settings -->
      <ConvertSettings v-model="quality" />

      <!-- Actions -->
      <ActionBar
        :converting="converting"
        :has-result="!!result"
        @convert="onConvert"
        @download="download"
        @reset="onReset"
      />

      <!-- Preview -->
      <PreviewPanel :preview-url="previewUrl" :result="result" />
    </main>

    <footer class="w-full border-t border-gray-200 dark:border-gray-700">
      <p class="mx-auto max-w-3xl px-6 py-3 text-center text-xs text-gray-400 dark:text-gray-500">
        {{ t('footer') }}
      </p>
    </footer>
  </div>
</template>
