#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
D="$ROOT/.ai-tools/asset-studio"
PIN="97a8a80d3a37bf95c5e136d93d2c7da399fff753"
mkdir -p "$D"
if [[ "${1:-}" != "--no-apt" ]]; then
  sudo apt-get update
  sudo apt-get install -y --no-install-recommends blender ffmpeg python3-venv
fi
for x in git node npm blender ffmpeg python3; do command -v "$x" >/dev/null || { echo "Missing $x" >&2; exit 3; }; done
if [[ ! -d "$D/blockbench/.git" ]]; then
  git clone --no-checkout https://github.com/jasonjgardner/blockbench-mcp-plugin.git "$D/blockbench"
fi
git -C "$D/blockbench" fetch --depth=1 origin "$PIN"
git -C "$D/blockbench" checkout --detach "$PIN"
test "$(git -C "$D/blockbench" rev-parse HEAD)" = "$PIN"
# Local npm installation (no global executable, no public listener).
if [[ ! -x "$D/node_modules/.bin/bun" ]]; then
  npm install --prefix "$D" --no-audit --no-fund bun@1.3.8
fi
export PATH="$D/node_modules/.bin:$PATH"
(cd "$D/blockbench" && bun install --frozen-lockfile)
python3 -m venv "$D/venv"
"$D/venv/bin/pip" install --disable-pip-version-check 'numpy==2.2.6' 'Pillow==11.3.0'
node --version
bun --version
blender --version | head -n 1
ffmpeg -version | head -n 1
"$D/venv/bin/python" -c 'import PIL,numpy; print("Pillow",PIL.__version__,"numpy",numpy.__version__)'
bash "$ROOT/tools/asset_studio/bb.sh" --list
