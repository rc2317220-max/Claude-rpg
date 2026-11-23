#!/usr/bin/env python3
"""Package the single stable-API RustPG source bundle for Starling Compile v2."""
from __future__ import annotations
import json, pathlib, re, subprocess, sys, tomllib, zipfile
ROOT=pathlib.Path(__file__).resolve().parents[1]
PROJECT=ROOT/'project'
RELEASES=ROOT/'releases'
NAME='RustPG.hmod'
ROOT_ALLOWED={'Cargo.toml','Cargo.lock','build.rs','horizon-mod.toml','layouts.json','icon.png'}
FORBIDDEN_RE=re.compile(r'(?im)whole_game_mixins|hnmod|\[\[native\.(?:mixin|bridge|module|state|injection)\]\]|rustpg_(?:quests|campaign|stack_rules|block_meter|preview_renderer|puffish_graph)|inventory\.stack_')

def accepted(name:str)->bool:
    if name in ROOT_ALLOWED:return True
    if name.startswith('src/') and name.endswith(('.rs','.toml')):return True
    if name.startswith(('assets/','data/')):return True
    return False

def main()->None:
    subprocess.run([sys.executable,str(ROOT/'scripts/generate_creatures.py')],check=True)
    # The Starling 0.9.0 GUI draws native modded entities as their vanilla
    # templates. Package exact-mesh portraits for all twelve RustPG species.
    subprocess.run([sys.executable,str(ROOT/'scripts/generate_creature_portraits.py')],check=True)
    files={}
    for item in PROJECT.rglob('*'):
        if not item.is_file() or item.is_symlink():continue
        rel=item.relative_to(PROJECT).as_posix()
        if set(item.relative_to(PROJECT).parts)&{'.git','.horizon','target','__pycache__'}:continue
        if accepted(rel):files[rel]=item.read_bytes()
    portrait_paths=[p for p in files
                    if p.startswith('assets/rustpg/textures/gui/portraits/') and p.endswith('.png')]
    if len(portrait_paths)!=12:
        raise SystemExit(f'expected all 12 accurate mesh portraits, got {len(portrait_paths)}')
    for path in portrait_paths:
        data=files[path]
        if data[:8]!=bytes([137,80,78,71,13,10,26,10]) or int.from_bytes(data[16:20],'big')!=128:
            raise SystemExit(f'invalid original-mesh portrait: {path}')
    required={'Cargo.toml','horizon-mod.toml','src/lib.rs'}
    if not required<=files.keys():raise SystemExit('missing required native source')
    # Pinned Starling 0.9.0 only admits six living entity templates.
    # A single invalid entity rejects every RustPG registry/renderer at load time.
    living_templates={'minecraft:zombie','minecraft:wolf','minecraft:cat',
                      'minecraft:parrot','minecraft:villager','minecraft:wandering_trader'}
    creature_files=[(p,data) for p,data in files.items()
                    if p.startswith('data/rustpg/horizon/entity_type/') and p.endswith('.json')]
    if len(creature_files)!=12:
        raise SystemExit(f'expected 12 RustPG living entity types, found {len(creature_files)}')
    for path,data in creature_files:
        kind=json.loads(data)
        if kind.get('template') not in living_templates:
            raise SystemExit(f'unsupported Starling 0.9.0 entity template in {path}: {kind.get("template")!r}')
    cargo=tomllib.loads(files['Cargo.toml'].decode())
    manifest=tomllib.loads(files['horizon-mod.toml'].decode())
    if 'features' in cargo:raise SystemExit('Compile-v2 package must not contain Cargo features')
    native=manifest.get('native',{})
    expected={
        'game':'26.3',
        'horizon':'ff303502a4f7982551a0e0d3622d9268efe9e829',
        'mixins':'5a64af938ec86b00d53d16a408aabc0038fb3e57',
    }
    for field,value in expected.items():
        if native.get(field)!=value:
            raise SystemExit(f'Starling 0.9.0 native manifest requires {field}={value}')
    bad=set(native)&{'mixin','bridge','module','state','injection'}
    if bad:raise SystemExit(f'engine-patch entries are forbidden: {sorted(bad)}')
    for path,data in files.items():
        if path.endswith(('.rs','.toml')):
            text=data.decode('utf-8')
            if FORBIDDEN_RE.search(text):raise SystemExit(f'forbidden legacy feature in {path}')
    RELEASES.mkdir(exist_ok=True)
    dest=RELEASES/NAME
    with zipfile.ZipFile(dest,'w',zipfile.ZIP_DEFLATED,compresslevel=7) as z:
        for path in sorted(files):z.writestr(path,files[path])
    with zipfile.ZipFile(dest) as z:
        assert z.testzip() is None
        assert set(z.namelist())==set(files)
    print(f'RustPG stable package: {dest} ({len(files)} files)')
if __name__=='__main__':main()
