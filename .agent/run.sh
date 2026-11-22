#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

echo "=== Horizon launcher CLI probe ==="
work=/tmp/horizon-launcher-probe
rm -rf "$work"
mkdir -p "$work"
cd "$work"

url='https://github.com/arismoko/horizon/releases/download/launcher%400.2.10/horizon-fork-launcher-linux-x86_64.tar.gz'
curl -L --fail --retry 3 -o launcher.tar.gz "$url"
tar -xzf launcher.tar.gz
find . -maxdepth 3 -type f -printf '%p %s bytes\n' | sort | head -80

launcher="$(find . -maxdepth 3 -type f -name 'horizon-launcher' -o -name 'horizon-fork-launcher' | head -1)"
if [ -z "$launcher" ]; then
  echo "launcher executable not found" >&2
  exit 2
fi
chmod +x "$launcher"

echo "=== --help ==="
"$launcher" --help | head -120

echo "=== agent-config ==="
"$launcher" agent-config | head -120 || true

echo "=== account sign-in help ==="
"$launcher" account sign-in --help | head -80 || true

echo "LAUNCHER_PATH=$(realpath "$launcher")"
echo "Horizon launcher CLI probe OK"
