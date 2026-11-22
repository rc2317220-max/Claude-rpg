#!/usr/bin/env python3
"""Drive a real Horizon client and capture RustPG visual evidence.

This is not a mock renderer. It talks to Horizon's own horizon-ctl control socket.
For pixel-accurate screenshots, attach to (or launch) a native Horizon build that
already contains RustPG. Use --offscreen, not --headless: headless has no renderer.
"""
from __future__ import annotations
import argparse, json, os, subprocess, sys, time
from pathlib import Path

def call(ctl: str, *args: str, check: bool = True):
    cmd=[ctl,*map(str,args)]
    p=subprocess.run(cmd,text=True,capture_output=True)
    raw=(p.stdout or "").strip()
    if check and p.returncode:
        raise RuntimeError(f"{' '.join(cmd)} failed ({p.returncode})\n{p.stderr}\n{raw}")
    try:
        return json.loads(raw) if raw else {"ok": p.returncode==0}
    except json.JSONDecodeError:
        return {"ok": p.returncode==0,"stdout":raw,"stderr":(p.stderr or "").strip()}

def save_json(path:Path,value):
    path.write_text(json.dumps(value,ensure_ascii=False,indent=2),encoding="utf-8")

def find_entity_id(value,kind:str):
    if isinstance(value,dict):
        k=str(value.get("kind") or value.get("type") or value.get("entity_type") or "")
        if kind in k and isinstance(value.get("id"),int):
            return value["id"]
        for v in value.values():
            found=find_entity_id(v,kind)
            if found is not None:return found
    elif isinstance(value,list):
        for v in value:
            found=find_entity_id(v,kind)
            if found is not None:return found
    return None

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--ctl",default=os.environ.get("HORIZON_CTL","horizon-ctl"))
    ap.add_argument("--game",help="native Horizon client executable already built with RustPG")
    ap.add_argument("--game-dir",default=os.environ.get("HORIZON_GAME_DIR",".visual-game"))
    ap.add_argument("--world",default="RustPGVisual")
    ap.add_argument("--output",default="visual-artifacts")
    ap.add_argument("--width",type=int,default=720)
    ap.add_argument("--height",type=int,default=1280)
    ap.add_argument("--attach",action="store_true",help="do not launch/quit; use an already controlled game")
    args=ap.parse_args()

    out=Path(args.output).resolve();out.mkdir(parents=True,exist_ok=True)
    report={"exact_renderer":True,"mode":"attach" if args.attach else "launch","steps":[]}

    if not args.attach:
        if not args.game:
            ap.error("--game is required unless --attach is used")
        Path(args.game_dir).mkdir(parents=True,exist_ok=True)
        launch=[
            "launch","--game",str(Path(args.game).resolve()),
            "--world",args.world,"--creative","--wait","--offscreen",
            "--developer-single-player",
            "--game-dir",str(Path(args.game_dir).resolve()),
            "--width",str(args.width),"--height",str(args.height),
        ]
        report["launch"]=call(args.ctl,*launch)

    try:
        report["hello"]=call(args.ctl,"hello")
        report["ready"]=call(args.ctl,"wait-ready","120")
        report["status"]=call(args.ctl,"status")
        call(args.ctl,"screenshot",str(out/"00-world.png"))
        report["steps"].append("world screenshot")

        # The real RustPG V key path (normal key mapping + input_key fallback).
        call(args.ctl,"key","v","tap")
        time.sleep(0.35)
        report["rustpg_ui"]=call(args.ctl,"ui",check=False)
        save_json(out/"01-rustpg-ui-widgets.json",report["rustpg_ui"])
        call(args.ctl,"screenshot",str(out/"01-rustpg-menu.png"))
        report["steps"].append("V-opened RustPG UI screenshot")

        # Return to gameplay; one of these will be appropriate depending on screen type.
        call(args.ctl,"key","esc","tap",check=False)
        call(args.ctl,"close",check=False)
        time.sleep(0.15)

        # Real renderer / target-HUD scene with an original RustPG creature.
        target_kind="rustpg:storm_gryphon"
        summon=call(args.ctl,"summon",target_kind,check=False)
        if not summon.get("ok",False):
            target_kind="rustpg:mana_hound"
            summon=call(args.ctl,"summon",target_kind,check=False)
        report["summon"]={"kind":target_kind,"result":summon}
        time.sleep(0.25)
        entities=call(args.ctl,"entities","32",target_kind,check=False)
        save_json(out/"02-entities.json",entities)
        eid=find_entity_id(entities,target_kind)
        if eid is not None:
            call(args.ctl,"look-at-entity",str(eid),check=False)
            time.sleep(0.25)
            call(args.ctl,"screenshot",str(out/"02-target-hud.png"))
            report["target_entity_id"]=eid
            # One real attack lets the damage/target HUD update through the actual game hook.
            call(args.ctl,"attack",str(eid),check=False)
            time.sleep(0.25)
            call(args.ctl,"screenshot",str(out/"03-after-attack.png"))
            report["steps"].append("custom creature + target/damage HUD")
        else:
            report["target_warning"]="custom creature did not appear in entities output"

        report["rift_status"]=call(args.ctl,"command","rustpg rift status",check=False)
        report["messages"]=call(args.ctl,"messages","20",check=False)
        report["events"]=call(args.ctl,"events","--limit","50",check=False)
        report["logs"]=call(args.ctl,"logs","--lines","160",check=False)
    finally:
        save_json(out/"visual-report.json",report)
        if not args.attach:
            call(args.ctl,"quit",check=False)

    print(out)
    return 0

if __name__=="__main__":
    raise SystemExit(main())
