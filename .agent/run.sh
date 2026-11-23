#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
mkdir -p .agent/artifacts/starling-runtime-lab

echo "=== android sdk probe ==="
echo "ANDROID_HOME=$ANDROID_HOME"
echo "ANDROID_SDK_ROOT=$ANDROID_SDK_ROOT"
command -v sdkmanager || true
command -v avdmanager || true
command -v emulator || true
command -v adb || true
find "${ANDROID_HOME:-/opt/none}" -maxdepth 3 \( -name sdkmanager -o -name avdmanager -o -name emulator -o -name adb \) -type f 2>/dev/null | sort | head -80 || true
ls -la "${ANDROID_HOME:-/opt/none}" 2>/dev/null | head -80 || true
ls -la /usr/local/lib/android/sdk 2>/dev/null | head -80 || true
echo "=== kvm ==="
ls -l /dev/kvm || true
id
