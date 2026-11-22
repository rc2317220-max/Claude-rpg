#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT="$ROOT/project"
OUT="$ROOT/releases/Claude-RPG-dev.hmod"
rm -f "$OUT"
(cd "$PROJECT" && zip -qr "$OUT" . -x 'target/*' '.git/*' '*.log')
echo "Gerado: $OUT"
unzip -tq "$OUT" >/dev/null
echo 'ZIP/HMOD íntegro.'
