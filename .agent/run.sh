#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

STARLING="$PWD/.ai-tools/starling-0.9.0"
STARLING_SHA=ff303502a4f7982551a0e0d3622d9268efe9e829
mkdir -p .ai-tools .agent/artifacts/starling-runtime-lab

echo "=== runner capabilities ==="
uname -a
uname -m
lscpu | sed -n '1,35p' || true
free -h || true
ls -l /dev/kvm 2>&1 || true
command -v vulkaninfo || true
command -v xvfb-run || true
command -v Xvfb || true
command -v adb || true
command -v emulator || true
rustc --version || true
cargo --version || true
java -version 2>&1 | head -5 || true

echo "=== fetch Starling 0.9.0 exact source ==="
if [ ! -d "$STARLING/.git" ]; then
  git init "$STARLING"
  git -C "$STARLING" remote add origin https://github.com/starling-client/starling.git
fi
git -C "$STARLING" fetch --depth=1 origin "$STARLING_SHA"
git -C "$STARLING" checkout --detach FETCH_HEAD
git -C "$STARLING" rev-parse HEAD

echo "=== locate account-free runtime and downloader ==="
grep -R -n --exclude-dir=target -- "--developer-single-player" "$STARLING/crates" "$STARLING/launcher" | head -40 || true
grep -R -n --exclude-dir=target "pub fn download_files\|fn download_files\|download_files()" "$STARLING/crates" | head -80 || true
grep -R -n --exclude-dir=target "PlayOffline\|OfflineProfile\|files are fetched without a sign-in" "$STARLING/crates/horizon-client/src" | head -100 || true

echo "=== package map ==="
cargo metadata --manifest-path "$STARLING/Cargo.toml" --no-deps --format-version 1 > .agent/artifacts/starling-runtime-lab/metadata.json
python3 - <<'PY'
import json
m=json.load(open('.agent/artifacts/starling-runtime-lab/metadata.json'))
for p in m['packages']:
    if any(x in p['name'] for x in ['horizon-client','horizon-prepare','starling']):
        print(p['name'], p['manifest_path'])
PY

echo "=== launch api source ==="
grep -R -n --exclude-dir=target "developer_single_player\|developer-single-player" "$STARLING/crates/horizon-client/src" | head -80 || true

echo "=== done ==="
