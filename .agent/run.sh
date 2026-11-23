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

echo "=== build RustPG source package ==="
python3 scripts/package_native.py
test -s releases/RustPG.hmod
cp releases/RustPG.hmod "$OUT/RustPG-source.hmod"
unzip -p releases/RustPG.hmod horizon-mod.toml | tee "$OUT/rustpg-manifest.toml"
grep -q 'id = "rustpg"' "$OUT/rustpg-manifest.toml"

echo "=== fetch exact Starling 0.9.0 ==="
rm -rf "$STARLING"
git clone --filter=blob:none https://github.com/starling-client/starling.git "$STARLING"
git -C "$STARLING" fetch --depth=1 origin "$STARLING_COMMIT"
git -C "$STARLING" checkout --detach "$STARLING_COMMIT"
test "$(git -C "$STARLING" rev-parse HEAD)" = "$STARLING_COMMIT"

echo "=== exact Rust + Android toolchain ==="
rustup toolchain install 1.99.1 --profile minimal --component rust-src
rustup default 1.99.1
rustup target add x86_64-linux-android --toolchain 1.99.1 || true
if ! cargo ndk --version >/dev/null 2>&1; then
  cargo install cargo-ndk --locked
fi
yes | sdkmanager --licenses >/dev/null 2>&1 || true
sdkmanager   "platform-tools"   "emulator"   "platforms;android-36"   "build-tools;35.0.0"   "ndk;28.2.13676358"   "system-images;android-35;google_apis;x86_64"

rustc -vV | tee "$OUT/rustc.txt"
cargo ndk --version | tee "$OUT/cargo-ndk.txt"

echo "=== prepare Starling Android native SDK ==="
cd "$STARLING"
python3 tools/android_native_sdk.py prepare

echo "=== build Starling 0.9.0 x86_64 with dylib native SDK ==="
export RUSTFLAGS="-C prefer-dynamic"
python3 packaging/android/build_apk.py --abi x86_64 --profile quick
APK="$STARLING/target/android/starling-0.9.0-x86_64.apk"
test -s "$APK"
sha256sum "$APK" | tee "$OUT/starling-native-sdk-apk.sha256"
stat -c '%n %s bytes' "$APK" | tee "$OUT/starling-native-sdk-apk-size.txt"
unzip -l "$APK" | grep 'lib/x86_64/' | tee "$OUT/apk-native-libraries.txt"

echo "=== compile RustPG as real x86_64 Android native library ==="
NATIVE="$OUT/native-libs"
python3 tools/android_native_sdk.py mods "$NATIVE" "$ROOT/releases/RustPG.hmod" --profile quick --abi x86_64
test -s "$NATIVE/librustpg.so"
test -s "$NATIVE/native-mods.json"
cat "$NATIVE/native-mods.json" | tee "$OUT/native-mods-record.json"
sha256sum "$NATIVE/librustpg.so" | tee "$OUT/librustpg.so.sha256"
readelf -d "$NATIVE/librustpg.so" > "$OUT/librustpg-dynamic.txt" || true

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

echo "=== save startup target Game ==="
adb shell input tap 1390 620
sleep 12

echo "=== install RustPG native-libs into HORIZON_PRIVATE ==="
adb shell am force-stop "$PKG" || true
adb root >/dev/null 2>&1 || true
adb wait-for-device
adb shell "rm -rf '$GAME_HOME/native-libs'; mkdir -p '$GAME_HOME/native-libs'"
adb push "$NATIVE/." "$GAME_HOME/native-libs/" | tee "$OUT/native-libs-push.txt"
APP_UID="$(adb shell dumpsys package "$PKG" | sed -n 's/.*userId=\([0-9][0-9]*\).*/\1/p' | head -1 | tr -d '\r')"
[ -n "$APP_UID" ] && adb shell "chown -R $APP_UID:$APP_UID '$GAME_HOME/native-libs'"

# Agent Control is read from the game home on the next start.
adb shell "printf 'agentControl:true\nnoAi:false\n' > '$GAME_HOME/horizon-extras.txt'"
[ -n "$APP_UID" ] && adb shell "chown $APP_UID:$APP_UID '$GAME_HOME/horizon-extras.txt'"
adb shell "find '$GAME_HOME/native-libs' -maxdepth 3 -type f -ls" > "$OUT/native-libs-on-device.txt"
adb shell "cat '$GAME_HOME/native-libs/native-mods.json'" > "$OUT/native-mods-on-device.json"

echo "=== restart Starling with real RustPG code ==="
adb logcat -c
adb shell am start -W -n "$PKG/$PKG.LauncherActivity" > "$OUT/restart-native.txt" 2>&1 || true
sleep 30
adb exec-out screencap -p > "$OUT/05-native-start.png" || true

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
  adb logcat -d -v threadtime > "$OUT/logcat-native-no-control.txt" || true
  grep -Ei 'rustpg|native mods|native-libs|dlopen|symbol|error|failed|panic'     "$OUT/logcat-native-no-control.txt" | tail -500 || true
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
cmd("key v tap",30)
time.sleep(5)
ui=cmd("ui",60)
cmd("screenshot /sdcard/Download/rustpg-native-live.png",120)
cmd("status",60)
log.close()
print("UI_AFTER_V=",ui[:4000])
PY

echo "=== collect proof ==="
adb pull /sdcard/Download/rustpg-native-live.png "$OUT/06-rustpg-native-live.png" >/dev/null
adb exec-out screencap -p > "$OUT/07-device-final.png" || true
adb logcat -d -v threadtime > "$OUT/logcat-final.txt" || true
grep -Ei 'rustpg|native mods|native-libs|horizon_native|error|failed|panic'   "$OUT/logcat-final.txt" | tail -700 > "$OUT/rustpg-native-runtime.log" || true

test -s "$OUT/06-rustpg-native-live.png"
grep -q '"command": "key v tap"' "$OUT/control-session.jsonl"
grep -q 'rustpg' "$OUT/native-mods-on-device.json"
echo "=== REAL STARLING x86_64 + RUSTPG NATIVE SESSION COMPLETE ==="

# rerun-after-native-manifest-mixins-pin
