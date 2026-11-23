#!/usr/bin/env bash
set -euo pipefail
echo "=== audit pinned Starling/Horizon GUI entity preview implementation ==="
git clone --filter=blob:none --no-checkout https://github.com/arismoko/horizon.git .agent/engine-probe
git -C .agent/engine-probe fetch --depth=1 origin ff303502a4f7982551a0e0d3622d9268efe9e829
git -C .agent/engine-probe checkout --detach ff303502a4f7982551a0e0d3622d9268efe9e829
for term in 'GuiEntity::Existing' 'GuiEntity::Kind' 'GuiEntityView' 'GuiEntityError' 'register_entity_renderers' 'EntityRenderer::submit'; do
  echo "=== SEARCH: $term ==="
  git -C .agent/engine-probe grep -n -F "$term" -- 'crates/**/*.rs' 2>/dev/null | head -25 || true
done
echo "=== SDK GUI PROBE COMPLETE ==="
