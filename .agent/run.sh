#!/usr/bin/env bash
set -euo pipefail
echo "=== RustPG 6 UI rework validation ==="
echo "HEAD=$(git rev-parse HEAD)"
bash scripts/validate.sh

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

echo "=== Horizon visual tooling availability ==="
command -v horizon-launcher || true
command -v horizon-ctl || true
find /usr /opt -maxdepth 3 -type f \( -name 'horizon-launcher' -o -name 'horizon-ctl' \) 2>/dev/null | head -20 || true

echo "=== version strings ==="
grep -RIn --exclude='RustPG.hmod' --exclude-dir='.git' '6\.0\.0\|5\.0\.0' CLAUDE.md AI_CONTEXT.txt project scripts .github 2>/dev/null | head -120 || true
