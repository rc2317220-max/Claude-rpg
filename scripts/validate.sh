#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT="$ROOT/project"

echo '[1/6] Manifestos obrigatórios'
test -f "$PROJECT/Cargo.toml"
test -f "$PROJECT/horizon-mod.toml"
test -f "$PROJECT/src/lib.rs"

echo '[2/6] JSON'
python3 - "$PROJECT" <<'PY'
import json, pathlib, sys
root=pathlib.Path(sys.argv[1])
files=list(root.rglob('*.json'))
for p in files:
    with p.open('r',encoding='utf-8') as f: json.load(f)
print(f'JSON válidos: {len(files)}')
PY

echo '[3/6] Rust format (checagem)'
if command -v cargo >/dev/null 2>&1; then
  (cd "$PROJECT" && cargo fmt --all -- --check) || echo 'Aviso: rustfmt encontrou diferenças ou o workspace usa dependências específicas do Starling.'
else
  echo 'cargo não disponível; pulando.'
fi

echo '[4/6] Referências de módulos'
python3 - "$PROJECT" <<'PY'
import pathlib,re,sys
root=pathlib.Path(sys.argv[1]); lib=(root/'src/lib.rs').read_text(encoding='utf-8')
mods=re.findall(r'(?m)^\s*(?:pub\s+)?mod\s+([A-Za-z0-9_]+)\s*;',lib)
missing=[m for m in mods if not (root/'src'/f'{m}.rs').exists() and not (root/'src'/m/'mod.rs').exists()]
print('Módulos declarados:',len(mods))
if missing:
    raise SystemExit('Módulos ausentes: '+', '.join(missing))
PY

echo '[5/6] Auditoria das árvores importadas'
python3 "$ROOT/scripts/audit_skill_trees.py"

echo '[6/6] Integridade estrutural concluída'
echo 'OK — isso não substitui a compilação nativa/teste no Starling.'
