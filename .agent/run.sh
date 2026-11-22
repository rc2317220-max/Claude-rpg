#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
PIN=ff303502a4f7982551a0e0d3622d9268efe9e829
HORIZON="$PWD/.ai-tools/horizon"

echo "=== prepare exact pinned Horizon source ==="
mkdir -p .ai-tools
if [ ! -d "$HORIZON/.git" ]; then
  git clone --filter=blob:none https://github.com/arismoko/horizon.git "$HORIZON"
fi
git -C "$HORIZON" fetch --depth=1 origin "$PIN"
git -C "$HORIZON" checkout --detach "$PIN"

echo "=== regenerate RustPG creature sources/textures ==="
python3 scripts/generate_creatures.py

echo "=== RustPG validation/check/tests against pinned Horizon ==="
HORIZON_SOURCE="$HORIZON" bash scripts/rustpg_build.sh

echo "=== RustPG Horizon entity visual lab ==="
bash scripts/run_entity_visual_lab.sh .agent/artifacts/rustpg-entities

echo "=== manifest ==="
cat .agent/artifacts/rustpg-entities/manifest.json
echo "=== PNG count ==="
find .agent/artifacts/rustpg-entities -type f -name '*.png' | wc -l
