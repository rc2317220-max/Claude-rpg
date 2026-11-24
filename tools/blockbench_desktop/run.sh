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
Xvfb "$DISPLAY" -screen 0 1600x1000x24 -nolisten tcp >"$OUT/xvfb.log" 2>&1 &
XV=$!
APP=""; WM=""
cleanup(){
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
node "$ROOT/tools/blockbench_desktop/bridge.mjs" "$ROOT/.agent/blockbench-desktop-commands.json" || {
  import -window root "$OUT/desktop-failure.png" 2>/dev/null || true
  exit 1
}
import -window root "$OUT/desktop-window.png"
