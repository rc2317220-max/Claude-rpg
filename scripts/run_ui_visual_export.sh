#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HORIZON="${HORIZON_SOURCE:-$ROOT/.ai-tools/horizon}"
API="$HORIZON/crates/horizon-mod-api"
[[ -d "$API" ]] || { echo "missing pinned Horizon mod API: $API" >&2; exit 2; }

backup="$(mktemp)"
cp "$ROOT/project/Cargo.toml" "$backup"
restore(){ cp "$backup" "$ROOT/project/Cargo.toml"; rm -f "$backup"; }
trap restore EXIT

python3 - "$ROOT/project/Cargo.toml" "$API" <<'PY'
from pathlib import Path
import sys
p=Path(sys.argv[1]); api=Path(sys.argv[2]).resolve()
s=p.read_text()
old='horizon-mod-api = { path = ".horizon/sdk/api/crates/horizon-mod-api" }'
new=f'horizon-mod-api = {{ path = "{api}" }}'
if old not in s:
    raise SystemExit("expected Horizon dependency not found")
p.write_text(s.replace(old,new))
PY

cargo test --manifest-path "$ROOT/project/Cargo.toml" --lib \
  rustpg_ui_visual_lab::visual_lab_exports_svg -- --exact --nocapture

out="${RUSTPG_UI_LAB_OUT:-}"
[[ -n "$out" ]] || { echo "RUSTPG_UI_LAB_OUT must be set" >&2; exit 3; }
count="$(find "$out" -maxdepth 1 -type f -name '*.svg' | wc -l | tr -d ' ')"
[[ "$count" = "68" ]] || {
  echo "UI visual export regression: expected 68 SVGs, got $count" >&2
  exit 4
}
echo "UI visual export OK: $count SVGs"
