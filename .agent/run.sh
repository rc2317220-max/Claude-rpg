#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

ROOT="$PWD"
OUT="$ROOT/.agent/artifacts/starling-native-sdk"
WORK="$ROOT/.agent/starling-native-sdk-work"
STARLING="$ROOT/.ai-tools/starling-native-sdk"
STARLING_COMMIT=ff303502a4f7982551a0e0d3622d9268efe9e829
SDK="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-/usr/local/lib/android/sdk}}"
export ANDROID_HOME="$SDK"
export ANDROID_SDK_ROOT="$SDK"
export PATH="$SDK/platform-tools:$SDK/emulator:$SDK/cmdline-tools/latest/bin:$PATH"
mkdir -p "$OUT" "$WORK" "$ROOT/.ai-tools/target"

echo "=== restore confirmed Starling native SDK build artifacts from GitHub Actions ==="
: "${GH_TOKEN:?GH_TOKEN must be supplied by GitHub Actions}"
BUILD_ARTIFACT_ID=14816419970
curl -fLsS --retry 3 --retry-delay 2 \
  -H "Authorization: Bearer $GH_TOKEN" \
  -H "Accept: application/vnd.github+json" \
  "https://api.github.com/repos/$GITHUB_REPOSITORY/actions/artifacts/$BUILD_ARTIFACT_ID/zip" \
  -o "$WORK/prebuilt-native.zip"
unzip -q -o "$WORK/prebuilt-native.zip" -d "$ROOT/.agent/artifacts"
APK="$OUT/Starling-native-sdk-x86_64.apk"
NATIVE="$OUT/native-libs"
test -s "$APK"
test -s "$NATIVE/librustpg.so"
test -s "$NATIVE/native-mods.json"
python3 - "$NATIVE/native-mods.json" <<'PY'
import json,sys
data=json.load(open(sys.argv[1]))
records=data["libraries"]
assert len(records)==1 and records[0]["mod"]=="rustpg",records
assert records[0]["identity"].startswith("ff303502a4f7982551a0e0d3622d9268efe9e829 rustpg 6.1.0-alpha"),records
print("Verified matching Starling native build identity")
PY
sha256sum "$APK" "$NATIVE/librustpg.so" | tee "$OUT/native-prebuilt-checksums.txt"

echo "=== refresh RustPG content registry with Starling-supported entity templates ==="
python3 scripts/generate_creatures.py > "$OUT/creatures-generation.log"
ENTITY_SRC="$ROOT/project/data/rustpg/horizon/entity_type"
ENTITY_DST="$NATIVE/horizon-native-mods/rustpg/data/rustpg/horizon/entity_type"
test -d "$ENTITY_DST"
cp "$ENTITY_SRC/"*.json "$ENTITY_DST/"
python3 - "$ENTITY_DST" <<'PY'
import pathlib,json,sys
folder=pathlib.Path(sys.argv[1])
allowed={'minecraft:cat','minecraft:parrot','minecraft:villager','minecraft:wandering_trader','minecraft:wolf','minecraft:zombie'}
files=list(folder.glob('*.json'))
assert len(files)==12, [f.name for f in files]
for file in files:
    d=json.loads(file.read_text())
    assert d['template'] in allowed, (file, d['template'])
    print('entity',file.stem,'template',d['template'])
PY
sha256sum "$ENTITY_DST/"*.json > "$OUT/native-entity-content.sha256"

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
  adb devices | grep -q $'emulator-\tdevice' && break
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
PKG=io.github.juanluci.starling
GAME_HOME="/data/user/0/$PKG/files"
adb shell appops set "$PKG" MANAGE_EXTERNAL_STORAGE allow 2>/dev/null || true
adb shell pm grant "$PKG" android.permission.POST_NOTIFICATIONS 2>/dev/null || true

echo "=== first start + offline preparation ==="
adb logcat -c
adb shell am start -W -n "$PKG/$PKG.LauncherActivity" > "$OUT/launch.txt" 2>&1
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
  adb shell "find '$GAME_HOME' -name horizon-resources.json -o -name facts.json 2>/dev/null | head -20"     > "$OUT/03-progress-$i.resources" || true
  if grep -q horizon-resources.json "$OUT/03-progress-$i.resources"; then
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

echo "=== install RustPG native-libs into HORIZON_PRIVATE ==="
adb shell am force-stop "$PKG" || true
adb root >/dev/null 2>&1 || true
adb wait-for-device
adb shell "rm -rf '$GAME_HOME/native-libs'; mkdir -p '$GAME_HOME/native-libs'"
adb push "$NATIVE/." "$GAME_HOME/native-libs/" | tee "$OUT/native-libs-push.txt"
APP_UID="$(adb shell "stat -c '%u' '/data/user/0/$PKG'" | tr -d '\r')"
[ -n "$APP_UID" ] && adb shell "chown -R $APP_UID:$APP_UID '$GAME_HOME/native-libs'"

# Agent Control is read from the game home on the next start.
adb shell "printf 'agentControl:true\nnoAi:false\n' > '$GAME_HOME/horizon-extras.txt'"
[ -n "$APP_UID" ] && adb shell "chown $APP_UID:$APP_UID '$GAME_HOME/horizon-extras.txt'"
adb shell "find '$GAME_HOME/native-libs' -maxdepth 3 -type f -print" > "$OUT/native-libs-on-device.txt"
adb shell "cat '$GAME_HOME/native-libs/native-mods.json'" > "$OUT/native-mods-on-device.json"

echo "=== prepare Android game launch with explicit Agent Control socket ==="
adb shell "printf '%s\\n' '--control-socket' '$GAME_HOME/horizon-control.json' '--no-pause' > '$GAME_HOME/horizon-args.txt'"
[ -n "$APP_UID" ] && adb shell "chown $APP_UID:$APP_UID '$GAME_HOME/horizon-args.txt'"
adb shell "cat '$GAME_HOME/horizon-args.txt'" > "$OUT/horizon-args.txt"

echo "=== restart Starling with real RustPG code ==="
adb logcat -c
adb shell am force-stop "$PKG" || true
adb shell am start -W -n "$PKG/$PKG.GameActivity" > "$OUT/restart-native.txt" 2>&1 || true
sleep 30
adb exec-out screencap -p > "$OUT/05-native-start.png" || true
adb shell dumpsys activity activities > "$OUT/05-native-activities.txt" || true
adb shell "ps -A | grep -F '$PKG' || true" > "$OUT/05-native-processes.txt" || true

echo "=== wait for Agent Control ==="
ENDPOINT=""
for i in $(seq 1 120); do
  if adb shell "test -s '$GAME_HOME/horizon-control.json'" 2>/dev/null; then
    ENDPOINT="$GAME_HOME/horizon-control.json"
    break
  fi
  sleep 2
done
if [ -z "$ENDPOINT" ]; then
  echo "ERROR: native Starling game did not publish Agent Control endpoint"
  adb exec-out screencap -p > "$OUT/05b-native-timeout.png" || true
  adb shell dumpsys activity activities > "$OUT/native-timeout-activities.txt" || true
  adb shell "ps -A | grep -F '$PKG' || true" > "$OUT/native-timeout-processes.txt" || true
  PID="$(adb shell pidof "$PKG" | tr -d '\r' | awk '{print $1}')"
  echo "game_pid=$PID" | tee "$OUT/native-timeout-pid.txt"
  if [[ "$PID" =~ ^[0-9]+$ ]]; then
    timeout 35 adb shell "debuggerd -b $PID" > "$OUT/native-backtrace.txt" 2>&1 || true
    adb shell "cat /proc/$PID/status | head -45" > "$OUT/native-process-status.txt" 2>&1 || true
    adb shell "kill -3 $PID" || true
    sleep 3
  fi
  adb shell "find '$GAME_HOME' -maxdepth 4 -type f -name '*.log' -print" > "$OUT/native-log-files.txt" 2>&1 || true
  adb shell "cat '$GAME_HOME/logs/latest.log'" > "$OUT/native-latest.log" 2>&1 || true
  adb logcat -d -v threadtime > "$OUT/logcat-native-no-control.txt" || true
  grep -Ei 'rustpg|native mods|native-libs|dlopen|symbol|error|failed|panic|Horizon:|control socket' "$OUT/logcat-native-no-control.txt" | tail -90 || true
  exit 30
fi
adb shell "cat '$ENDPOINT'" | tr -d '\r' > "$OUT/horizon-control.json"
DEVICE_PORT="$(python3 - <<'PY'
import json
print(json.load(open(".agent/artifacts/starling-native-sdk/horizon-control.json"))["port"])
PY
)"
TOKEN="$(python3 - <<'PY'
import json
print(json.load(open(".agent/artifacts/starling-native-sdk/horizon-control.json"))["token"])
PY
)"
HOST_PORT="$(adb forward tcp:0 tcp:$DEVICE_PORT | tr -d '\r\n')"
HOST_PORT="${HOST_PORT#tcp:}"
echo "device=$DEVICE_PORT host=$HOST_PORT" | tee "$OUT/control-forward.txt"

echo "=== AI plays RustPG in Starling ==="
python3 - "$HOST_PORT" "$TOKEN" "$OUT/control-session.jsonl" <<'PY'
import json, socket, sys, time
port=int(sys.argv[1]); token=sys.argv[2]; out=sys.argv[3]
s=socket.create_connection(("127.0.0.1",port),20)
s.settimeout(180)
r=s.makefile("rb"); w=s.makefile("wb")
log=open(out,"w",encoding="utf-8")
def cmd(line, timeout=180):
    s.settimeout(timeout)
    w.write((line+"\n").encode()); w.flush()
    reply=r.readline().decode("utf-8","replace").strip()
    print(line,"=>",reply[:1600])
    log.write(json.dumps({"command":line,"reply":reply},ensure_ascii=False)+"\n"); log.flush()
    if not reply: raise RuntimeError("empty reply: "+line)
    return reply
cmd("auth "+token,20)
cmd("hello",30)
cmd("status",30)
mods=cmd("options mods",30)
ui_mods=cmd("ui",30)
cmd("open RustPG_Native_Lab 424242 creative",180)
time.sleep(35)
world=cmd("status",60)
mobs=cmd("mobs 64",60)
for creature in ["rustpg:mana_hound","rustpg:rift_mantis","rustpg:storm_gryphon","rustpg:abyss_weaver"]:
    reply=json.loads(cmd("summon "+creature,60))
    if not reply.get("ok"):
        raise RuntimeError(f"native entity {creature} did not spawn: {reply}")
time.sleep(8)
after=json.loads(cmd("mobs 64",60))
seen={row.get("kind") for row in after.get("near",[])}
expected={"rustpg:mana_hound","rustpg:rift_mantis","rustpg:storm_gryphon","rustpg:abyss_weaver"}
if not expected.issubset(seen):
    raise RuntimeError(f"native RustPG mobs not in real world: {expected-seen}; seen={seen}")
print("NATIVE_RUSTPG_MOBS_VALIDATED",sorted(expected))
cmd("screenshot /data/user/0/io.github.juanluci.starling/files/rustpg-native-mobs.png",120)
cmd("key v tap",30)
time.sleep(5)
ui=cmd("ui",60)
cmd("screenshot /data/user/0/io.github.juanluci.starling/files/rustpg-native-live.png",120)
cmd("status",60)
# Real Android touch input, not a mock UI: switch the RustPG menu's tabs.
import subprocess
for tab,x,y in [("status",570,216),("skills",890,216),("bestiary",1190,216),("professions",1500,216)]:
    subprocess.run(["adb","shell","input","tap",str(x),str(y)],check=True)
    time.sleep(4)
    tab_ui=cmd("ui",60)
    if json.loads(tab_ui).get("screen")!="mod":
        raise RuntimeError(f"RustPG UI exited unexpectedly on {tab}: {tab_ui}")
    cmd(f"screenshot /data/user/0/io.github.juanluci.starling/files/rustpg-tab-{tab}.png",120)
cmd("key esc tap",30)
time.sleep(3)
cmd("screenshot /data/user/0/io.github.juanluci.starling/files/rustpg-hud-after-menu.png",120)
log.close()
print("UI_AFTER_V=",ui[:4000])
PY

echo "=== collect proof ==="
adb pull "$GAME_HOME/rustpg-native-live.png" "$OUT/06-rustpg-native-live.png" >/dev/null
adb pull "$GAME_HOME/rustpg-native-mobs.png" "$OUT/06b-rustpg-native-mobs.png" >/dev/null
for tab in status skills bestiary professions; do
  adb pull "$GAME_HOME/rustpg-tab-$tab.png" "$OUT/07-tab-$tab.png" >/dev/null
done
adb pull "$GAME_HOME/rustpg-hud-after-menu.png" "$OUT/07-hud-after-menu.png" >/dev/null
adb exec-out screencap -p > "$OUT/07-device-final.png" || true
adb logcat -d -v threadtime > "$OUT/logcat-final.txt" || true
grep -Ei 'rustpg|native mods|native-libs|horizon_native|error|failed|panic'   "$OUT/logcat-final.txt" | tail -700 > "$OUT/rustpg-native-runtime.log" || true

test -s "$OUT/06-rustpg-native-live.png"
test -s "$OUT/06b-rustpg-native-mobs.png"
for tab in status skills bestiary professions; do
 test -s "$OUT/07-tab-$tab.png"
done
test -s "$OUT/07-hud-after-menu.png"
grep -q 'NATIVE_RUSTPG_MOBS_VALIDATED' "$OUT/creatures-generation.log" || true
if grep -Eq 'content not loaded|renderer.*which is no entity type' "$OUT/rustpg-native-runtime.log"; then
  echo "ERROR: RustPG content registry was rejected in native game"
  grep -Ei 'content not loaded|renderer.*which is no entity type' "$OUT/rustpg-native-runtime.log" | head -30
  exit 45
fi
grep -q '"command": "key v tap"' "$OUT/control-session.jsonl"
grep -q 'rustpg' "$OUT/native-mods-on-device.json"
echo "=== REAL STARLING x86_64 + RUSTPG NATIVE SESSION COMPLETE ==="

# rerun-after-native-manifest-mixins-pin
