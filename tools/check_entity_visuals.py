#!/usr/bin/env python3
"""Measure RustPG entity visual captures for obvious regressions."""
from __future__ import annotations
import argparse,json,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"scripts"))
from remaster_handcrafted_creatures import decode

def measure(path:Path):
    w,h,color,rows=decode(path.read_bytes())
    bpp=4 if color==6 else 3
    count=0; total=0.0; xs=[]; ys=[]
    for y,row in enumerate(rows):
        for x in range(w):
            at=x*bpp
            r,g,b=row[at:at+3]
            a=row[at+3] if bpp==4 else 255
            if a<=4 or r+g+b<=6: continue
            count+=1; xs.append(x);ys.append(y)
            total+=0.2126*r+0.7152*g+0.0722*b
    if not count:
        return {"luminance":0.0,"coverage":0.0,"bbox":[0,0,0,0]}
    return {
        "luminance":round(total/count,2),
        "coverage":round(count/(w*h),4),
        "bbox":[min(xs),min(ys),max(xs)+1,max(ys)+1],
    }

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("root",type=Path)
    ap.add_argument("--output",type=Path)
    ap.add_argument("--strict",action="store_true")
    args=ap.parse_args()
    report={}; bad=[]
    for creature in sorted(p for p in args.root.iterdir() if p.is_dir()):
        shots={}
        for png in sorted(creature.glob("*.png")):
            m=measure(png); shots[png.stem]=m
            if m["coverage"]<0.035 or m["luminance"]<28.0:
                bad.append(f"{creature.name}/{png.name}: lum={m['luminance']} cov={m['coverage']}")
        if shots: report[creature.name]=shots
    text=json.dumps(report,indent=2,ensure_ascii=False)
    if args.output:
        args.output.parent.mkdir(parents=True,exist_ok=True)
        args.output.write_text(text+"\n",encoding="utf-8")
    print(text)
    if bad:
        print("\nVisual QA warnings:",file=sys.stderr)
        print("\n".join(" - "+x for x in bad),file=sys.stderr)
        if args.strict: raise SystemExit(2)

if __name__=="__main__": main()
