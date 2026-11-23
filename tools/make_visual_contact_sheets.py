#!/usr/bin/env python3
"""Build labelled SVG contact sheets from RustPG visual-lab PNG outputs."""
from __future__ import annotations
import argparse, base64, html, math
from pathlib import Path

def uri(path:Path)->str:
    return "data:image/png;base64,"+base64.b64encode(path.read_bytes()).decode("ascii")

def sheet(files:list[Path], out:Path, title:str, cols:int=4, thumb_w:int=220, thumb_h:int=140):
    if not files:return
    gap=14; label_h=19; top=34
    rows=math.ceil(len(files)/cols)
    width=gap+cols*(thumb_w+gap)
    height=top+rows*(thumb_h+label_h+gap)
    parts=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
           '<rect width="100%" height="100%" fill="#080d13"/>',
           f'<text x="{gap}" y="22" fill="#d7e3e7" font-family="monospace" font-size="14">{html.escape(title)}</text>']
    for i,p in enumerate(files):
        row,col=divmod(i,cols); x=gap+col*(thumb_w+gap); y=top+row*(thumb_h+label_h+gap)
        parts.append(f'<rect x="{x-2}" y="{y-2}" width="{thumb_w+4}" height="{thumb_h+4}" fill="#111b24" stroke="#547184"/>')
        parts.append(f'<image href="{uri(p)}" x="{x}" y="{y}" width="{thumb_w}" height="{thumb_h}" preserveAspectRatio="xMidYMid meet"/>')
        parts.append(f'<text x="{x}" y="{y+thumb_h+14}" fill="#91a0a7" font-family="monospace" font-size="10">{html.escape(p.stem)}</text>')
    parts.append('</svg>')
    out.parent.mkdir(parents=True,exist_ok=True)
    out.write_text("\n".join(parts),encoding="utf-8")

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("ui",type=Path)
    ap.add_argument("entities",type=Path)
    ap.add_argument("out",type=Path)
    args=ap.parse_args()
    for label in ("compact","phone","wide","ultrawide"):
        files=sorted(args.ui.glob(f"*-{label}.png"))
        sheet(files,args.out/f"ui-{label}.svg",f"RustPG UI · {label}",4)
    creatures=sorted(p for p in args.entities.iterdir() if p.is_dir()) if args.entities.exists() else []
    for pose in ("idle-three_quarter","action-three_quarter","move-three_quarter"):
        files=[p/f"{pose}.png" for p in creatures if (p/f"{pose}.png").exists()]
        sheet(files,args.out/f"entities-{pose}.svg",f"RustPG creatures · {pose}",4,180,180)

if __name__=="__main__":main()
