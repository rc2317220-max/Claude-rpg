#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

OUT="$PWD/.agent/artifacts/starling-runtime-lab"
mkdir -p "$OUT"

echo "=== Android/KVM environment ==="
uname -a
ls -l /dev/kvm || true
echo "ANDROID_HOME=${ANDROID_HOME:-}"
echo "ANDROID_SDK_ROOT=${ANDROID_SDK_ROOT:-}"
SDK="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-/usr/local/lib/android/sdk}}"
export ANDROID_SDK_ROOT="$SDK"
export ANDROID_HOME="$SDK"
export PATH="$SDK/platform-tools:$SDK/emulator:$SDK/cmdline-tools/latest/bin:$SDK/cmdline-tools/bin:$PATH"
command -v sdkmanager
command -v avdmanager
command -v adb || true
command -v emulator || true
sdkmanager --version

echo "=== install Android x86_64 emulator image ==="
yes | sdkmanager --licenses >/dev/null 2>&1 || true
sdkmanager "platform-tools" "emulator" "system-images;android-35;google_apis;x86_64"

echo "=== create AVD ==="
rm -rf "$HOME/.android/avd/StarlingCI.avd" "$HOME/.android/avd/StarlingCI.ini"
echo "no" | avdmanager create avd --force --name StarlingCI --package "system-images;android-35;google_apis;x86_64" --device "pixel_6"
sudo chmod 666 /dev/kvm || true

echo "=== boot Android emulator ==="
adb kill-server || true
emulator -avd StarlingCI \
  -no-window -noaudio -no-boot-anim -no-snapshot -wipe-data \
  -gpu swiftshader_indirect -accel on -memory 4096 -cores 4 \
  -camera-back none -camera-front none \
  >"$OUT/emulator.log" 2>&1 &
EMU_PID=$!
trap 'kill "$EMU_PID" 2>/dev/null || true' EXIT

adb wait-for-device
for i in $(seq 1 180); do
  BOOT="$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')"
  if [ "$BOOT" = "1" ]; then break; fi
  sleep 2
done
[ "$(adb shell getprop sys.boot_completed | tr -d '\r')" = "1" ]
adb shell input keyevent 82 || true
sleep 2

echo "=== emulator facts ==="
adb shell getprop ro.build.version.release | tee "$OUT/android-version.txt"
adb shell getprop ro.product.cpu.abi | tee "$OUT/android-abi.txt"
adb shell getprop ro.hardware.vulkan || true
adb shell dumpsys SurfaceFlinger > "$OUT/surfaceflinger.txt" || true
adb shell pm list features | grep -Ei "vulkan|opengl" | tee "$OUT/gpu-features.txt" || true
adb exec-out screencap -p > "$OUT/android-home.png"

echo "=== download exact Starling 0.9.0 x86_64 release ==="
APK="$OUT/starling-0.9.0-x86_64.apk"
curl -fL --retry 5 --retry-delay 2 \
  "https://github.com/starling-client/starling/releases/download/v0.9.0/starling-0.9.0-x86_64.apk" \
  -o "$APK"
sha256sum "$APK" | tee "$OUT/starling.apk.sha256"
echo "7759e2f542e41ed60d4a87430df69565a3e9760ed5e2242e10661e5cae15a7b7  $APK" | sha256sum -c -

echo "=== install Starling ==="
adb install -r "$APK"

echo "=== launch Starling LauncherActivity ==="
adb logcat -c
set +e
adb shell am start -W -n io.github.juanluci.starling/io.github.juanluci.starling.LauncherActivity | tee "$OUT/am-start.txt"
START_RC=${PIPESTATUS[0]}
set -e
sleep 15
adb shell pidof io.github.juanluci.starling | tee "$OUT/pid.txt" || true
adb shell dumpsys activity activities > "$OUT/activities.txt" || true
adb shell dumpsys window > "$OUT/window.txt" || true
adb exec-out screencap -p > "$OUT/01-starling-launch.png" || true
adb logcat -d -v threadtime > "$OUT/logcat.txt" || true

echo "=== Starling launch summary ==="
echo "am_start_rc=$START_RC"
grep -E "mResumedActivity|topResumedActivity|io.github.juanluci.starling" "$OUT/activities.txt" | tail -20 || true
grep -Ei "FATAL EXCEPTION|RustPanic|panic|Vulkan|EGL|Starling|horizon" "$OUT/logcat.txt" | tail -120 || true

echo "=== runtime probe complete ==="
