#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MODELS="$ROOT/project/models"
mkdir -p "$MODELS"
exec npx -y github:jasonjgardner/blockbench-mcp-plugin#97a8a80d3a37bf95c5e136d93d2c7da399fff753 --root "$MODELS" "$@"
