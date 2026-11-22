#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

bash scripts/validate.sh

if [[ -n "${HORIZON_SOURCE:-}" ]]; then
  API="$HORIZON_SOURCE/crates/horizon-mod-api"
  [[ -d "$API" ]] || { echo "HORIZON_SOURCE has no crates/horizon-mod-api: $HORIZON_SOURCE" >&2; exit 2; }
  cp project/Cargo.toml /tmp/rustpg-Cargo.toml
  trap 'cp /tmp/rustpg-Cargo.toml project/Cargo.toml' EXIT
  python3 - "$API" <<'PY'
from pathlib import Path
import sys
p=Path("project/Cargo.toml")
s=p.read_text()
old='horizon-mod-api = { path = ".horizon/sdk/api/crates/horizon-mod-api" }'
new=f'horizon-mod-api = {{ path = "{Path(sys.argv[1]).resolve()}" }}'
if old not in s: raise SystemExit("expected Horizon dependency not found")
p.write_text(s.replace(old,new))
PY
  cargo check --manifest-path project/Cargo.toml --lib
  cargo test --manifest-path project/Cargo.toml --lib -- --test-threads=2
fi

echo "$ROOT/releases/RustPG.hmod"
