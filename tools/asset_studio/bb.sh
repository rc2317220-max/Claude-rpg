#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
D="$ROOT/.ai-tools/asset-studio"
export PATH="$D/node_modules/.bin:$PATH"
export BB_RENDER_HOME="$D/render-cache"
WORK="${RUSTPG_ASSET_ROOT:-$ROOT/project/models}"
test -d "$WORK"
test -f "$D/blockbench/headless/call.ts" || { echo "Run bash scripts/asset_studio_setup.sh first" >&2; exit 3; }
if [[ "${1:-}" = serve ]]; then
  shift
  exec bun run "$D/blockbench/headless/index.ts" --root "$WORK" --root "$ROOT/project/assets" "$@"
fi
exec bun run "$D/blockbench/headless/call.ts" --root "$WORK" --root "$ROOT/project/assets" "$@"
