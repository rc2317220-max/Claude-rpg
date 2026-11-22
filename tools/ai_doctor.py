#!/usr/bin/env python3
from __future__ import annotations
import argparse, hashlib, json, os, re, shutil, subprocess
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
PIN="ff303502a4f7982551a0e0d3622d9268efe9e829"
UI_FILES=[
"project/src/lib.rs","project/src/rustpg_ui_window.rs","project/src/rustpg_ui_layout.rs",
"project/src/rustpg_ui_system.rs","project/src/rustpg_ui_theme.rs","project/src/rustpg_ui_widgets.rs",
"project/src/rustpg_ui_input.rs","project/src/rustpg_ui_animation.rs","project/src/rustpg_hud.rs",
"project/src/rustpg_target_hud.rs","project/src/rustpg_boss_ui.rs","project/src/rustpg_bestiary.rs",
"project/src/rustpg_view.rs","project/src/rustpg_inspector.rs","project/src/rustpg_nodes.rs",
"project/src/rustpg_skillgraph.rs",
]

def which(name):
    p=shutil.which(name)
    if p:return p
    local=ROOT/".ai-tools/bin"/name
    return str(local) if local.is_file() and os.access(local,os.X_OK) else None

def run(*cmd):
    try:return subprocess.check_output(cmd,cwd=ROOT,text=True,stderr=subprocess.DEVNULL).strip()
    except Exception:return None

def ui_digest():
    h=hashlib.sha256()
    for rel in UI_FILES:
        p=ROOT/rel
        if not p.is_file():continue
        h.update(rel.encode());h.update(b"\0");h.update(p.read_bytes());h.update(b"\0")
    return h.hexdigest()

def bundle_digest():
    p=ROOT/"AI_UI_SOURCE_BUNDLE.txt"
    if not p.is_file():return None
    m=re.search(r"^Source-SHA256:\s*([0-9a-f]{64})\s*$",p.read_text(errors="replace"),re.M)
    return m.group(1) if m else None

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--json",action="store_true")
    a=ap.parse_args()
    cargo=which("cargo"); rustc=which("rustc"); python=which("python3"); git=which("git")
    launcher=which("horizon-launcher"); ctl=which("horizon-ctl")
    game=os.environ.get("HORIZON_GAME","")
    game_ok=bool(game and Path(game).is_file())
    src=ROOT/".ai-tools/horizon"
    src_pin=run("git","-C",str(src),"rev-parse","HEAD") if src.is_dir() else None
    current=ui_digest(); bundled=bundle_digest()
    data={
      "repo":str(ROOT),
      "git_head":run("git","rev-parse","HEAD"),
      "git_branch":run("git","branch","--show-current"),
      "tools":{"git":git,"python3":python,"cargo":cargo,"rustc":rustc,
               "horizon_launcher":launcher,"horizon_ctl":ctl},
      "horizon":{"required_pin":PIN,"source":str(src) if src.is_dir() else None,
                 "source_pin":src_pin,"pin_matches":src_pin==PIN if src_pin else False},
      "context":{"ui_source_sha256":current,"bundle_sha256":bundled,
                 "ui_bundle_current":bool(bundled and bundled==current)},
      "game":{"HORIZON_GAME":game or None,"executable_present":game_ok},
    }
    blockers=[]
    if not all((git,python,cargo,rustc)):blockers.append("basic build tools missing")
    if not (launcher or ctl):blockers.append("Horizon MCP/CLI missing; run scripts/ai_bootstrap.sh")
    if not game_ok:blockers.append("no prepared Horizon game executable in HORIZON_GAME; visual screenshots unavailable")
    if not data["context"]["ui_bundle_current"]:blockers.append("AI_UI_SOURCE_BUNDLE.txt is stale; run scripts/generate_ai_context.py")
    data["build_ready"]=all((git,python,cargo,rustc))
    data["mcp_ready"]=bool(launcher or ctl)
    data["visual_ready"]=bool((launcher or ctl) and game_ok)
    data["blockers"]=blockers
    if a.json:
        print(json.dumps(data,ensure_ascii=False,indent=2))
    else:
        print("RustPG AI doctor")
        print("================")
        print(f"build_ready : {data['build_ready']}")
        print(f"mcp_ready   : {data['mcp_ready']}")
        print(f"visual_ready: {data['visual_ready']}")
        print(f"git_head    : {data['git_head']}")
        print(f"horizon_ctl : {ctl or '-'}")
        print(f"launcher    : {launcher or '-'}")
        print(f"HORIZON_GAME: {game or '-'}")
        print(f"UI bundle   : {'current' if data['context']['ui_bundle_current'] else 'STALE'}")
        if blockers:
            print("\nBlockers:")
            for b in blockers:print(f"- {b}")
    return 0

if __name__=="__main__":
    raise SystemExit(main())
