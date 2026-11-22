#!/usr/bin/env bash
set -euo pipefail

echo "=== clean-runner AI bootstrap acceptance test ==="
bash scripts/ai_bootstrap.sh --full
source .ai-tools/env.sh

echo "=== horizon-ctl smoke ==="
horizon-ctl --help | head -40

echo "=== AI doctor ==="
python3 tools/ai_doctor.py --json --require-context

echo "=== MCP entrypoint availability ==="
test -x .ai-tools/bin/horizon-ctl
test -f .mcp.json
echo "AI bootstrap acceptance OK"
