#!/usr/bin/env bash
set -euo pipefail

echo "=== RustPG agent terminal diagnostic ==="
echo "commit: $(git rev-parse HEAD)"
echo "pwd: $(pwd)"
echo
echo "--- system ---"
uname -a
echo
echo "--- tools ---"
git --version
python3 --version
rustc --version
cargo --version
echo
echo "--- network ---"
git ls-remote https://github.com/arismoko/horizon.git HEAD | head -n 1
echo
echo "--- repo ---"
test -f project/Cargo.toml
test -f scripts/validate.sh
echo "repo checkout OK"
echo
echo "--- validation smoke test ---"
bash scripts/validate.sh
