#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
echo "=== RustPG Horizon entity visual lab ==="
bash scripts/run_entity_visual_lab.sh .agent/artifacts/rustpg-entities
echo "=== manifest ==="
cat .agent/artifacts/rustpg-entities/manifest.json
echo "=== PNG count ==="
find .agent/artifacts/rustpg-entities -type f -name '*.png' | wc -l

# retry-after-size-fix
