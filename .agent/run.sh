#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
echo "=== regenerate RustPG creature sources/textures ==="
python3 scripts/generate_creatures.py
echo "=== RustPG validation/check/tests against pinned Horizon ==="
HORIZON_SOURCE="$PWD/.ai-tools/horizon" bash scripts/rustpg_build.sh
echo "=== RustPG Horizon entity visual lab ==="
bash scripts/run_entity_visual_lab.sh .agent/artifacts/rustpg-entities
echo "=== manifest ==="
cat .agent/artifacts/rustpg-entities/manifest.json
echo "=== PNG count ==="
find .agent/artifacts/rustpg-entities -type f -name '*.png' | wc -l
