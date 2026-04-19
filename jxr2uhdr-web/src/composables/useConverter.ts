import { ref, shallowRef } from "vue";
import { convertJxrToUltraHdr } from "./useWasm";
import type { ConvertResult } from "./useWasm";

const converting = ref(false);
const error = ref<string | null>(null);
const result = shallowRef<ConvertResult | null>(null);
const previewUrl = ref<string | null>(null);
const fileName = ref<string | null>(null);
const quality = ref(90);

function revokePreviewUrl() {
  if (previewUrl.value) {
    URL.revokeObjectURL(previewUrl.value);
    previewUrl.value = null;
  }
}

export function useConverter() {
  async function convert(file: File) {
    converting.value = true;
    error.value = null;
    revokePreviewUrl();
    result.value = null;
    fileName.value = file.name;

    try {
      const buffer = await file.arrayBuffer();
      const jxrBytes = new Uint8Array(buffer);

      const convertResult = await convertJxrToUltraHdr(
        jxrBytes,
        quality.value,
      );

      result.value = convertResult;

      const blob = new Blob([convertResult.jpeg.buffer as ArrayBuffer], {
        type: "image/jpeg",
      });
      previewUrl.value = URL.createObjectURL(blob);
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      converting.value = false;
    }
  }

  function download() {
    if (!previewUrl.value || !fileName.value) return;
    const a = document.createElement("a");
    a.href = previewUrl.value;
    const baseName = fileName.value.replace(/\.jxr$/i, "");
    a.download = `${baseName}_ultrahdr.jpg`;
    a.click();
  }

  function reset() {
    revokePreviewUrl();
    result.value = null;
    error.value = null;
    fileName.value = null;
  }

  return {
    converting,
    error,
    result,
    previewUrl,
    fileName,
    quality,
    convert,
    download,
    reset,
  };
}
