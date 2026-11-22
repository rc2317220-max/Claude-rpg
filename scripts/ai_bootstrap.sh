#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PIN="ff303502a4f7982551a0e0d3622d9268efe9e829"
TOOLS="${RUSTPG_AI_TOOLS_DIR:-$ROOT/.ai-tools}"
HORIZON="$TOOLS/horizon"
BIN="$TOOLS/bin"
FULL=0
CHECK_ONLY=0
for arg in "$@"; do
  case "$arg" in
    --full) FULL=1 ;;
    --check) CHECK_ONLY=1 ;;
    -h|--help)
      cat <<'EOF'
Usage: bash scripts/ai_bootstrap.sh [--check] [--full]
  default  prepare pinned horizon-ctl + AI context
  --check  diagnose only, no download/build
  --full   also run RustPG validation/check/tests against pinned Horizon
EOF
      exit 0 ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

if [ "$CHECK_ONLY" = 1 ]; then
  exec python3 "$ROOT/tools/ai_doctor.py"
fi

mkdir -p "$TOOLS" "$BIN"

# Exact Linux headers used by Horizon CI for the client.
if command -v apt-get >/dev/null 2>&1 && command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
  sudo apt-get update
  sudo apt-get install -y --no-install-recommends libasound2-dev libudev-dev
fi

for cmd in git cargo rustc python3; do
  command -v "$cmd" >/dev/null 2>&1 || { echo "missing required tool: $cmd" >&2; exit 3; }
done

if [ ! -d "$HORIZON/.git" ]; then
  git clone --filter=blob:none https://github.com/arismoko/horizon.git "$HORIZON"
fi
git -C "$HORIZON" fetch --depth=1 origin "$PIN"
git -C "$HORIZON" checkout --detach "$PIN"

if [ ! -x "$BIN/horizon-ctl" ] || [ ! -f "$TOOLS/horizon-ctl.pin" ] || [ "$(cat "$TOOLS/horizon-ctl.pin")" != "$PIN" ]; then
  cargo build --manifest-path "$HORIZON/Cargo.toml" --profile fast -p horizon-client-lib --bin horizon-ctl
  cp "$HORIZON/target/fast/horizon-ctl" "$BIN/horizon-ctl"
  chmod +x "$BIN/horizon-ctl"
  printf "%s\n" "$PIN" > "$TOOLS/horizon-ctl.pin"
fi

cat > "$TOOLS/env.sh" <<EOF
export RUSTPG_ROOT="$ROOT"
export HORIZON_SOURCE="$HORIZON"
export HORIZON_CTL="$BIN/horizon-ctl"
export PATH="$BIN:$PATH"
EOF

python3 "$ROOT/scripts/generate_ai_context.py"

if [ "$FULL" = 1 ]; then
  HORIZON_SOURCE="$HORIZON" bash "$ROOT/scripts/rustpg_build.sh"
fi

echo
echo "Prepared RustPG AI environment."
echo "Run: source .ai-tools/env.sh"
python3 "$ROOT/tools/ai_doctor.py"
