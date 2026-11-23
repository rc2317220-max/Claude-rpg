#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

OUT="$PWD/.agent/artifacts/starling-android"
WORK="$PWD/.agent/android-runtime"
CACHE="$PWD/.agent/starling-cache"
PKG=io.github.juanluci.starling
mkdir -p "$OUT" "$WORK" "$CACHE"

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

echo "=== KVM / boot Android ==="
sudo chmod 666 /dev/kvm
adb kill-server || true
emulator @StarlingCI \
  -no-window -noaudio -no-boot-anim -no-snapshot -wipe-data \
  -gpu swiftshader_indirect -accel on -memory 4096 -cores 4 \
  -camera-back none -camera-front none \
  >"$OUT/emulator.log" 2>&1 &
EMU_PID=$!
trap 'kill "$EMU_PID" 2>/dev/null || true' EXIT

for i in $(seq 1 90); do
  adb devices | grep -q $'emulator-\tdevice' && break
  kill -0 "$EMU_PID" 2>/dev/null || { cat "$OUT/emulator.log"; exit 10; }
  sleep 2
done
adb wait-for-device
for i in $(seq 1 120); do
  [ "$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ] && break
  sleep 2
done
[ "$(adb shell getprop sys.boot_completed | tr -d '\r')" = "1" ] || exit 11
adb shell input keyevent 82 || true
sleep 2
adb root >/dev/null 2>&1 || true
adb wait-for-device
adb exec-out screencap -p > "$OUT/00-android-home.png"

echo "=== exact Starling 0.9.0 x86_64 ==="
APK="$WORK/starling-0.9.0-x86_64.apk"
curl -fL --retry 5 --retry-delay 2 \
  "https://github.com/starling-client/starling/releases/download/v0.9.0/starling-0.9.0-x86_64.apk" -o "$APK"
ACTUAL="$(sha256sum "$APK" | awk '{print $1}')"
echo "$ACTUAL" | tee "$OUT/starling-apk.sha256"
[ "$ACTUAL" = "7759e2f542e41ed60d4a87430df69565a3e9760ed5e2242e10661e5cae15a7b7" ]
adb install -r "$APK" | tee "$OUT/install.txt"
adb shell appops set "$PKG" MANAGE_EXTERNAL_STORAGE allow 2>/dev/null || true
adb shell pm grant "$PKG" android.permission.POST_NOTIFICATIONS 2>/dev/null || true

APPDATA="/data/user/0/$PKG"
APP_UID="$(adb shell dumpsys package "$PKG" | sed -n 's/.*userId=\([0-9][0-9]*\).*/\1/p' | head -1 | tr -d '\r')"
echo "app_uid=$APP_UID" | tee "$OUT/app-uid.txt"

echo "=== restore persisted Starling state, if any ==="
RESTORED=0
if [ -f "$CACHE/offline-started.marker" ] && [ -d "$CACHE/appdata" ] && find "$CACHE/appdata" -type f -print -quit | grep -q .; then
  echo "fresh app-data context before restore:"
  adb shell "ls -Zd '$APPDATA' '$APPDATA/files' 2>/dev/null || true"
  tar -C "$CACHE/appdata" -cf - . | adb shell "mkdir -p '$APPDATA'; tar -C '$APPDATA' -xf -"
  adb shell "chown -R $APP_UID:$APP_UID '$APPDATA'; chmod 700 '$APPDATA'; restorecon -RF '$APPDATA' 2>/dev/null || true"
  echo "app-data context after restorecon:"
  adb shell "ls -Zd '$APPDATA' '$APPDATA/files' '$APPDATA/cache' 2>/dev/null || true"
  adb shell "ls -ld '$APPDATA' '$APPDATA/files' '$APPDATA/cache' 2>/dev/null || true"
  RESTORED=1
  echo "restored persisted app data with Android ownership/SELinux context"
fi

persist_state() {
  echo "=== persist Starling app state ==="
  rm -rf "$CACHE/appdata.new"
  mkdir -p "$CACHE/appdata.new"
  adb root >/dev/null 2>&1 || true
  adb wait-for-device >/dev/null 2>&1 || true
  if adb shell "test -d '$APPDATA'"; then
    adb exec-out "tar -C '$APPDATA' -cf - ." | tar -C "$CACHE/appdata.new" -xf - || true
    if find "$CACHE/appdata.new" -type f -print -quit | grep -q .; then
      rm -rf "$CACHE/appdata"
      mv "$CACHE/appdata.new" "$CACHE/appdata"
    fi
  fi
  du -sh "$CACHE/appdata" 2>/dev/null | tee "$OUT/cache-size.txt" || true
}

echo "=== launch Starling ==="
adb logcat -c
adb shell am force-stop "$PKG" || true
adb shell am start -W -n "$PKG/$PKG.LauncherActivity" | tee "$OUT/am-start.txt"
sleep 20
adb exec-out screencap -p > "$OUT/01-launch.png"

if [ "$RESTORED" = "0" ]; then
  echo "=== first-run launcher setup ==="
  adb shell input tap 1700 500
  sleep 3
  adb exec-out screencap -p > "$OUT/02-launcher-clean.png"

  adb shell input tap 820 910
  sleep 4
  adb exec-out screencap -p > "$OUT/03-target-choice.png"

  # 'Game' in the first-run launcher target dialog.
  adb shell input tap 1530 650
  sleep 10
  adb exec-out screencap -p > "$OUT/04-game-setup-start.png"

  echo "=== offline profile ==="
  # Step::Start lower-right = Play offline.
  adb shell input tap 1434 656
  sleep 4
  adb exec-out screencap -p > "$OUT/05-offline-confirm.png"

  # Step::Offline primary right = confirm Play offline.
  adb shell input tap 1382 617
  sleep 4
  adb exec-out screencap -p > "$OUT/06-offline-name-form.png"

  # Offline form: player name, then Use profile.
  adb shell input tap 1200 510
  adb shell input text RustPGTest
  sleep 2
  adb shell input keyevent 4 || true
  sleep 1
  adb exec-out screencap -p > "$OUT/07-offline-name-filled.png"
  adb shell input tap 1387 650
  touch "$CACHE/offline-started.marker"
  sleep 8
  adb exec-out screencap -p > "$OUT/08-download-start.png"
else
  echo "=== restored state: allow Starling to resume ==="
  sleep 8
  adb exec-out screencap -p > "$OUT/08-restored-state.png"
fi

echo "=== monitor download / preparation ==="
READY=""
for i in $(seq -w 1 40); do
  ROOT="$APPDATA/files"
  adb shell "find '$ROOT' -name horizon-resources.json -o -name facts.json 2>/dev/null | head -30" > "$OUT/progress-$i.resources" || true
  if grep -q 'horizon-resources.json' "$OUT/progress-$i.resources"; then
    READY=1
    touch "$CACHE/resources-ready.marker"
    echo "prepared resources detected at poll $i"
    break
  fi
  adb exec-out screencap -p > "$OUT/progress-$i.png" || true
  adb logcat -d -v brief | grep -Ei 'starling|download|prepar|mojang|java|error|panic' | tail -220 > "$OUT/progress-$i.log" || true
  sleep 30
done

adb exec-out screencap -p > "$OUT/10-after-preparation.png" || true
adb shell "du -sh '$APPDATA/files' 2>/dev/null" | tee "$OUT/private-size.txt" || true
adb logcat -d -v threadtime > "$OUT/logcat-prepared.txt" || true

if [ -z "$READY" ]; then
  echo "prepared resources not ready in this run; saving resumable state"
  persist_state
  exit 20
fi

echo "=== build current RustPG hmod and stage import inbox ==="
python3 scripts/package_native.py
test -s releases/RustPG.hmod
cp releases/RustPG.hmod "$OUT/RustPG.hmod"
sha256sum "$OUT/RustPG.hmod" > "$OUT/RustPG.hmod.sha256"

adb shell "mkdir -p '$APPDATA/files/import-inbox'"
adb push releases/RustPG.hmod /data/local/tmp/RustPG.hmod
adb shell "cp /data/local/tmp/RustPG.hmod '$APPDATA/files/import-inbox/RustPG.hmod'; chown -R $APP_UID:$APP_UID '$APPDATA/files/import-inbox'; rm -f /data/local/tmp/RustPG.hmod"
adb shell "ls -lh '$APPDATA/files/import-inbox'" > "$OUT/import-inbox-before-game.txt" || true

echo "=== enable Agent Control in both possible Android game homes ==="
PRIVATE_EXTRAS="$APPDATA/files/horizon-extras.txt"
SHARED_HOME="/sdcard/Starling"
adb shell "mkdir -p '$SHARED_HOME'"
adb shell "(grep -v '^agentControl:' '$PRIVATE_EXTRAS' 2>/dev/null || true) > '$PRIVATE_EXTRAS.tmp'; echo 'agentControl:true' >> '$PRIVATE_EXTRAS.tmp'; mv '$PRIVATE_EXTRAS.tmp' '$PRIVATE_EXTRAS'; chown $APP_UID:$APP_UID '$PRIVATE_EXTRAS'"
adb shell "(grep -v '^agentControl:' '$SHARED_HOME/horizon-extras.txt' 2>/dev/null || true) > '$SHARED_HOME/horizon-extras.tmp'; echo 'agentControl:true' >> '$SHARED_HOME/horizon-extras.tmp'; mv '$SHARED_HOME/horizon-extras.tmp' '$SHARED_HOME/horizon-extras.txt'"
adb shell "cat '$PRIVATE_EXTRAS'" > "$OUT/horizon-extras-private.txt" || true
adb shell "cat '$SHARED_HOME/horizon-extras.txt'" > "$OUT/horizon-extras-shared.txt" || true

echo "=== finish onboarding: choose Game ==="
# 2400x1080 physical pixels. Real captured Game button: x=1209..1563, y=588..649.
adb exec-out screencap -p > "$OUT/11-ready-before-game.png"
adb shell input tap 1386 618
sleep 8
adb exec-out screencap -p > "$OUT/12-after-game-choice.png"

echo "=== wait for title/game and inbox processing ==="
for i in $(seq -w 1 30); do
  sleep 3
  adb exec-out screencap -p > "$OUT/game-wait-$i.png" || true
  adb shell "find '$APPDATA/files' /sdcard/Starling -maxdepth 5 \\( -iname '*rustpg*' -o -iname '*.hmod' -o -iname 'horizon-control.json' \\) 2>/dev/null | head -200" > "$OUT/game-wait-$i.paths" || true
  if adb shell "test -s '$APPDATA/files/horizon-control.json' || test -s '$SHARED_HOME/horizon-control.json'"; then
    echo "Agent Control endpoint appeared at wait $i"
    break
  fi
done

adb shell "find '$APPDATA/files' /sdcard/Starling -maxdepth 7 \\( -iname '*rustpg*' -o -iname '*.hmod' -o -iname 'horizon-control.json' -o -iname 'horizon-extras.txt' \\) 2>/dev/null | head -400" > "$OUT/runtime-paths.txt" || true
adb shell "ls -la '$APPDATA/files/import-inbox' 2>/dev/null" > "$OUT/import-inbox-after-game.txt" || true
adb logcat -d -v threadtime > "$OUT/logcat-after-game.txt" || true
adb exec-out screencap -p > "$OUT/13-game-current.png" || true

echo "=== locate real Agent Control endpoint ==="
ENDPOINT=""
for candidate in "$APPDATA/files/horizon-control.json" "$SHARED_HOME/horizon-control.json"; do
  if adb shell "test -s '$candidate'"; then ENDPOINT="$candidate"; break; fi
done
if [ -z "$ENDPOINT" ]; then
  echo "Agent Control endpoint not found after choosing Game"
  grep -Ei 'agent.?control|horizon-control|RustPG|rustpg|hmod|native mod|import|error|failed|panic' "$OUT/logcat-after-game.txt" | tail -500 || true
  persist_state
  exit 30
fi

echo "endpoint=$ENDPOINT" | tee "$OUT/control-endpoint-path.txt"
adb shell "cat '$ENDPOINT'" | tr -d '\r' > "$OUT/horizon-control.json"
DEVICE_PORT="$(python3 -c 'import json; print(json.load(open(".agent/artifacts/starling-android/horizon-control.json"))["port"])')"
TOKEN="$(python3 -c 'import json; print(json.load(open(".agent/artifacts/starling-android/horizon-control.json"))["token"])')"
HOST_PORT="$(adb forward tcp:0 tcp:$DEVICE_PORT | tr -d '\r')"
echo "device_port=$DEVICE_PORT host_port=$HOST_PORT" | tee "$OUT/control-forward.txt"

echo "=== drive the actual Starling game through Agent Control ==="
python3 - "$HOST_PORT" "$TOKEN" "$OUT/control-session.jsonl" <<'PY'
import json,socket,sys,time
port=int(sys.argv[1]); token=sys.argv[2]; out=sys.argv[3]
s=socket.create_connection(("127.0.0.1",port),10)
s.settimeout(120)
f=s.makefile("rwb", buffering=0)
log=open(out,"w",encoding="utf-8")
def send(cmd, timeout=120):
    s.settimeout(timeout)
    f.write((cmd+"\n").encode())
    rep=f.readline().decode("utf-8","replace").strip()
    log.write(json.dumps({"command":cmd,"reply":rep},ensure_ascii=False)+"\n"); log.flush()
    print(cmd, "=>", rep[:1200])
    return rep
send("auth "+token,20)
send("hello",20)
send("status",30)
send("ui",30)
send("open RustPG_Agent_Lab 424242 creative",120)
time.sleep(20)
send("status",30)
send("mobs 64",30)
send("screenshot /sdcard/Download/starling-real-game.png",60)
# Probe whether RustPG is already active; keep the reply even if the entity is not registered yet.
send("summon rustpg:dusk_revenant",30)
send("mobs 64",30)
log.close()
PY

adb pull /sdcard/Download/starling-real-game.png "$OUT/14-agent-real-game.png" >/dev/null 2>&1 || true
adb exec-out screencap -p > "$OUT/15-device-final.png" || true
adb logcat -d -v threadtime > "$OUT/logcat-final.txt" || true
adb shell "find '$APPDATA/files' /sdcard/Starling -maxdepth 8 \\( -iname '*rustpg*' -o -iname '*.hmod' \\) 2>/dev/null | head -500" > "$OUT/rustpg-final-paths.txt" || true

persist_state

echo "=== FINAL STARLING RUNTIME SUMMARY ==="
cat "$OUT/private-size.txt" || true
cat "$OUT/rustpg-final-paths.txt" || true
cat "$OUT/control-session.jsonl" || true
test -s "$OUT/14-device-final.png"
echo "=== STARLING + RUSTPG REAL RUNTIME COMPLETE ==="
