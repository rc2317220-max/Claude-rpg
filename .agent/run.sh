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
echo "=== regenerate canonical rift-forged UI assets ==="
python3 scripts/generate_ui_assets.py

echo "=== RustPG Horizon entity visual lab ==="
bash scripts/run_entity_visual_lab.sh .agent/artifacts/rustpg-entities

echo "=== objective entity visual QA report ==="
python3 tools/check_entity_visuals.py .agent/artifacts/rustpg-entities --output .agent/artifacts/entity-visual-quality.json

export RUSTPG_ENTITY_PREVIEWS="$PWD/.agent/artifacts/rustpg-entities"
echo "=== fast UI-only export against pinned Horizon ==="
export RUSTPG_UI_LAB_OUT="$PWD/.agent/artifacts/ui-svg"
HORIZON_SOURCE="$HORIZON" bash scripts/run_ui_visual_export.sh

echo "=== render UI SVGs to PNG ==="
bash scripts/render_ui_visual_lab.sh .agent/artifacts/ui-svg .agent/artifacts/ui-png

echo "=== labelled visual contact sheets ==="
python3 tools/make_visual_contact_sheets.py \
  .agent/artifacts/ui-png \
  .agent/artifacts/rustpg-entities \
  .agent/artifacts/contact-svg
mkdir -p .agent/artifacts/contact-png
for svg in .agent/artifacts/contact-svg/*.svg; do
  [ -f "$svg" ] || continue
  rsvg-convert "$svg" -o ".agent/artifacts/contact-png/$(basename "${svg%.svg}.png")"
done

echo "=== preserve generated visual sources for promotion ==="
mkdir -p .agent/artifacts/generated/project/src
mkdir -p .agent/artifacts/generated/project/assets/rustpg/textures/entity
mkdir -p .agent/artifacts/generated/project/assets/rustpg/textures/gui
mkdir -p .agent/artifacts/generated/project/models/creatures
cp project/src/rustpg_creatures_generated.rs .agent/artifacts/generated/project/src/
cp project/src/rustpg_generated_models.rs project/src/rustpg_generated_hunter.rs .agent/artifacts/generated/project/src/
cp project/assets/rustpg/textures/entity/*.png .agent/artifacts/generated/project/assets/rustpg/textures/entity/
cp -R project/assets/rustpg/textures/gui/. .agent/artifacts/generated/project/assets/rustpg/textures/gui/
cp project/models/creatures/ashen_sentinel.bbmodel project/models/creatures/rift_hunter.bbmodel .agent/artifacts/generated/project/models/creatures/
echo "=== package exactly the sources used for this capture ==="
python3 scripts/package_native.py
cp releases/RustPG.hmod .agent/artifacts/RustPG.hmod
(cd .agent/artifacts && sha256sum RustPG.hmod >RustPG.hmod.sha256)
git rev-parse HEAD >.agent/artifacts/revision.txt

echo "=== visual artifact summary ==="
find .agent/artifacts/{rustpg-entities,ui-png,contact-png} -name '*.png' | wc -l
