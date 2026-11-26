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

# This AVD is disposable. In run 45748355456, Android's Google SDK
# setup process ANR covered the Starling setup for the entire timeout.
# Disable only that non-game setup helper, never the game or SystemUI.
if adb shell pm list packages com.google.android.googlesdksetup | tr -d '\r' | grep -Fxq 'package:com.google.android.googlesdksetup'; then
  adb shell am force-stop com.google.android.googlesdksetup || true
  adb shell pm disable-user --user 0 com.google.android.googlesdksetup | tee "$OUT/google-sdk-setup.txt"
fi

echo "=== install custom exact-commit Starling ==="
adb install -r "$APK" | tee "$OUT/install.txt"
PKG=io.github.juanluci.starling.preview
ACTIVITY=io.github.juanluci.starling
GAME_HOME="/data/user/0/$PKG/files"
adb shell appops set "$PKG" MANAGE_EXTERNAL_STORAGE allow 2>/dev/null || true
adb shell pm grant "$PKG" android.permission.POST_NOTIFICATIONS 2>/dev/null || true
# Fresh disposable AVD: dismiss Android's immersive-mode education screen.
adb shell settings put secure immersive_mode_confirmations confirmed
adb shell settings get secure immersive_mode_confirmations > "$OUT/immersive-confirmation.txt"

# Preserve evidence from the first frame, and select the actual offline UI.
trap 'adb exec-out screencap -p > "$OUT/final-device.png" || true; adb logcat -d -v threadtime > "$OUT/logcat.txt" || true; kill "$EMU_PID" 2>/dev/null || true' EXIT
APP_UID="$(adb shell "stat -c '%u' '/data/user/0/$PKG'" | tr -d '\r')"
test -n "$APP_UID"
python3 -m pip install --break-system-packages numpy Pillow
# Pinned source: ha_launcher::start consumes this ordinary local Play request.
adb shell "mkdir -p '$GAME_HOME' '/storage/emulated/0/StarlingPreview'; touch '$GAME_HOME/launcher-play'; printf '%s\n' '{\"name\":\"RustPGNative\",\"profile\":0,\"open_directly\":true}' > '/storage/emulated/0/StarlingPreview/android-launcher.json'; cp '/storage/emulated/0/StarlingPreview/android-launcher.json' '$GAME_HOME/android-launcher.json'; chown $APP_UID:$APP_UID '$GAME_HOME/launcher-play' '$GAME_HOME/android-launcher.json'"
echo "=== first start + offline preparation ==="
adb logcat -c
adb shell am start -W -n "$PKG/$ACTIVITY.GameActivity" > "$OUT/launch.txt" 2>&1
python3 - "$OUT" <<'PY'
import io, sys, time, subprocess
from pathlib import Path
from PIL import Image
import numpy as np
out=Path(sys.argv[1])
def adb(*args):
    return subprocess.check_output(["adb",*args])
def button(name,width_dp,x_dp,height_dp=40):
    deadline=time.monotonic()+150
    last=None
    while time.monotonic()<deadline:
        data=adb("exec-out","screencap","-p")
        (out/f"setup-{name}-latest.png").write_bytes(data)
        img=Image.open(io.BytesIO(data)).convert("RGB")
        w,h=img.size
        if w<=h:
            time.sleep(2);continue
        d=max(1,min(4,w/1040,h/700))
        cx=w/2+x_dp*d
        a=np.asarray(img,dtype=np.int16)
        # The ANR dialog captured in run 45748355456 is a different modal.
        # Inspect accessibility text and dismiss only the exact disposable
        # emulator setup helper's ANR, not any game crash.
        mid=a[int(h*.27):int(h*.73),int(w*.23):int(w*.84)]
        if mid.size and (mid.min(axis=2)>200).mean()>.45:
            import re, xml.etree.ElementTree as ET
            try:
                subprocess.run(["adb","shell","uiautomator","dump","/sdcard/rustpg-window.xml"],timeout=20,check=True,stdout=subprocess.DEVNULL)
                root=ET.fromstring(adb("shell","cat","/sdcard/rustpg-window.xml"))
                labels=[n.get("text","").strip().lower() for n in root.iter("node")]
                if any("com.google.android.googlesdksetup" in label and "responding" in label for label in labels):
                    (out/"setup-google-sdk-anr.png").write_bytes(data)
                    close=next((n for n in root.iter("node") if n.get("text","").strip().lower()=="close app"),None)
                    if close is not None:
                        x1,y1,x2,y2=map(int,re.findall(r"\d+",close.attrib["bounds"]))
                        adb("shell","input","tap",str((x1+x2)//2),str((y1+y2)//2))
                    adb("shell","am","force-stop","com.google.android.googlesdksetup")
                    time.sleep(2);last=None;continue
            except (ET.ParseError,subprocess.SubprocessError,KeyError,ValueError):
                pass
        # Recognize the actual SystemUI overlay captured in run 45330563519.
        # Do not click the Starling setup screen while this modal covers it.
        panel=a[:int(h*.52),int(w*.25):int(w*.75)]
        if panel.size and (panel.min(axis=2)>210).mean()>.75:
            import re, xml.etree.ElementTree as ET
            try:
                subprocess.run(["adb","shell","uiautomator","dump","/sdcard/rustpg-window.xml"],timeout=20,check=True,stdout=subprocess.DEVNULL)
                root=ET.fromstring(adb("shell","cat","/sdcard/rustpg-window.xml"))
                got=next((n for n in root.iter("node") if n.get("text","").strip().lower()=="got it" and n.get("package","")=="com.android.systemui"),None)
                if got is not None:
                    x1,y1,x2,y2=map(int,re.findall(r"\d+",got.attrib["bounds"]))
                    (out/"setup-android-fullscreen-dialog.png").write_bytes(data)
                    adb("shell","input","tap",str((x1+x2)//2),str((y1+y2)//2))
                    time.sleep(2);last=None;continue
            except (ET.ParseError,subprocess.SubprocessError,KeyError,ValueError):
                pass
            # Fallback only when the captured blue button is actually visible.
            x1,x2,y1,y2=int(w*.66),int(w*.75),int(h*.39),int(h*.53)
            tile=a[y1:y2,x1:x2]
            blue=(tile[:,:,2]>tile[:,:,0]+25)&(tile[:,:,2]>tile[:,:,1]+12)&(tile[:,:,0]<120)
            if blue.size and blue.mean()>.25:
                yy,xx=np.nonzero(blue)
                adb("shell","input","tap",str(x1+int(np.median(xx))),str(y1+int(np.median(yy))))
                (out/"setup-android-fullscreen-dialog.png").write_bytes(data)
                time.sleep(2);last=None;continue
        # Full-width purple accent fill, not text, border or progress bar.
        mask=(a[:,:,2]>115)&(a[:,:,2]>a[:,:,0]+10)&(a[:,:,0]>a[:,:,1]+8)&(a[:,:,2]>a[:,:,1]+35)
        x0=max(0,round(cx-width_dp*d*.38));x1=min(w,round(cx+width_dp*d*.38))
        good=np.flatnonzero(mask[:,x0:x1].mean(axis=1)>.65)
        # Button text may interrupt rows; bridge small gaps.
        groups=np.split(good,np.where(np.diff(good)>max(2,round(5*d)))[0]+1)
        for group in groups:
            if not len(group):continue
            span=group[-1]-group[0]+1
            if span<height_dp*d*.55 or span>height_dp*d*1.20:continue
            y=int((group[0]+group[-1])/2)
            if not h*.40<y<h*.78:continue
            point=(round(cx),y,d)
            if last and abs(last[1]-y)<4 and abs(last[0]-cx)<4:
                (out/f"setup-{name}.png").write_bytes(data)
                print(name,point,flush=True)
                return point
            last=point
            break
        time.sleep(2)
    raise RuntimeError("Setup screen not recognized: "+name)
def tap(x,y):
    adb("shell","input","tap",str(round(x)),str(round(y)))
    time.sleep(3)
# setup/mod.rs draws Microsoft at right in row 1; Offline at right in row 2.
x,y,d=button("start",300,156)
tap(x,y+52*d)
x,y,d=button("offline-confirm",230,121)
tap(x,y)
x,y,d=button("offline-form",239,124.5,36)
# account_form.rs prefills the configured local name; Save submits it.
tap(x,y)
PY
adb exec-out screencap -p > "$OUT/02-download-start.png"

READY=""
for i in $(seq -w 1 24); do
  sleep 35
  adb exec-out screencap -p > "$OUT/03-progress-$i.png" || true
  adb shell "find '$GAME_HOME' '/storage/emulated/0/StarlingPreview' -name sunrise-resources.json -o -name facts.json 2>/dev/null | head -20"     > "$OUT/03-progress-$i.resources" || true
  if adb shell "test -s '$GAME_HOME/resources/sunrise-resources.json' && test -s '$GAME_HOME/resources/facts.json'"; then
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

echo "=== finish actual first-run game preferences ==="
# The pinned onboarding handler accepts Right (Game) then Enter.
# Wait for its own log, rather than tapping a download screen.
for i in $(seq 1 90); do
  adb logcat -d -s starling:I > "$OUT/onboarding-current.log" || true
  if grep -F 'Starling: showing first-run preferences' "$OUT/onboarding-current.log" > "$OUT/onboarding.log"; then break; fi
  sleep 2
done
if [ -s "$OUT/onboarding.log" ]; then
  adb shell input keyevent 22
  adb shell input keyevent 66
  sleep 12
fi
adb exec-out screencap -p > "$OUT/04b-game-choice.png" || true

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
adb shell "rm -f '$GAME_HOME/sunrise-control.json'; touch '$GAME_HOME/launcher-play'; chown $APP_UID:$APP_UID '$GAME_HOME/launcher-play'"
adb shell am start -W -n "$PKG/$ACTIVITY.GameActivity" > "$OUT/launch-game.txt"
for i in $(seq 1 120); do
  if adb shell "test -s '$GAME_HOME/sunrise-control.json'"; then break; fi
  sleep 2
done
adb shell "test -s '$GAME_HOME/sunrise-control.json'" || { echo "BLOCKED: Preview did not start Agent Control"; exit 30; }
args=()
if [ "$MODE" = baseline ]; then args+=(--baseline); fi
python3 tools/sunrise_functional_lab.py --endpoint "$GAME_HOME/sunrise-control.json" --device-home "$GAME_HOME" --out "$OUT" "${args[@]}"
