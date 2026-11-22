#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT="$ROOT/project"

echo '[1/8] Manifestos obrigatórios'
test -f "$PROJECT/Cargo.toml"
test -f "$PROJECT/horizon-mod.toml"
test -f "$PROJECT/src/lib.rs"

echo '[2/8] JSON'
python3 - "$PROJECT" <<'PY'
import json, pathlib, sys
root=pathlib.Path(sys.argv[1])
files=list(root.rglob('*.json'))
for p in files:
    with p.open('r',encoding='utf-8') as f: json.load(f)
print(f'JSON válidos: {len(files)}')
PY

echo '[3/8] Rust format (checagem)'
if command -v cargo >/dev/null 2>&1; then
  (cd "$PROJECT" && cargo fmt --all -- --check) || echo 'Aviso: rustfmt encontrou diferenças ou o workspace usa dependências específicas do Starling.'
else
  echo 'cargo não disponível; pulando.'
fi

echo '[4/8] Referências de módulos'
python3 - "$PROJECT" <<'PY'
import pathlib,re,sys
root=pathlib.Path(sys.argv[1]); lib=(root/'src/lib.rs').read_text(encoding='utf-8')
mods=re.findall(r'(?m)^\s*(?:pub\s+)?mod\s+([A-Za-z0-9_]+)\s*;',lib)
missing=[m for m in mods if not (root/'src'/f'{m}.rs').exists() and not (root/'src'/m/'mod.rs').exists()]
print('Módulos declarados:',len(mods))
if missing:
    raise SystemExit('Módulos ausentes: '+', '.join(missing))
PY

echo '[5/8] Auditoria das árvores importadas'
python3 "$ROOT/scripts/audit_skill_trees.py"

echo '[6/8] Auditoria de assets GUI'
python3 "$ROOT/scripts/audit_assets.py"

echo '[7/8] Verificação dos modelos Blockbench incorporados ao renderizador'
python3 "$ROOT/scripts/build_creature_model.py" --check
if [ -f "$PROJECT/models/creatures/rift_hunter.bbmodel" ]; then
  python3 "$ROOT/scripts/build_creature_model.py" \
    --model "$PROJECT/models/creatures/rift_hunter.bbmodel" \
    --rust "$PROJECT/src/rustpg_generated_hunter.rs" \
    --texture "$PROJECT/assets/rustpg/textures/entity/rift_hunter.png" \
    --prefix HUNTER --check
fi

echo '[8/8] Integridade estrutural concluída'
echo 'OK — isso não substitui a compilação nativa/teste no Starling.'
