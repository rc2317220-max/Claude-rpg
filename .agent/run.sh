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

echo "=== create AVD ==="
rm -rf "$ANDROID_AVD_HOME/StarlingCI.avd" "$ANDROID_AVD_HOME/StarlingCI.ini"
echo no | avdmanager create avd --force --name StarlingCI \
  --package "system-images;android-35;google_apis;x86_64" \
  --device "pixel_6" --path "$ANDROID_AVD_HOME/StarlingCI.avd"
printf 'path=%s\npath.rel=avd/StarlingCI.avd\ntarget=android-35\n' "$ANDROID_AVD_HOME/StarlingCI.avd" > "$ANDROID_AVD_HOME/StarlingCI.ini"

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
adb root >/dev/null 2>&1 || true
adb wait-for-device

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
PKG=io.github.juanluci.starling
adb shell appops set "$PKG" MANAGE_EXTERNAL_STORAGE allow 2>/dev/null || true
adb shell pm grant "$PKG" android.permission.POST_NOTIFICATIONS 2>/dev/null || true

echo "=== launch Starling launcher ==="
adb logcat -c
adb shell am force-stop "$PKG" || true
adb shell am start -W -n "$PKG/$PKG.LauncherActivity" | tee "$OUT/am-start.txt"
sleep 20
adb exec-out screencap -p > "$OUT/01-starling-launch.png"

echo "=== dismiss Android fullscreen education ==="
adb shell input tap 1700 500
sleep 3
adb exec-out screencap -p > "$OUT/02-launcher-clean.png"

echo "=== open Set up Starling ==="
adb shell input tap 820 910
sleep 4
adb exec-out screencap -p > "$OUT/03-target-choice.png"

echo "=== choose Game ==="
adb shell input tap 1530 650
sleep 10
adb exec-out screencap -p > "$OUT/04-game-setup-start.png"

echo "=== choose Play offline ==="
# Step::Start: four buttons in a 2x2 grid. Offline is lower-right.
adb shell input tap 1440 660
sleep 4
adb exec-out screencap -p > "$OUT/05-offline-confirm.png"

echo "=== confirm Play offline ==="
# Step::Offline: primary action is the right-hand button.
adb shell input tap 1390 620
sleep 4
adb exec-out screencap -p > "$OUT/06-offline-name-form.png"

echo "=== create offline profile RustPGTest ==="
# Offline form geometry from account_form.rs at 2400x1080.
adb shell input tap 1200 510
sleep 1
adb shell input text RustPGTest
sleep 2
adb shell input keyevent 4 || true
sleep 1
adb exec-out screencap -p > "$OUT/07-offline-name-filled.png"
adb shell input tap 1390 653
sleep 8
adb exec-out screencap -p > "$OUT/08-download-start.png"

echo "=== monitor Starling download / prepare ==="
READY=""
for i in $(seq -w 1 22); do
  sleep 40
  adb exec-out screencap -p > "$OUT/09-progress-$i.png" || true
  adb logcat -d -v brief | grep -Ei 'starling|download|prepar|mojang|java|error|panic' | tail -220 > "$OUT/09-progress-$i.log" || true
  ROOT="/data/user/0/$PKG/files"
  adb shell "find '$ROOT' -name horizon-resources.json -o -name facts.json 2>/dev/null | head -20" > "$OUT/09-progress-$i.resources" || true
  if grep -q 'horizon-resources.json' "$OUT/09-progress-$i.resources"; then
    READY=1
    echo "prepared resources detected at progress $i"
    break
  fi
  if ! adb shell pidof "$PKG" >/dev/null 2>&1; then
    echo "Starling process stopped during preparation"
    break
  fi
done

echo "=== resource state ==="
adb shell "find /data/user/0/$PKG/files -maxdepth 5 -type f 2>/dev/null | head -400" > "$OUT/private-files.txt" || true
adb shell "du -sh /data/user/0/$PKG/files 2>/dev/null" | tee "$OUT/private-size.txt" || true
adb exec-out screencap -p > "$OUT/10-after-preparation.png" || true
adb logcat -d -v threadtime > "$OUT/logcat-full.txt" || true

if [ -z "$READY" ]; then
  echo "ERROR: prepared resources not detected"
  grep -Ei 'error|failed|panic|download|prepar|mojang|java' "$OUT/logcat-full.txt" | tail -400 || true
  exit 20
fi

echo "=== build current RustPG hmod ==="
python3 scripts/package_native.py
test -s releases/RustPG.hmod
cp releases/RustPG.hmod "$OUT/RustPG.hmod"
sha256sum "$OUT/RustPG.hmod" > "$OUT/RustPG.hmod.sha256"

echo "=== import RustPG.hmod into Starling ==="
adb push releases/RustPG.hmod /sdcard/Download/RustPG.hmod
adb shell am start -W \
  -a android.intent.action.VIEW \
  -d file:///sdcard/Download/RustPG.hmod \
  -t application/octet-stream \
  -n "$PKG/$PKG.LauncherActivity" \
  > "$OUT/hmod-import.txt" 2>&1 || true
sleep 12
adb exec-out screencap -p > "$OUT/11-after-rustpg-import.png" || true
adb shell "find /data/user/0/$PKG/files -iname '*rustpg*' -o -iname '*.hmod' 2>/dev/null | head -100" > "$OUT/rustpg-installed-paths.txt" || true
adb logcat -d -v threadtime > "$OUT/logcat-after-hmod.txt" || true

echo "=== finish Starling onboarding: choose Game ==="
# The real 2400x1080 Starling-ready screen: Game is the right-hand button.
adb shell input tap 1390 620
sleep 15
adb exec-out screencap -p > "$OUT/12-after-game-choice.png" || true

echo "=== force-enable RustPG + Agent Control for next game start ==="
GAME_HOME="/data/user/0/$PKG/files"
APP_UID="$(adb shell dumpsys package "$PKG" | sed -n 's/.*userId=\([0-9][0-9]*\).*/\1/p' | head -1 | tr -d '\r')"
echo "app_uid=$APP_UID" | tee "$OUT/app-uid.txt"

# Ensure the imported package is exactly where the game's mod loader reads it.
adb shell "mkdir -p '$GAME_HOME/horizon-mods'"
adb push releases/RustPG.hmod /data/local/tmp/RustPG.hmod >/dev/null
adb shell "cp /data/local/tmp/RustPG.hmod '$GAME_HOME/horizon-mods/RustPG.hmod'"

adb shell "(
  grep -v '^mod\\.rustpg:' '$GAME_HOME/horizon-mods.txt' 2>/dev/null || true
  echo 'mod.rustpg:true'
) > '$GAME_HOME/horizon-mods.txt.tmp'; mv '$GAME_HOME/horizon-mods.txt.tmp' '$GAME_HOME/horizon-mods.txt'"

adb shell "(
  grep -v '^agentControl:' '$GAME_HOME/horizon-extras.txt' 2>/dev/null || true
  grep -v '^noAi:' '$GAME_HOME/horizon-extras.txt' 2>/dev/null || true
  echo 'agentControl:true'
  echo 'noAi:false'
) > '$GAME_HOME/horizon-extras.txt.tmp'; mv '$GAME_HOME/horizon-extras.txt.tmp' '$GAME_HOME/horizon-extras.txt'"

if [ -n "$APP_UID" ]; then
  adb shell "chown -R $APP_UID:$APP_UID '$GAME_HOME/horizon-mods' '$GAME_HOME/horizon-mods.txt' '$GAME_HOME/horizon-extras.txt'"
fi
adb shell "cat '$GAME_HOME/horizon-mods.txt'" | tr -d '\r' | tee "$OUT/horizon-mods.txt"
adb shell "cat '$GAME_HOME/horizon-extras.txt'" | tr -d '\r' | tee "$OUT/horizon-extras.txt"
adb shell "find '$GAME_HOME/horizon-mods' -maxdepth 2 -type f -ls" > "$OUT/rustpg-installed-paths.txt" || true

echo "=== force Agent Control through Starling's Android launch arguments ==="
CONTROL_PATH="$GAME_HOME/horizon-control.json"
ARGS_PATH="$GAME_HOME/horizon-args.txt"
adb shell "printf '%s\\n' '--control-socket' '$CONTROL_PATH' '--no-pause' > '$ARGS_PATH'"
if [ -n "$APP_UID" ]; then
  adb shell "chown $APP_UID:$APP_UID '$ARGS_PATH' '$GAME_HOME/horizon-mods.txt' '$GAME_HOME/horizon-extras.txt' 2>/dev/null || true"
fi
adb shell "cat '$ARGS_PATH'" | tr -d '\r' | tee "$OUT/horizon-args.txt"

echo "=== restart the real Starling game with RustPG enabled ==="
adb logcat -c
adb shell am force-stop "$PKG" || true
adb shell am start -W -n "$PKG/$PKG.LauncherActivity" > "$OUT/restart.txt" 2>&1 || true
sleep 25
adb exec-out screencap -p > "$OUT/13-restarted-starling.png" || true
adb shell dumpsys activity activities > "$OUT/13-activities.txt" || true

echo "=== wait for Android Agent Control endpoint ==="
ENDPOINT=""
for i in $(seq 1 120); do
  for p in "$CONTROL_PATH" "$GAME_HOME/horizon-control.json" "/sdcard/Starling/horizon-control.json"; do
    if adb shell "test -s '$p'" 2>/dev/null; then ENDPOINT="$p"; break 2; fi
  done
  sleep 2
done
if [ -z "$ENDPOINT" ]; then
  echo "ERROR: Agent Control endpoint not created"
  adb shell "cat '$ARGS_PATH'" > "$OUT/horizon-args-device.txt" 2>&1 || true
  adb shell "cat '$GAME_HOME/horizon-extras.txt'" > "$OUT/horizon-extras-device.txt" 2>&1 || true
  adb shell "cat '$GAME_HOME/horizon-mods.txt'" > "$OUT/horizon-mods-device.txt" 2>&1 || true
  adb shell "find '$GAME_HOME' -maxdepth 3 -type f 2>/dev/null | sort | head -500" > "$OUT/game-home-files.txt" || true
  adb logcat -d -v threadtime > "$OUT/logcat-no-control.txt" || true
  grep -Ei 'arguments|rustpg|control socket|agent|mod|error|failed|panic' "$OUT/logcat-no-control.txt" | tail -500 || true
  exit 30
fi

echo "$ENDPOINT" | tee "$OUT/control-endpoint-path.txt"
adb shell "cat '$ENDPOINT'" | tr -d '\r' > "$OUT/horizon-control.json"
cat "$OUT/horizon-control.json"
DEVICE_PORT="$(python3 - <<'PY'
import json
print(json.load(open(".agent/artifacts/starling-android/horizon-control.json"))["port"])
PY
)"
TOKEN="$(python3 - <<'PY'
import json
print(json.load(open(".agent/artifacts/starling-android/horizon-control.json"))["token"])
PY
)"
HOST_PORT="$(adb forward tcp:0 tcp:$DEVICE_PORT | tr -d '\r\n')"
HOST_PORT="${HOST_PORT#tcp:}"
echo "device_port=$DEVICE_PORT host_port=$HOST_PORT" | tee "$OUT/control-forward.txt"

echo "=== control Starling itself: verify RustPG, create world, play ==="
python3 - "$HOST_PORT" "$TOKEN" "$OUT/control-session.jsonl" <<'PY'
import json, socket, sys, time
port=int(sys.argv[1]); token=sys.argv[2]; out=sys.argv[3]
s=socket.create_connection(("127.0.0.1",port),20)
s.settimeout(180)
r=s.makefile("rb")
w=s.makefile("wb")
log=open(out,"w",encoding="utf-8")
def cmd(line, timeout=180):
    s.settimeout(timeout)
    w.write((line+"\n").encode()); w.flush()
    reply=r.readline().decode("utf-8","replace").strip()
    log.write(json.dumps({"command":line,"reply":reply},ensure_ascii=False)+"\n"); log.flush()
    print(line, "=>", reply[:1200])
    if not reply:
        raise RuntimeError("empty reply for "+line)
    return reply

cmd("auth "+token,20)
cmd("hello",30)
cmd("status",30)
cmd("options mods",30)
cmd("ui",30)
cmd("open RustPG_Agent_Lab 424242 creative",180)
time.sleep(30)
cmd("status",60)
cmd("mobs 64",60)
cmd("key v tap",30)
time.sleep(5)
cmd("ui",60)
cmd("screenshot /data/user/0/io.github.juanluci.starling/files/rustpg-live-ui.png",120)
cmd("status",60)
log.close()
PY

echo "=== collect real in-game evidence ==="
adb pull "$GAME_HOME/rustpg-live-ui.png" "$OUT/14-rustpg-live-ui.png" >/dev/null 2>&1 || true
adb exec-out screencap -p > "$OUT/15-device-final.png" || true
adb logcat -d -v threadtime > "$OUT/logcat-final.txt" || true
grep -Ei 'rustpg|mod.*rustpg|control socket|error|failed|panic' "$OUT/logcat-final.txt" | tail -500 > "$OUT/rustpg-runtime-log.txt" || true

echo "=== final summary ==="
cat "$OUT/private-size.txt" || true
cat "$OUT/horizon-mods.txt" || true
cat "$OUT/horizon-extras.txt" || true
cat "$OUT/rustpg-installed-paths.txt" || true
cat "$OUT/control-session.jsonl" || true
cat "$OUT/rustpg-runtime-log.txt" || true

test -s "$OUT/14-rustpg-live-ui.png"
grep -q '"command": "open RustPG_Agent_Lab 424242 creative"' "$OUT/control-session.jsonl"
grep -q '"command": "key v tap"' "$OUT/control-session.jsonl"
echo "=== STARLING + RUSTPG REAL PLAY SESSION COMPLETE ==="
