#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"
echo "[build] npm run tauri:build:macos"
npm run tauri:build:macos

echo "[done] Check src-tauri/target/*/release/bundle for macOS artifacts."
