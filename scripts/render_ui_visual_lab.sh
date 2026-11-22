#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SVG_DIR="${1:-$ROOT/.agent/artifacts/ui-svg}"
PNG_DIR="${2:-$ROOT/.agent/artifacts/ui-png}"
mkdir -p "$SVG_DIR" "$PNG_DIR"

if ! command -v rsvg-convert >/dev/null 2>&1; then
  if command -v apt-get >/dev/null 2>&1 && command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
    sudo apt-get update
    sudo apt-get install -y --no-install-recommends librsvg2-bin
  else
    echo "rsvg-convert missing; SVG outputs remain available" >&2
    exit 0
  fi
fi

shopt -s nullglob
for svg in "$SVG_DIR"/*.svg; do
  name="$(basename "${svg%.svg}")"
  rsvg-convert "$svg" -o "$PNG_DIR/$name.png"
done
echo "UI visual lab PNGs:"
find "$PNG_DIR" -type f -name '*.png' -printf '%f %s bytes\n' | sort
