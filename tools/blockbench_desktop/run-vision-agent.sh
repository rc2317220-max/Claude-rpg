#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
LAB="$ROOT/.ai-tools/vision-agent"
OUT="$ROOT/.agent/artifacts/blockbench-desktop"
mkdir -p "$LAB" "$OUT"
cd "$LAB"
curl -fL --retry 3 --max-time 180 -o llama.zip https://github.com/ggml-org/llama.cpp/releases/download/b6500/llama-b6500-bin-ubuntu-x64.zip
echo "716803084e565fe779bc2d01ea576c418be141f8fabac97509bc6d7c3cd786b2  llama.zip" | sha256sum -c -
unzip -q llama.zip -d bin
SERVER=$(find "$LAB/bin" -type f -name llama-server -print -quit)
test -x "$SERVER"
export LD_LIBRARY_PATH="$(dirname "$SERVER"):${LD_LIBRARY_PATH:-}"
BASE=https://huggingface.co/ggml-org/Qwen2.5-VL-3B-Instruct-GGUF/resolve/5037fcf
curl -fL --retry 3 --max-time 600 -o model.gguf "$BASE/Qwen2.5-VL-3B-Instruct-Q4_K_M.gguf"
curl -fL --retry 3 --max-time 600 -o mmproj.gguf "$BASE/mmproj-Qwen2.5-VL-3B-Instruct-Q8_0.gguf"
sha256sum model.gguf mmproj.gguf > "$OUT/vision-model-sha256.txt"
"$SERVER" -m model.gguf --mmproj mmproj.gguf --host 127.0.0.1 --port 8091 -c 8192 -t "$(nproc)" -tb "$(nproc)" -np 1 -ngl 0 >"$OUT/vision-server.log" 2>&1 &
PID=$!
trap 'kill "$PID" 2>/dev/null || true' EXIT
for i in $(seq 1 180); do
  if curl -fsS http://127.0.0.1:8091/health >/dev/null 2>&1; then break; fi
  kill -0 "$PID" || { tail -60 "$OUT/vision-server.log"; exit 1; }
  sleep 2
done
curl -fsS http://127.0.0.1:8091/health
cd "$ROOT"
bash tools/blockbench_desktop/run.sh
