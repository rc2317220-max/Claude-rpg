#!/usr/bin/env python3
"""Verify that statically referenced RustPG GUI sprites exist and report asset coverage."""
from pathlib import Path
import re
import sys

root=Path(__file__).resolve().parents[1]
src=root/"project/src"
assets=root/"project/assets/rustpg/textures"
pattern=re.compile(r'["\'](rustpg:gui/[a-z0-9_./-]+)["\']')
refs=set()
by_file={}
for path in src.glob("*.rs"):
    text=path.read_text(encoding="utf-8")
    found=set(pattern.findall(text))
    if found:
        by_file[path.name]=found
        refs.update(found)

missing=[]
for key in sorted(refs):
    rel=key.split(":",1)[1]
    path=assets/(rel+".png")
    if not path.is_file():
        missing.append((key,str(path.relative_to(root))))

all_png={f"rustpg:{p.relative_to(assets).with_suffix('').as_posix()}" for p in assets.rglob("*.png")}
used_existing=refs & all_png
unused=all_png-refs
print(f"GUI assets: {len(all_png)} PNGs; {len(used_existing)} referenced statically; {len(unused)} not statically referenced.")
print(f"Rust files with GUI references: {len(by_file)}")
if unused:
    sample=", ".join(sorted(unused)[:12])
    print(f"Unused/static-dynamic sample: {sample}")
if missing:
    print("Missing referenced sprites:",file=sys.stderr)
    for key,path in missing:
        print(f"  {key} -> {path}",file=sys.stderr)
    raise SystemExit(1)
