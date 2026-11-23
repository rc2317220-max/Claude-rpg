#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
PIN=ff303502a4f7982551a0e0d3622d9268efe9e829
HORIZON="$PWD/.ai-tools/horizon"

echo "=== prepare exact pinned Horizon source ==="
mkdir -p .ai-tools .agent/artifacts
if [ ! -d "$HORIZON/.git" ]; then
  git clone --filter=blob:none https://github.com/arismoko/horizon.git "$HORIZON"
fi
git -C "$HORIZON" fetch --depth=1 origin "$PIN"
git -C "$HORIZON" checkout --detach "$PIN"

echo "=== regenerate procedural RustPG creatures ==="
python3 scripts/generate_creatures.py
echo "=== remaster hand-authored creature textures ==="
python3 scripts/remaster_handcrafted_creatures.py

echo "=== RustPG validation/check/tests + UI layer export ==="
export RUSTPG_UI_LAB_OUT="$PWD/.agent/artifacts/ui-svg"
HORIZON_SOURCE="$HORIZON" bash scripts/rustpg_build.sh

echo "=== render UI SVGs to PNG ==="
bash scripts/render_ui_visual_lab.sh .agent/artifacts/ui-svg .agent/artifacts/ui-png

echo "=== RustPG Horizon entity visual lab ==="
bash scripts/run_entity_visual_lab.sh .agent/artifacts/rustpg-entities

echo "=== objective entity visual QA report ==="
python3 tools/check_entity_visuals.py .agent/artifacts/rustpg-entities --output .agent/artifacts/entity-visual-quality.json

echo "=== preserve generated visual sources for promotion ==="
mkdir -p .agent/artifacts/generated/project/src
mkdir -p .agent/artifacts/generated/project/assets/rustpg/textures/entity
mkdir -p .agent/artifacts/generated/project/models/creatures
cp project/src/rustpg_creatures_generated.rs .agent/artifacts/generated/project/src/
cp project/src/rustpg_generated_models.rs project/src/rustpg_generated_hunter.rs .agent/artifacts/generated/project/src/
cp project/assets/rustpg/textures/entity/*.png .agent/artifacts/generated/project/assets/rustpg/textures/entity/
cp project/models/creatures/ashen_sentinel.bbmodel project/models/creatures/rift_hunter.bbmodel .agent/artifacts/generated/project/models/creatures/
cp releases/RustPG.hmod .agent/artifacts/RustPG-visual-loop.hmod

echo "=== visual artifact summary ==="
echo -n "entity_png="; find .agent/artifacts/rustpg-entities -type f -name "*.png" | wc -l
echo -n "ui_svg="; find .agent/artifacts/ui-svg -type f -name "*.svg" | wc -l
echo -n "ui_png="; find .agent/artifacts/ui-png -type f -name "*.png" | wc -l
find .agent/artifacts -maxdepth 3 -type f -printf "%p %s bytes\n" | sort | tail -120

# ui-svg-syntax-fix-retry

# retry-after-ui-float-fix

# visual-round-2-responsive-silhouettes-actions

# visual-round-3-hud-coverage
