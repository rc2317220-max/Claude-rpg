#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT="$ROOT/project"
python3 - "$PROJECT" <<'PY'
from pathlib import Path
import json,re,sys,tomllib
root=Path(sys.argv[1])
cargo=tomllib.loads((root/'Cargo.toml').read_text())
manifest=tomllib.loads((root/'horizon-mod.toml').read_text())
assert cargo['package']['name']=='mod-rustpg'
assert 'features' not in cargo
assert manifest['id']=='rustpg' and manifest['name']=='RustPG'
assert not (set(manifest.get('native',{})) & {'mixin','bridge','module','state','injection'})
for p in root.rglob('*.json'):json.loads(p.read_text())
lib=(root/'src/lib.rs').read_text()
mods=re.findall(r'(?m)^mod\s+([A-Za-z0-9_]+)\s*;',lib)
missing=[m for m in mods if not (root/'src'/f'{m}.rs').is_file() and not (root/'src'/m/'mod.rs').is_file()]
assert not missing,missing
patterns=[r'whole_game_mixins',r'hnmod',r'\[\[native\.(?:mixin|bridge|module|state|injection)',r'rustpg_(?:quests|campaign|stack_rules|block_meter|preview_renderer|puffish_graph)',r'inventory\.stack_']
for p in list((root/'src').rglob('*.rs'))+[root/'Cargo.toml',root/'horizon-mod.toml']:
    text=p.read_text()
    for pat in patterns:assert not re.search(pat,text,re.I),(p,pat)
    assert not re.search(r'\bquests?\b|\bcampaign\b|miss(?:ão|ao|ões|oes)|\bdaily\b',text,re.I),p
assert 'key.keyboard.v' in lib and 'fn input_key' in lib
assert 'GuiEntity::Existing' in lib and 'GuiEntity::kind' in lib
print(f'Validation OK: {len(mods)} Rust modules, stable public API only')
PY
python3 "$ROOT/scripts/package_native.py"
