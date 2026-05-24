#!/bin/bash
# Full build pipeline
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== AstraShell Full Build ==="
echo ""

# 1. Build Rust runtime
echo ">>> Building Rust runtime..."
bash "$SCRIPT_DIR/build-rust.sh" release

# 2. Build Android app
echo ""
echo ">>> Building Android app..."
cd "$SCRIPT_DIR/../android-app"
./gradlew assembleDebug

echo ""
echo "=== Build Complete ==="
echo "APK: android-app/app/build/outputs/apk/debug/app-debug.apk"
