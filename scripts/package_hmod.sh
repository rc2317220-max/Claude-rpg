#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
python3 "$ROOT/scripts/package_native.py"
for name in RustPG-dev.hmod RustPG-core.hmod; do
  unzip -tq "$ROOT/releases/$name" >/dev/null
done
echo "ZIP/HMOD íntegros. Core: Compile v2; Dev: código-fonte com mixins (somente whole-game)."
