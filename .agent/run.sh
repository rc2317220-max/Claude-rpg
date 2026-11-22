#!/usr/bin/env bash
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
OUT="$ROOT/.agent/artifacts/horizon-entity-fixture"
HORIZON=/tmp/horizon-entity-lab
PIN=ff303502a4f7982551a0e0d3622d9268efe9e829

rm -rf "$OUT" "$HORIZON"
mkdir -p "$OUT"

echo "=== install software Vulkan/display support ==="
sudo apt-get update
sudo apt-get install -y --no-install-recommends mesa-vulkan-drivers libvulkan1 vulkan-tools xvfb libasound2-dev libudev-dev

echo "=== clone pinned Horizon ==="
git clone --filter=blob:none https://github.com/arismoko/horizon.git "$HORIZON"
git -C "$HORIZON" checkout --detach "$PIN"

echo "=== software Vulkan probe ==="
vulkaninfo --summary | head -120 || true

echo "=== compile asset-free entity fixture ==="
cd "$HORIZON"
python3 tools/run_standard_pack_gpu_entities.py "$OUT" --check-only

echo "=== render asset-free entity fixture ==="
xvfb-run -a -s "-screen 0 1280x720x24"   python3 tools/run_standard_pack_gpu_entities.py "$OUT"

echo "=== outputs ==="
find "$OUT" -maxdepth 2 -type f -printf '%p %s bytes\n' | sort
python3 - <<'PY'
from pathlib import Path
import json
out=Path("'"$OUT"'")
for p in sorted(out.glob("*.png")):
    print("PNG",p.name,p.stat().st_size)
r=out/"result.json"
if r.exists():
    print(r.read_text())
PY
