#!/usr/bin/env bash
set -euo pipefail
echo "=== regenerate RustPG visual assets ==="
python3 scripts/generate_ui_assets.py
python3 scripts/generate_creatures.py
bash scripts/validate.sh

git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git add project/assets/rustpg/textures/gui         project/assets/rustpg/textures/entity         project/src/rustpg_creatures_generated.rs         project/data/rustpg/horizon/entity_type         project/data/rustpg/loot_table/entities
if git diff --cached --quiet; then
  echo "generated assets already current"
else
  git commit -m "assets: regenerate RustPG visual system and creature materials"
  git reset --hard HEAD
  git fetch origin rework/rustpg-6-ui
  git rebase origin/rework/rustpg-6-ui
  git push origin HEAD:rework/rustpg-6-ui
fi

echo "=== generated asset summary ==="
find project/assets/rustpg/textures/gui -type f -name '*.png' | wc -l
find project/assets/rustpg/textures/entity -type f -name '*.png' | wc -l
