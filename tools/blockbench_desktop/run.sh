#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
LAB="$ROOT/.ai-tools/blockbench-desktop"
OUT="$ROOT/.agent/artifacts/blockbench-desktop"
mkdir -p "$OUT" "$LAB/profile" "$OUT/checkpoint" "$LAB/input"
export LIBGL_ALWAYS_SOFTWARE=1
export BB_BRIDGE_PORT=8188
# Explicit launch permits startup diagnostics and screenshots of the app itself.
# CDP is loopback-only and used solely to load the known, pinned local plugin.
export BB_HEADLESS=0
export DISPLAY=:99
SIZE=1600x1000
if [[ -n "${BB_REFERENCE_PATH:-}" ]]; then SIZE=2400x1200; fi
Xvfb "$DISPLAY" -screen 0 "$SIZE"x24 -nolisten tcp >"$OUT/xvfb.log" 2>&1 &
XV=$!
APP=""; WM=""; REC=""; REF=""
cleanup(){
  if [[ -n "$REF" ]]; then kill "$REF" 2>/dev/null || true; fi
  if [[ -n "$REC" ]]; then kill -INT "$REC" 2>/dev/null || true; wait "$REC" 2>/dev/null || true; fi
  if [[ -n "$APP" ]]; then kill "$APP" 2>/dev/null || true; fi
  if [[ -n "$WM" ]]; then kill "$WM" 2>/dev/null || true; fi
  kill "$XV" 2>/dev/null || true
}
trap cleanup EXIT
for i in $(seq 1 40); do if xdpyinfo >/dev/null 2>&1; then break; fi; sleep .25; done
openbox >"$OUT/openbox.log" 2>&1 & WM=$!
glxinfo -B >"$OUT/graphics.txt" 2>&1 || true
blockbench --user-data-dir="$LAB/profile" --remote-debugging-address=127.0.0.1 --remote-debugging-port=9222 --enable-unsafe-swiftshader --use-gl=angle --use-angle=swiftshader --disable-dev-shm-usage >"$OUT/blockbench.log" 2>&1 & APP=$!
export BB_LAB="$LAB" BB_OUT="$OUT" BB_ROOT="$ROOT"
if [[ -n "${BB_REFERENCE_PATH:-}" ]]; then
  # Only checked-out or restored local raster files. No external URL fetch.
  case "$BB_REFERENCE_PATH" in
    '$INPUT/'*) REF_PATH="$LAB/input/${BB_REFERENCE_PATH#\$INPUT/}" ;;
    *) REF_PATH="$ROOT/$BB_REFERENCE_PATH" ;;
  esac
  REF_PATH=$(realpath -e "$REF_PATH")
  case "$REF_PATH" in "$ROOT/"*) ;; *) echo "reference outside workspace"; exit 1;; esac
  [[ -f "$REF_PATH" ]] || exit 1
  python3 "$ROOT/tools/blockbench_desktop/reference_viewer.py" "$REF_PATH" "$OUT" >"$OUT/reference-viewer.log" 2>&1 & REF=$!
  for i in $(seq 1 80); do
    [[ -s "$OUT/reference-ack.json" ]] && break
    kill -0 "$REF" 2>/dev/null || { cat "$OUT/reference-viewer.log"; exit 1; }
    sleep .25
  done
  test -s "$OUT/reference-ack.json"
fi
# Record actual X11 desktop edits, not a fabricated animation or stream.
ffmpeg -hide_banner -loglevel warning -y -f x11grab -framerate 4 -video_size "$SIZE" -i "$DISPLAY" -vf "scale=1600:-2" -c:v libx264 -preset ultrafast -crf 27 -pix_fmt yuv420p "$OUT/desktop-editing.mp4" >"$OUT/recording.log" 2>&1 & REC=$!
node "$ROOT/tools/blockbench_desktop/bridge.mjs" "$ROOT/.agent/blockbench-desktop-commands.json" || {
  import -window root "$OUT/desktop-failure.png" 2>/dev/null || true
  exit 1
}
import -window root "$OUT/desktop-window.png"
