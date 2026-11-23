#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

OUT="$PWD/.agent/artifacts/starling-android"
WORK="$PWD/.agent/android-runtime"
mkdir -p "$OUT" "$WORK"

SDK="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-/usr/local/lib/android/sdk}}"
export ANDROID_SDK_ROOT="$SDK"
export ANDROID_HOME="$SDK"
export PATH="$SDK/platform-tools:$SDK/emulator:$SDK/cmdline-tools/latest/bin:$PATH"
export ANDROID_USER_HOME="$WORK/user"
export ANDROID_AVD_HOME="$WORK/avd"
mkdir -p "$ANDROID_USER_HOME" "$ANDROID_AVD_HOME"

echo "=== install emulator packages ==="
yes | sdkmanager --licenses >/dev/null 2>&1 || true
sdkmanager "platform-tools" "emulator" "system-images;android-35;google_apis;x86_64"

echo "=== create explicit AVD ==="
rm -rf "$ANDROID_AVD_HOME/StarlingCI.avd" "$ANDROID_AVD_HOME/StarlingCI.ini"
echo no | avdmanager create avd --force --name StarlingCI \
  --package "system-images;android-35;google_apis;x86_64" \
  --device "pixel_6" --path "$ANDROID_AVD_HOME/StarlingCI.avd"
printf 'path=%s\npath.rel=avd/StarlingCI.avd\ntarget=android-35\n' "$ANDROID_AVD_HOME/StarlingCI.avd" > "$ANDROID_AVD_HOME/StarlingCI.ini"
echo "AVDs:"; emulator -list-avds
find "$ANDROID_AVD_HOME" -maxdepth 2 -type f | sort | head -80

echo "=== KVM ==="
sudo chmod 666 /dev/kvm
ls -l /dev/kvm

echo "=== boot Android ==="
adb kill-server || true
emulator @StarlingCI \
  -no-window -noaudio -no-boot-anim -no-snapshot -wipe-data \
  -gpu swiftshader_indirect -accel on -memory 4096 -cores 4 \
  -camera-back none -camera-front none \
  >"$OUT/emulator.log" 2>&1 &
EMU_PID=$!
trap 'kill "$EMU_PID" 2>/dev/null || true' EXIT

for i in $(seq 1 90); do
  if adb devices | grep -q $'emulator-\tdevice'; then break; fi
  if ! kill -0 "$EMU_PID" 2>/dev/null; then
    echo "emulator exited early"; cat "$OUT/emulator.log"; exit 10
  fi
  sleep 2
done
adb wait-for-device
for i in $(seq 1 120); do
  [ "$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ] && break
  sleep 2
done
[ "$(adb shell getprop sys.boot_completed | tr -d '\r')" = "1" ] || { cat "$OUT/emulator.log"; exit 11; }
adb shell input keyevent 82 || true
sleep 2

echo "=== Android facts ==="
adb shell getprop ro.build.version.release | tee "$OUT/android-version.txt"
adb shell getprop ro.product.cpu.abi | tee "$OUT/android-abi.txt"
adb shell pm list features | grep -Ei 'vulkan|opengl' | tee "$OUT/gpu-features.txt" || true
adb exec-out screencap -p > "$OUT/00-android-home.png"

echo "=== exact official Starling 0.9.0 x86_64 ==="
APK="$WORK/starling-0.9.0-x86_64.apk"
curl -fL --retry 5 --retry-delay 2 \
  "https://github.com/starling-client/starling/releases/download/v0.9.0/starling-0.9.0-x86_64.apk" -o "$APK"
ACTUAL="$(sha256sum "$APK" | awk '{print $1}')"
echo "$ACTUAL" | tee "$OUT/starling-apk.sha256"
[ "$ACTUAL" = "7759e2f542e41ed60d4a87430df69565a3e9760ed5e2242e10661e5cae15a7b7" ]
adb install -r "$APK" | tee "$OUT/install.txt"

echo "=== launch Starling ==="
adb logcat -c
adb shell am force-stop io.github.juanluci.starling || true
adb shell am start -W -n io.github.juanluci.starling/io.github.juanluci.starling.LauncherActivity | tee "$OUT/am-start.txt"
sleep 20

adb shell pidof io.github.juanluci.starling | tee "$OUT/pid.txt" || true
adb shell dumpsys activity activities > "$OUT/activities.txt" || true
adb shell dumpsys window > "$OUT/window.txt" || true
adb exec-out screencap -p > "$OUT/01-starling-launch.png"
adb logcat -d -v threadtime > "$OUT/logcat.txt" || true
adb shell 'find /sdcard -maxdepth 5 -type f 2>/dev/null | grep -Ei "starling|horizon|control" | head -200' > "$OUT/shared-files.txt" || true
adb shell 'find /sdcard/Android/data/io.github.juanluci.starling -maxdepth 6 -type f 2>/dev/null | head -300' > "$OUT/app-external-files.txt" || true

echo "=== launch summary ==="
grep -E 'mResumedActivity|topResumedActivity|io.github.juanluci.starling' "$OUT/activities.txt" | tail -40 || true
grep -Ei 'FATAL EXCEPTION|RustPanic|panic|Vulkan|EGL|Starling|horizon|offline|account|download' "$OUT/logcat.txt" | tail -240 || true

echo "=== success: Starling process/screenshot captured ==="
test -s "$OUT/01-starling-launch.png"
