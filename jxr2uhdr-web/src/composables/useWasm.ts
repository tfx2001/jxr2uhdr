/**
 * Emscripten jxr2uhdr module types & loader.
 *
 * The exported C functions correspond to `#[no_mangle] extern "C"` in
 * `jxr2uhdr-cli/src/wasm.rs`.
 */

export interface Jxr2uhdrModule {
  _malloc(size: number): number;
  _free(ptr: number): void;
  _jxr_decode_jxr(ptr: number, len: number): number;
  _jxr_image_encode_ultra_hdr(handle: number, quality: number): number;
  _jxr_image_width(handle: number): number;
  _jxr_image_height(handle: number): number;
  _jxr_image_pixels_ptr(handle: number): number;
  _jxr_image_pixels_len(handle: number): number;
  _jxr_image_free(handle: number): void;
  _jxr_buffer_ptr(handle: number): number;
  _jxr_buffer_len(handle: number): number;
  _jxr_buffer_free(handle: number): void;
  _jxr_last_error_message_ptr(): number;
  _jxr_last_error_message_len(): number;
  HEAPU8: Uint8Array;
}

export interface Jxr2uhdrModuleFactory {
  (moduleOverrides?: { locateFile?: (path: string) => string }): Promise<Jxr2uhdrModule>;
}

export interface ConvertResult {
  /** Ultra HDR JPEG bytes */
  jpeg: Uint8Array;
  /** Decoded image width */
  width: number;
  /** Decoded image height */
  height: number;
}

let moduleInstance: Jxr2uhdrModule | null = null;
let initPromise: Promise<Jxr2uhdrModule> | null = null;

/**
 * Load and cache the Emscripten jxr2uhdr module.
 *
 * The generated `jxr2uhdr.js` glue script is loaded via a `<script>` tag
 * injection so that Vite / Rolldown does not try to bundle it at build time.
 * It must be placed at `public/wasm/jxr2uhdr.js`.
 */
export async function loadWasmModule(): Promise<Jxr2uhdrModule> {
  if (moduleInstance) return moduleInstance;
  if (initPromise) return initPromise;

  initPromise = new Promise<Jxr2uhdrModule>((resolve, reject) => {
    const globalName = "jxr2uhdrModule";

    // Check if already loaded
    const existing = (globalThis as Record<string, unknown>)[globalName];
    if (typeof existing === "function") {
      const factory = existing as Jxr2uhdrModuleFactory;
      factory({ locateFile: (p: string) => `/wasm/${p}` }).then(resolve, reject);
      return;
    }

    const script = document.createElement("script");
    script.src = "/wasm/jxr2uhdr.js";
    script.async = true;
    script.onload = () => {
      const factory = (globalThis as Record<string, unknown>)[globalName];
      if (typeof factory !== "function") {
        reject(new Error(`jxr2uhdr factory not found on globalThis.${globalName}`));
        return;
      }
      (factory as Jxr2uhdrModuleFactory)({
        locateFile: (p: string) => `/wasm/${p}`,
      }).then(resolve, reject);
    };
    script.onerror = () => reject(new Error("Failed to load /wasm/jxr2uhdr.js"));
    document.head.appendChild(script);
  }).catch((err) => {
    // Clear initPromise on failure to allow retry
    initPromise = null;
    throw err;
  });

  moduleInstance = await initPromise;
  return moduleInstance;
}

/**
 * Read the last error message from the Wasm module.
 */
export function getLastError(mod: Jxr2uhdrModule): string {
  const ptr = mod._jxr_last_error_message_ptr();
  const len = mod._jxr_last_error_message_len();
  if (!ptr || len === 0) return "Unknown error";
  return new TextDecoder().decode(mod.HEAPU8.slice(ptr, ptr + len));
}

/**
 * Copy a Uint8Array into the Wasm heap, returning the allocated pointer.
 * Caller is responsible for freeing via `mod._free(ptr)`.
 */
export function copyToHeap(mod: Jxr2uhdrModule, data: Uint8Array): number {
  const ptr = mod._malloc(data.length);
  if (!ptr) throw new Error("Wasm malloc failed");
  mod.HEAPU8.set(data, ptr);
  return ptr;
}

/**
 * Read bytes from the Wasm heap.
 */
export function readFromHeap(
  mod: Jxr2uhdrModule,
  ptr: number,
  len: number,
): Uint8Array {
  return new Uint8Array(mod.HEAPU8.slice(ptr, ptr + len));
}

/**
 * Convert a JXR file (as raw bytes) to an Ultra HDR JPEG.
 */
export async function convertJxrToUltraHdr(
  jxrBytes: Uint8Array,
  quality: number,
): Promise<ConvertResult> {
  const mod = await loadWasmModule();

  // 1. Copy JXR bytes into Wasm heap
  const inputPtr = copyToHeap(mod, jxrBytes);

  // 2. Decode JXR
  const imageHandle = mod._jxr_decode_jxr(inputPtr, jxrBytes.length);
  mod._free(inputPtr);

  if (!imageHandle) {
    throw new Error(`JXR decode failed: ${getLastError(mod)}`);
  }

  try {
    const width = mod._jxr_image_width(imageHandle);
    const height = mod._jxr_image_height(imageHandle);

    // 3. Encode to Ultra HDR
    const bufferHandle = mod._jxr_image_encode_ultra_hdr(imageHandle, quality);

    if (!bufferHandle) {
      throw new Error(`Ultra HDR encode failed: ${getLastError(mod)}`);
    }

    try {
      // 4. Read result
      const outPtr = mod._jxr_buffer_ptr(bufferHandle);
      const outLen = mod._jxr_buffer_len(bufferHandle);
      const jpeg = readFromHeap(mod, outPtr, outLen);

      return { jpeg, width, height };
    } finally {
      mod._jxr_buffer_free(bufferHandle);
    }
  } finally {
    mod._jxr_image_free(imageHandle);
  }
}
