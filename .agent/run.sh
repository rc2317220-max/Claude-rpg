#!/usr/bin/env bash
set -euo pipefail

echo "=== RustPG 6.1.0-alpha final visual rework gate ==="
echo "HEAD=$(git rev-parse HEAD)"

echo "=== Horizon visual tooling probe (before build) ==="
echo "horizon-launcher=$(command -v horizon-launcher || true)"
echo "horizon-ctl=$(command -v horizon-ctl || true)"
find /usr /opt "$HOME" -maxdepth 4 -type f \( -name 'horizon-launcher' -o -name 'horizon-ctl' \) 2>/dev/null | head -20 || true
python3 tools/horizon_visual_driver.py --help >/tmp/horizon-driver-help.txt 2>&1 || true
head -80 /tmp/horizon-driver-help.txt || true

echo "=== generate authored visual assets ==="
python3 scripts/generate_ui_assets.py
python3 scripts/generate_creatures.py
python3 scripts/generate_ai_context.py

echo "=== structural/package validation ==="
bash scripts/validate.sh

echo "=== checkout pinned Horizon SDK ==="
rm -rf /tmp/rustpg-horizon
git clone --filter=blob:none --no-checkout https://github.com/arismoko/horizon.git /tmp/rustpg-horizon
git -C /tmp/rustpg-horizon checkout ff303502a4f7982551a0e0d3622d9268efe9e829

cp project/Cargo.toml /tmp/rustpg-Cargo.toml
trap 'cp /tmp/rustpg-Cargo.toml project/Cargo.toml' EXIT
python3 - <<'PY'
from pathlib import Path
p=Path("project/Cargo.toml")
s=p.read_text()
old='horizon-mod-api = { path = ".horizon/sdk/api/crates/horizon-mod-api" }'
new='horizon-mod-api = { path = "/tmp/rustpg-horizon/crates/horizon-mod-api" }'
if old not in s:
    raise SystemExit("Horizon dependency marker missing")
p.write_text(s.replace(old,new))
PY

echo "=== cargo check ==="
cargo check --manifest-path project/Cargo.toml --lib

echo "=== cargo test ==="
cargo test --manifest-path project/Cargo.toml --lib -- --test-threads=2

echo "=== restore distributable manifest ==="
cp /tmp/rustpg-Cargo.toml project/Cargo.toml
trap - EXIT

echo "=== persist generated visual assets/context after successful gates ==="
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git add   project/assets/rustpg/textures/gui   project/assets/rustpg/textures/entity   project/src/rustpg_creatures_generated.rs   project/data/rustpg/horizon/entity_type   project/data/rustpg/loot_table/entities   AI_CONTEXT.txt SOURCE_INDEX.md REPO_MAP.md UI_MAP.md COMBAT_MAP.md RIFTS_MAP.md CREATURES_MAP.md

# releases/RustPG.hmod is intentionally not committed on the rework branch.
git restore --worktree releases/RustPG.hmod 2>/dev/null || true

if git diff --cached --quiet; then
  echo "generated assets/context already current"
else
  git commit -m "assets: publish validated 6.1.0-alpha rift-forged visuals"
  git push origin HEAD:rework/rustpg-6-ui
fi

echo "=== final summary ==="
echo "GUI_PNG=$(find project/assets/rustpg/textures/gui -type f -name '*.png' | wc -l)"
echo "ENTITY_PNG=$(find project/assets/rustpg/textures/entity -type f -name '*.png' | wc -l)"
echo "PACKAGE=$(stat -c '%s bytes' releases/RustPG.hmod 2>/dev/null || true)"
