#!/usr/bin/env bash
set -euo pipefail
# Real desktop app, isolated runner profile; no public MCP port or external tunnel.
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
LAB="$ROOT/.ai-tools/blockbench-desktop"
OUT="$ROOT/.agent/artifacts/blockbench-desktop"
mkdir -p "$LAB" "$OUT"
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends xvfb xauth openbox libgtk-3-0t64 libnss3 libasound2t64 libgbm1 libgl1-mesa-dri libegl1 mesa-utils dbus-x11 fonts-dejavu-core imagemagick ffmpeg wmctrl python3-tk python3-pil.imagetk
curl --fail --location --retry 3 --output "$LAB/blockbench.deb" https://github.com/JannisX11/blockbench/releases/download/v5.2.2/Blockbench_5.2.2.deb
echo "76ea903a2d0086a112f177586efdeba9a1a87ea0218b2f1bbb4b2dc3ce177031  $LAB/blockbench.deb" | sha256sum -c -
sudo apt-get install -y "$LAB/blockbench.deb"
command -v blockbench
git init "$LAB/mcp"
git -C "$LAB/mcp" remote add origin https://github.com/RenasDemirbas/BlockBenchMCP.git
git -C "$LAB/mcp" fetch --depth=1 origin 6a54fb47bb801f6dcf7fd979eee8aad51a1f7842
git -C "$LAB/mcp" checkout --detach FETCH_HEAD
test "$(git -C "$LAB/mcp" rev-parse HEAD)" = 6a54fb47bb801f6dcf7fd979eee8aad51a1f7842
(cd "$LAB/mcp" && npm ci && npm run build)
test -s "$LAB/mcp/dist/blockbench_mcp.js"
test -s "$LAB/mcp/dist/mcp-server.js"
printf '%s\n' 'Blockbench 5.2.2 / MCP 6a54fb47bb801f6dcf7fd979eee8aad51a1f7842' > "$OUT/versions.txt"
