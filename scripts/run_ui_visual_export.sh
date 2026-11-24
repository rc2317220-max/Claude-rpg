#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HORIZON="$(realpath "${HORIZON_SOURCE:-$ROOT/.ai-tools/horizon}")"
[[ -d "$HORIZON/crates/horizon-mod-api" ]] || { echo "Missing Horizon mod API in $HORIZON" >&2; exit 2; }
: "${RUSTPG_UI_LAB_OUT:?Set RUSTPG_UI_LAB_OUT to the SVG output directory}"
mkdir -p "$RUSTPG_UI_LAB_OUT"
out="$(realpath "$RUSTPG_UI_LAB_OUT")"
stage="$(mktemp -d "$out/.ui-export.XXXXXXXX")"
trap 'rm -rf -- "$stage"' EXIT

# Reuse a configured checkout. A private manifest adapts other checkouts to
# the supplied SDK without changing their tracked manifest or lockfile.
mkdir -p "$stage/svg"
manifest="$ROOT/project/Cargo.toml"
if [[ "$(realpath -e "$ROOT/project/.horizon/sdk/api" 2>/dev/null || true)" != "$HORIZON" ]]; then
  mkdir -p "$stage/project"
  cp "$ROOT/project/Cargo.lock" "$stage/project/"
  for path in src data assets; do
    ln -s "$ROOT/project/$path" "$stage/project/$path"
  done
  python3 - "$manifest" "$stage/project/Cargo.toml" "$HORIZON" <<'PYSDK'
import json, sys
from pathlib import Path
source, dest, horizon = map(Path, sys.argv[1:])
text = source.read_text()
needle = '".horizon/sdk/api/crates/horizon-mod-api"'
if text.count(needle) != 1:
    sys.exit(f"Expected one Horizon SDK dependency in {source}")
dest.write_text(text.replace(needle, json.dumps(str(horizon / "crates/horizon-mod-api"))))
PYSDK
  manifest="$stage/project/Cargo.toml"
fi
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/project/target}" RUSTPG_UI_LAB_OUT="$stage/svg" \
  cargo test --locked --manifest-path "$manifest" --lib \
  rustpg_ui_visual_lab::visual_lab_exports_svg -- --exact --nocapture

python3 "$ROOT/tools/check_ui_capture.py" "$stage/svg" svg --publish "$out"
