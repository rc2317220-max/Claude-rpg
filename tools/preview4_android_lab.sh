#!/usr/bin/env bash
# Preview-specific runner: no stale Horizon APK, package, endpoint or native library.
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
APK="$(realpath "$1")"
MODE="$2"
NATIVE="${3:-}"
OUT="$ROOT/.agent/artifacts/preview4-$MODE"
WORK="$RUNNER_TEMP/preview4-$MODE"
SDK="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-/usr/local/lib/android/sdk}}"
export ANDROID_HOME="$SDK" ANDROID_SDK_ROOT="$SDK"
export PATH="$SDK/platform-tools:$SDK/emulator:$SDK/cmdline-tools/latest/bin:$PATH"
mkdir -p "$OUT" "$WORK"
sha256sum "$APK" > "$OUT/apk.sha256"
if [ "$MODE" = baseline ]; then
  echo "c7cce458826e8a4614f88264f716a9a4469ae98a5e9bd052ae76ede81e425595  $APK" | sha256sum -c -
elif [ "$MODE" = native ]; then
  NATIVE="$(realpath "$NATIVE")"
  test -s "$NATIVE/librustpg.so"
  test -s "$NATIVE/native-mods.json"
  sha256sum "$NATIVE/librustpg.so" > "$OUT/mod.sha256"
else exit 2; fi
echo "=== install Android emulator runtime ==="
yes | sdkmanager --licenses >/dev/null 2>&1 || true
sdkmanager "platform-tools" "emulator" "system-images;android-35;google_apis;x86_64"

echo "=== create Android x86_64 AVD ==="
cd "$ROOT"
export ANDROID_USER_HOME="$WORK/android-user"
export ANDROID_AVD_HOME="$WORK/avd"
mkdir -p "$ANDROID_USER_HOME" "$ANDROID_AVD_HOME"
rm -rf "$ANDROID_AVD_HOME/StarlingNative.avd" "$ANDROID_AVD_HOME/StarlingNative.ini"
echo no | avdmanager create avd --force --name StarlingNative   --package "system-images;android-35;google_apis;x86_64"   --device "pixel_6" --path "$ANDROID_AVD_HOME/StarlingNative.avd"
printf 'path=%s\npath.rel=avd/StarlingNative.avd\ntarget=android-35\n'   "$ANDROID_AVD_HOME/StarlingNative.avd" > "$ANDROID_AVD_HOME/StarlingNative.ini"

sudo chmod 666 /dev/kvm
adb kill-server || true
emulator @StarlingNative   -no-window -noaudio -no-boot-anim -no-snapshot -wipe-data   -gpu swiftshader_indirect -accel on -memory 4096 -cores 4   -camera-back none -camera-front none   >"$OUT/emulator.log" 2>&1 &
EMU_PID=$!
trap 'kill "$EMU_PID" 2>/dev/null || true' EXIT
for i in $(seq 1 90); do
  adb devices | grep -Eq '^emulator-[0-9]+[[:space:]]+device' && break
  kill -0 "$EMU_PID" 2>/dev/null || { cat "$OUT/emulator.log"; exit 10; }
  sleep 2
done
adb wait-for-device
for i in $(seq 1 120); do
  [ "$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ] && break
  sleep 2
done
[ "$(adb shell getprop sys.boot_completed | tr -d '\r')" = "1" ] || { cat "$OUT/emulator.log"; exit 11; }
adb shell input keyevent 82 || true
adb root >/dev/null 2>&1 || true
adb wait-for-device

echo "=== install custom exact-commit Starling ==="
adb install -r "$APK" | tee "$OUT/install.txt"
PKG=io.github.juanluci.starling.preview
ACTIVITY=io.github.juanluci.starling
GAME_HOME="/data/user/0/$PKG/files"
adb shell appops set "$PKG" MANAGE_EXTERNAL_STORAGE allow 2>/dev/null || true
adb shell pm grant "$PKG" android.permission.POST_NOTIFICATIONS 2>/dev/null || true

echo "=== first start + offline preparation ==="
adb logcat -c
adb shell am start -W -n "$PKG/$ACTIVITY.LauncherActivity" > "$OUT/launch.txt" 2>&1
sleep 20
adb shell input tap 1700 500
sleep 2
adb shell input tap 820 910
sleep 4
adb shell input tap 1530 650
sleep 10
adb exec-out screencap -p > "$OUT/01-setup-start.png"

# Play offline -> confirm -> name -> save.
adb shell input tap 1440 660
sleep 4
adb shell input tap 1390 620
sleep 4
adb shell input tap 1200 510
sleep 1
adb shell input text RustPGNative
sleep 2
adb shell input keyevent 4 || true
sleep 1
adb shell input tap 1390 653
sleep 8
adb exec-out screencap -p > "$OUT/02-download-start.png"

READY=""
for i in $(seq -w 1 24); do
  sleep 35
  adb exec-out screencap -p > "$OUT/03-progress-$i.png" || true
  adb shell "find '$GAME_HOME' -name sunrise-resources.json -o -name facts.json 2>/dev/null | head -20"     > "$OUT/03-progress-$i.resources" || true
  if grep -q sunrise-resources.json "$OUT/03-progress-$i.resources"; then
    READY=1
    echo "prepared resources detected at $i"
    break
  fi
done
if [ -z "$READY" ]; then
  adb logcat -d -v threadtime > "$OUT/logcat-prepare-failure.txt" || true
  echo "ERROR: Starling did not finish offline preparation"
  exit 20
fi
adb exec-out screencap -p > "$OUT/04-starling-ready.png"

echo "=== finish Starling setup and explicitly choose the Game entry ==="
# First button is 'Go fishing' (preparation screen); the next is 'Game' (ready chooser).
adb shell input tap 1390 620
sleep 6
adb exec-out screencap -p > "$OUT/04b-game-choice.png" || true
adb shell input tap 1390 620
sleep 18
adb exec-out screencap -p > "$OUT/04c-after-game-selection.png" || true
adb shell dumpsys activity activities > "$OUT/04c-activity-stack.txt" || true
echo "=== select the ACTUAL 'Game' button on the second 'Starling is ready' screen ==="
# Coordinates are native 2400x1080, not downsampled preview coordinates.
# After 'Go fishing' the preparer takes ~18s to reveal Launcher / Game.
# The previous test missed this final choice and never published Agent Control.
adb shell input tap 1390 620
sleep 15
adb exec-out screencap -p > "$OUT/04d-after-ready-game-button.png" || true
adb shell dumpsys activity activities > "$OUT/04d-activity-stack.txt" || true


# Always preserve diagnostic evidence, even if Android onboarding or runtime fails.
trap 'adb exec-out screencap -p > "$OUT/final-device.png" || true; adb logcat -d -v threadtime > "$OUT/logcat.txt" || true; kill "$EMU_PID" 2>/dev/null || true' EXIT
adb shell am force-stop "$PKG"
adb root >/dev/null 2>&1 || true
adb wait-for-device
APP_UID="$(adb shell "stat -c '%u' '/data/user/0/$PKG'" | tr -d '\r')"
test -n "$APP_UID"
if [ "$MODE" = native ]; then
  adb shell "rm -rf '$GAME_HOME/native-libs'; mkdir -p '$GAME_HOME/native-libs'"
  adb push "$NATIVE/." "$GAME_HOME/native-libs/"
  adb shell "chown -R $APP_UID:$APP_UID '$GAME_HOME/native-libs'"
fi
adb shell "printf '%s\\n' '--control-socket' '$GAME_HOME/sunrise-control.json' '--no-pause' > '$GAME_HOME/sunrise-args.txt'"
adb shell "printf 'agentControl:true\\nnoAi:false\\n' > '$GAME_HOME/sunrise-extras.txt'"
adb shell "chown $APP_UID:$APP_UID '$GAME_HOME/sunrise-args.txt' '$GAME_HOME/sunrise-extras.txt'"
adb shell "rm -f '$GAME_HOME/sunrise-control.json'"
adb shell am start -W -n "$PKG/$ACTIVITY.GameActivity" > "$OUT/launch-game.txt"
for i in $(seq 1 120); do
  if adb shell "test -s '$GAME_HOME/sunrise-control.json'"; then break; fi
  sleep 2
done
adb shell "test -s '$GAME_HOME/sunrise-control.json'" || { echo "BLOCKED: Preview did not start Agent Control"; exit 30; }
args=()
if [ "$MODE" = baseline ]; then args+=(--baseline); fi
python3 tools/sunrise_functional_lab.py --endpoint "$GAME_HOME/sunrise-control.json" --device-home "$GAME_HOME" --out "$OUT" "${args[@]}"
