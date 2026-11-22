#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT="$ROOT/project"
python3 "$ROOT/scripts/generate_ai_context.py"
python3 "$ROOT/tools/ai_doctor.py" --require-context >/dev/null
python3 "$ROOT/scripts/generate_creatures.py"
python3 - "$PROJECT" <<'PY'
from pathlib import Path
import json,re,sys,tomllib
root=Path(sys.argv[1])
cargo=tomllib.loads((root/'Cargo.toml').read_text())
manifest=tomllib.loads((root/'horizon-mod.toml').read_text())
assert cargo['package']['name']=='mod-rustpg'
assert cargo['package']['version']=='6.1.0-alpha'
assert 'features' not in cargo
assert manifest['id']=='rustpg' and manifest['name']=='RustPG'
assert manifest['version']=='6.1.0-alpha'
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
for required in ['key.keyboard.v','fn input_key','GuiEntity::Existing','GuiEntity::kind','mod rustpg_rifts;','mod rustpg_action_combat;','RuntimeGenerator::Void','tick_rifts(self,host,tick)']:
    assert required in lib,required
rifts=(root/'src/rustpg_rifts.rs').read_text()
for required in ['RESONANCE_READY','modifier_commands','frenzy_commands','assess_plan','RoomState','DungeonSession']:
    assert required in rifts,required
creatures=(root/'src/rustpg_native_entities.rs').read_text()
for mob in ['dusk_revenant','storm_gryphon','rune_colossus','ember_imp','abyss_weaver']:
    assert f'rustpg:{mob}' in creatures,mob
print(f'Validation OK: {len(mods)} Rust modules, RustPG 6.1.0-alpha stable public API only')
PY
python3 "$ROOT/scripts/package_native.py"
