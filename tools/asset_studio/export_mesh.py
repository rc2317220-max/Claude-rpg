#!/usr/bin/env python3
"""Export the existing independent inspector's cuboid/UV geometry to Blender."""
import argparse,json,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from creature_inspector import authored,geometry,parse_generated
p=argparse.ArgumentParser();p.add_argument("--model",type=Path);p.add_argument("--creature");p.add_argument("--project",type=Path,default=Path("project"));p.add_argument("--clip");p.add_argument("--time",type=float,default=0);p.add_argument("--output",type=Path,required=True);a=p.parse_args()
if bool(a.model)==bool(a.creature):p.error("Choose exactly one model or generated creature")
rig=authored(a.model) if a.model else parse_generated(a.project)[a.creature]
polys=geometry(rig,a.clip,a.time,"authored" if a.model else "runtime")
a.output.parent.mkdir(parents=True,exist_ok=True)
a.output.write_text(json.dumps({"mode":"authored_uv" if a.model else "legacy_runtime_full_face","faces":[{"vertices":pts.tolist(),"uv":uv.tolist()} for pts,uv,_ in polys]}))
