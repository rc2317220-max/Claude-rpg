#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

OUT="$PWD/.agent/artifacts/starling-native-runtime"
STARLING="$PWD/.ai-tools/starling-native-0.9.0"
PIN=ff303502a4f7982551a0e0d3622d9268efe9e829
mkdir -p "$OUT" .ai-tools

echo "=== prepare host ==="
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends libasound2-dev libudev-dev libvulkan1 mesa-vulkan-drivers vulkan-tools xvfb xauth >/dev/null
vulkaninfo --summary 2>&1 | tee "$OUT/vulkan.txt" || true

echo "=== fetch exact Starling 0.9.0 ==="
if [ ! -d "$STARLING/.git" ]; then
  git init "$STARLING"
  git -C "$STARLING" remote add origin https://github.com/starling-client/starling.git
fi
git -C "$STARLING" fetch --depth=1 origin "$PIN"
git -C "$STARLING" checkout --detach FETCH_HEAD
git -C "$STARLING" rev-parse HEAD | tee "$OUT/starling-revision.txt"

echo "=== inspect private home and control hooks ==="
grep -R -n --exclude-dir=target "fn private_home\|pub fn private_home" "$STARLING/crates" | tee "$OUT/private-home-grep.txt" || true
grep -R -n --exclude-dir=target "control.json\|agent control\|horizon-ctl\|control.*socket\|control.*port" "$STARLING/crates/horizon-client/src" | head -200 | tee "$OUT/control-grep.txt" || true

echo "=== build native Starling client ==="
export CARGO_TARGET_DIR="$PWD/.ai-tools/starling-target"
cargo build --manifest-path "$STARLING/Cargo.toml" -p horizon-client

BIN="$CARGO_TARGET_DIR/debug/horizon-client"
test -x "$BIN"
"$BIN" --help > "$OUT/help.txt" 2>&1 || true
"$BIN" --launch-api > "$OUT/launch-api.json" 2>&1 || true

echo "=== launch without account, developer single player ==="
HOME_ROOT="$PWD/.agent/starling-home"
GAME="$PWD/.agent/starling-game"
mkdir -p "$HOME_ROOT" "$GAME"
export STARLING_ACCOUNTS=1
export XDG_DATA_HOME="$HOME_ROOT/xdg-data"
export XDG_CONFIG_HOME="$HOME_ROOT/xdg-config"
export XDG_CACHE_HOME="$HOME_ROOT/xdg-cache"
export HORIZON_HOME="$HOME_ROOT/horizon"
export RUST_LOG=info
set +e
timeout 30s xvfb-run -a -s "-screen 0 1280x720x24" "$BIN" \
  --developer-single-player --world RustPG_AI_Test \
  --game-dir "$GAME" --home "$HOME_ROOT/horizon" \
  >"$OUT/client-first.log" 2>&1
RC=$?
set -e
echo "$RC" > "$OUT/client-first.rc"
find "$HOME_ROOT" -maxdepth 5 -type f -printf '%p %s\n' | sort > "$OUT/home-files.txt" || true
find "$GAME" -maxdepth 5 -type f -printf '%p %s\n' | sort > "$OUT/game-files.txt" || true

echo "=== key log ==="
grep -Ei "resource|account|offline|download|control|error|panic|vulkan|renderer|Starling" "$OUT/client-first.log" | tail -200 || true

echo "=== native probe done ==="
