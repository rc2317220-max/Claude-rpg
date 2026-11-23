#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
echo "=== RustPG HUD real model and menus: structural validation ==="
bash scripts/validate.sh
echo "=== Checkout exact pinned Horizon 0.9.0 SDK (no private patches) ==="
HORIZON=.agent/horizon-sdk-for-ui-check
if [ ! -e "$HORIZON/Cargo.toml" ]; then
  git clone --filter=blob:none --no-checkout https://github.com/arismoko/horizon.git "$HORIZON"
fi
git -C "$HORIZON" fetch --depth=1 origin ff303502a4f7982551a0e0d3622d9268efe9e829
git -C "$HORIZON" checkout --detach ff303502a4f7982551a0e0d3622d9268efe9e829
python3 - <<'PY'
from pathlib import Path
p=Path('project/Cargo.toml')
s=p.read_text()
old='horizon-mod-api = { path = ".horizon/sdk/api/crates/horizon-mod-api" }'
new='horizon-mod-api = { path = "../.agent/horizon-sdk-for-ui-check/crates/horizon-mod-api" }'
assert old in s, "manifest SDK dependency moved"
p.write_text(s.replace(old,new))
PY
echo "=== Locate pinned-SDK GUI entity preview vs world entity render pipeline ==="
git -C "$HORIZON" grep -n -F 'GuiEntity::Existing' -- 'crates/**/*.rs' | head -12 || true
git -C "$HORIZON" grep -n -F 'GuiEntityView' -- 'crates/**/*.rs' | head -12 || true
git -C "$HORIZON" grep -n -F 'register_entity_renderers' -- 'crates/horizon-client/**/*.rs' | head -12 || true
echo "=== Compile real HUD, menus and native entity renderer against pinned SDK ==="
cargo check --manifest-path project/Cargo.toml --lib
echo "=== Run complete RustPG regression suite ==="
cargo test --manifest-path project/Cargo.toml --lib -- --test-threads=2
echo "RUSTPG_UI_HUD_AND_PORTRAITS_VALIDATED"

# latest menu interaction audit

# all original models generate mesh-derived GUI portraits from source
