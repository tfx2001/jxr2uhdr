#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
WEB_PUBLIC_DIR="$ROOT_DIR/jxr2uhdr-web/public/wasm"
TARGET_DIR="$ROOT_DIR/target/wasm32-unknown-emscripten/release"

echo "==> Building jxr2uhdr for wasm32-unknown-emscripten..."
cargo build --release --target wasm32-unknown-emscripten -p jxr2uhdr

echo "==> Copying Wasm artifacts to jxr2uhdr-web/public/wasm/..."
mkdir -p "$WEB_PUBLIC_DIR"

cp "$TARGET_DIR/jxr2uhdr.js" "$WEB_PUBLIC_DIR/jxr2uhdr.js"
cp "$TARGET_DIR/jxr2uhdr.wasm" "$WEB_PUBLIC_DIR/jxr2uhdr.wasm"

echo "==> Done. Artifacts:"
ls -lh "$WEB_PUBLIC_DIR"
