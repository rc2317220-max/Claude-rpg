#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SVG_DIR="${1:-$ROOT/.agent/artifacts/ui-svg}"
PNG_DIR="${2:-$ROOT/.agent/artifacts/ui-png}"
python3 "$ROOT/tools/check_ui_capture.py" "$SVG_DIR" svg

if ! command -v rsvg-convert >/dev/null 2>&1; then
  if command -v apt-get >/dev/null && command -v sudo >/dev/null && sudo -n true 2>/dev/null; then
    sudo apt-get update
    sudo apt-get install -y --no-install-recommends librsvg2-bin
  else
    echo "Install librsvg2-bin to render UI captures to PNG." >&2
    exit 2
  fi
fi

mkdir -p "$PNG_DIR"
stage="$(mktemp -d "$PNG_DIR/.ui-render.XXXXXXXX")"
trap 'rm -rf -- "$stage"' EXIT
for svg in "$SVG_DIR"/*.svg; do
  name="$(basename "${svg%.svg}")"
  rsvg-convert "$svg" -o "$stage/$name.png"
done
python3 "$ROOT/tools/check_ui_capture.py" "$stage" png --publish "$PNG_DIR"
