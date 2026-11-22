#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
python3 "$ROOT/scripts/package_native.py"
unzip -tq "$ROOT/releases/RustPG.hmod" >/dev/null
echo 'RustPG.hmod íntegro — pacote único para Compile v2.'
