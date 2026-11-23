#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

OUT="$PWD/.agent/artifacts/starling-runtime-lab"
SDK="${ANDROID_HOME:-/usr/local/lib/android/sdk}"
SDKMANAGER="$SDK/cmdline-tools/latest/bin/sdkmanager"
AVDMANAGER="$SDK/cmdline-tools/latest/bin/avdmanager"
ADB="$SDK/platform-tools/adb"
EMU="$SDK/emulator/emulator"
APK="$OUT/starling-0.9.0-x86_64.apk"
mkdir -p "$OUT"

echo "=== enable kvm ==="
sudo chmod 666 /dev/kvm
ls -l /dev/kvm

echo "=== install Android emulator + x86_64 image ==="
yes | "$SDKMANAGER" --licenses >/dev/null || true
"$SDKMANAGER" "emulator" "platform-tools" "system-images;android-35;google_apis;x86_64"

echo "=== create AVD ==="
echo no | "$AVDMANAGER" create avd -n starling-ci -k "system-images;android-35;google_apis;x86_64" --force
"$AVDMANAGER" list avd

echo "=== boot emulator ==="
"$EMU" -avd starling-ci   -no-window -no-audio -no-boot-anim -no-snapshot   -gpu swiftshader_indirect   -memory 4096 -cores 4   -camera-back none -camera-front none   >"$OUT/emulator.log" 2>&1 &
EMU_PID=$!
echo "$EMU_PID" > "$OUT/emulator.pid"

cleanup() {
  "$ADB" emu kill >/dev/null 2>&1 || true
  kill "$EMU_PID" >/dev/null 2>&1 || true
}
trap cleanup EXIT

"$ADB" wait-for-device
for i in $(seq 1 180); do
  BOOT=$("$ADB" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r' || true)
  [ "$BOOT" = "1" ] && break
  sleep 2
done
BOOT=$("$ADB" shell getprop sys.boot_completed | tr -d '\r')
echo "boot_completed=$BOOT"
[ "$BOOT" = "1" ]

echo "=== emulator info ==="
"$ADB" shell getprop ro.product.cpu.abi
"$ADB" shell getprop ro.build.version.release
"$ADB" shell getprop ro.build.version.sdk
"$ADB" shell dumpsys SurfaceFlinger 2>/dev/null | grep -i -E 'GLES|Vulkan' | head -30 || true

echo "=== download exact Starling v0.9.0 x86_64 release ==="
curl -L --fail --retry 3   https://github.com/starling-client/starling/releases/download/v0.9.0/starling-0.9.0-x86_64.apk   -o "$APK"
stat -c '%n %s bytes' "$APK"
test "$(stat -c %s "$APK")" = "53679480"
sha256sum "$APK" | tee "$OUT/starling-apk.sha256"

echo "=== install Starling ==="
"$ADB" install -r "$APK"
"$ADB" shell pm list packages | grep -i starling || true

PKG=$("$ADB" shell pm list packages | sed -n 's/^package://p' | grep -i starling | head -1 | tr -d '\r')
if [ -z "$PKG" ]; then
  PKG=io.github.juanluci.starling
fi
echo "package=$PKG" | tee "$OUT/package.txt"

# Give the app the broad storage access it expects for its shared Starling folder.
"$ADB" shell appops set "$PKG" MANAGE_EXTERNAL_STORAGE allow 2>/dev/null || true
"$ADB" shell pm grant "$PKG" android.permission.POST_NOTIFICATIONS 2>/dev/null || true

echo "=== launch Starling ==="
"$ADB" shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1
sleep 12

echo "=== capture first real screen ==="
"$ADB" exec-out screencap -p > "$OUT/01-starling-first-screen.png"
"$ADB" shell uiautomator dump /sdcard/window.xml >/dev/null 2>&1 || true
"$ADB" pull /sdcard/window.xml "$OUT/01-window.xml" >/dev/null 2>&1 || true
"$ADB" shell dumpsys window windows > "$OUT/window-dump.txt" || true
"$ADB" logcat -d -v threadtime > "$OUT/logcat.txt" || true

file "$OUT/01-starling-first-screen.png"
ls -lh "$OUT/01-starling-first-screen.png"
echo "=== starling process ==="
"$ADB" shell pidof "$PKG" || true
echo "=== done ==="
