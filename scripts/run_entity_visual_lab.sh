#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PIN=ff303502a4f7982551a0e0d3622d9268efe9e829
TOOLS="${RUSTPG_AI_TOOLS_DIR:-$ROOT/.ai-tools}"
HORIZON="$TOOLS/horizon"
SIZE="${RUSTPG_ENTITY_LAB_SIZE:-256}"
OUT="${1:-$ROOT/.agent/artifacts/rustpg-entities}"

for cmd in git cargo rustc python3; do
  command -v "$cmd" >/dev/null 2>&1 || { echo "missing required tool: $cmd" >&2; exit 3; }
done

if command -v apt-get >/dev/null 2>&1 && command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
  sudo apt-get update
  sudo apt-get install -y --no-install-recommends mesa-vulkan-drivers libvulkan1 libasound2-dev libudev-dev
fi

if [ ! -e "$HORIZON/.git" ]; then
  mkdir -p "$TOOLS"
  git clone --filter=blob:none https://github.com/arismoko/horizon.git "$HORIZON"
fi
git -C "$HORIZON" fetch --depth=1 origin "$PIN"
git -C "$HORIZON" checkout --detach "$PIN"

# Compile the pinned renderer at the requested capture size. Restore the dedicated
# SDK cache afterwards so the next pass can use a different resolution.
relative_pixel=crates/horizon-client-render/src/gfx/entity_pixel_test.rs
git -C "$HORIZON" diff --quiet HEAD -- "$relative_pixel" || {
  echo "The SDK pixel renderer has local changes: $HORIZON/$relative_pixel" >&2
  exit 2
}
stage="$(mktemp -d)"
trap 'git -C "$HORIZON" restore --worktree -- "$relative_pixel"; rm -rf -- "$stage"' EXIT

PIXEL="$HORIZON/crates/horizon-client-render/src/gfx/entity_pixel_test.rs"
python3 - "$PIXEL" "$SIZE" <<'PY'
from pathlib import Path
import re,sys
p=Path(sys.argv[1]); size=int(sys.argv[2])
s=p.read_text()
old="const SIZE: u32 = 64;"
if old not in s: raise SystemExit("could not find entity pixel-test SIZE marker")
p.write_text(s.replace(old,f"const SIZE: u32 = {size};",1))
PY

mkdir -p "$stage/captures"
export RUSTPG_ENTITY_LAB_SIZE="$SIZE"
export WGPU_BACKEND=vulkan
export CARGO_TARGET_DIR="$TOOLS/target/entity-visual-lab"

cargo run --manifest-path "$ROOT/tools/entity_visual_lab/Cargo.toml" -- "$stage/captures"
python3 "$ROOT/tools/check_entity_visuals.py" "$stage/captures" > /dev/null
mkdir -p "$OUT"
cp -R "$stage/captures/." "$OUT/"
echo "Entity visual lab complete: $OUT"
find "$OUT" -type f -printf '%p %s bytes\\n' | sort
