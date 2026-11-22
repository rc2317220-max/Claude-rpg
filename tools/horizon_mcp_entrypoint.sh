#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if command -v horizon-launcher >/dev/null 2>&1; then
  exec horizon-launcher mcp
fi
if command -v horizon-ctl >/dev/null 2>&1; then
  exec horizon-ctl mcp
fi
if [ -x "$ROOT/.ai-tools/bin/horizon-ctl" ]; then
  exec "$ROOT/.ai-tools/bin/horizon-ctl" mcp
fi

cat >&2 <<'EOF'
RustPG Horizon MCP is not prepared.
Run:
  bash scripts/ai_bootstrap.sh
  source .ai-tools/env.sh
Then restart/reconnect the MCP client.
EOF
exit 127
