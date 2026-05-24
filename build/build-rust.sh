#!/bin/bash
# AstraShell build system
# Builds the native Rust runtime for Android (aarch64)

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/.."

# Configuration
RUST_TARGET="aarch64-linux-android"
ANDROID_API_LEVEL=30
BUILD_TYPE="${1:-release}"

echo "=== AstraShell Build System ==="
echo "Target: $RUST_TARGET"
echo "API Level: $ANDROID_API_LEVEL"
echo "Build Type: $BUILD_TYPE"
echo ""

# Detect NDK
if [ -z "${ANDROID_NDK_HOME:-}" ]; then
    echo "ERROR: ANDROID_NDK_HOME not set"
    echo "Set it to your Android NDK path"
    exit 1
fi

# Setup Rust targets
echo "[1/4] Adding Rust target..."
rustup target add "$RUST_TARGET" 2>/dev/null || true

# Setup NDK linker
echo "[2/4] Configuring NDK linker..."
TOOLCHAIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64"
export CC_aarch64_linux_android="$TOOLCHAIN/bin/aarch64-linux-android${ANDROID_API_LEVEL}-clang"
export AR_aarch64_linux_android="$TOOLCHAIN/bin/llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$CC_aarch64_linux_android"

# Build
echo "[3/4] Building Rust runtime..."
cd "$PROJECT_ROOT/runtime"

BUILD_FLAGS=""
if [ "$BUILD_TYPE" = "release" ]; then
    BUILD_FLAGS="--release"
fi

cargo build --target "$RUST_TARGET" $BUILD_FLAGS

echo "[4/4] Copying artifacts..."
mkdir -p "$PROJECT_ROOT/android-app/app/src/main/jniLibs/arm64-v8a"
cp "target/$RUST_TARGET/${BUILD_TYPE}/libastrashell.so" \
   "$PROJECT_ROOT/android-app/app/src/main/jniLibs/arm64-v8a/"

echo ""
echo "=== Build Complete ==="
echo "Runtime library: android-app/app/src/main/jniLibs/arm64-v8a/libastrashell.so"
echo ""
echo "To build the Android app:"
echo "  cd android-app && ./gradlew assembleDebug"
