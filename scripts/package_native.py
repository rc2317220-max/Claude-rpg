#!/usr/bin/env python3
"""Package the single stable-API RustPG source bundle for Starling Compile v2."""
from __future__ import annotations
import pathlib, re, tomllib, zipfile
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
    files={}
    for item in PROJECT.rglob('*'):
        if not item.is_file() or item.is_symlink():continue
        rel=item.relative_to(PROJECT).as_posix()
        if set(item.relative_to(PROJECT).parts)&{'.git','.horizon','target','__pycache__'}:continue
        if accepted(rel):files[rel]=item.read_bytes()
    required={'Cargo.toml','horizon-mod.toml','src/lib.rs'}
    if not required<=files.keys():raise SystemExit('missing required native source')
    cargo=tomllib.loads(files['Cargo.toml'].decode())
    manifest=tomllib.loads(files['horizon-mod.toml'].decode())
    if 'features' in cargo:raise SystemExit('Compile-v2 package must not contain Cargo features')
    native=manifest.get('native',{})
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
