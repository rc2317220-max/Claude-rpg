#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

PIN=ff303502a4f7982551a0e0d3622d9268efe9e829
HORIZON="$PWD/.ai-tools/horizon"

echo "=== prepare pinned Horizon SDK ==="
mkdir -p .ai-tools .agent/artifacts
if [ ! -d "$HORIZON/.git" ]; then
  git clone --filter=blob:none https://github.com/arismoko/horizon.git "$HORIZON"
fi
git -C "$HORIZON" fetch --depth=1 origin "$PIN"
git -C "$HORIZON" checkout --detach "$PIN"

echo "=== build RustPG.hmod ==="
HORIZON_SOURCE="$HORIZON" bash scripts/rustpg_build.sh

echo "=== verify artifact ==="
test -s releases/RustPG.hmod
cp releases/RustPG.hmod .agent/artifacts/RustPG.hmod
sha256sum .agent/artifacts/RustPG.hmod | tee .agent/artifacts/RustPG.hmod.sha256
ls -lh .agent/artifacts/RustPG.hmod

echo "=== build complete ==="

# retry-after-inspector-dock-fix
