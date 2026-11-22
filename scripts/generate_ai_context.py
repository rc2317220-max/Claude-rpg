#!/usr/bin/env python3
"""Regenerate AI-facing repository maps for tools that cannot browse GitHub trees."""
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[1]
SRC=ROOT/"project/src"
REPO="rc2317220-max/Claude-rpg"
try:
    SHA=subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip()
except Exception:
    SHA="HEAD"
BASE=f"https://github.com/{REPO}/blob/{SHA}/"

DESCRIPTIONS={
"lib.rs":"Main integration point: Horizon hooks, attachments, payloads, UI, combat and Rifts.",
"rustpg_profile.rs":"Persistent player profile and save codec.",
"rustpg_ui_window.rs":"Main RustPG window model.",
"rustpg_ui_layout.rs":"Responsive UI/HUD geometry.",
"rustpg_ui_system.rs":"UI system orchestration.",
"rustpg_ui_widgets.rs":"Reusable widgets.",
"rustpg_ui_input.rs":"Touch/mouse/key UI input.",
"rustpg_target_hud.rs":"Target/damage HUD.",
"rustpg_boss_ui.rs":"Boss HP bar styling.",
"rustpg_bestiary.rs":"Bestiary persistent state.",
"rustpg_view.rs":"Text/pages exposed by UI and commands.",
"rustpg_nodes.rs":"Skill-tree node definitions.",
"rustpg_action_combat.rs":"Server-authoritative weapon action timing.",
"rustpg_guard.rs":"Parry/dodge/perfect defense windows.",
"rustpg_combat_pipeline.rs":"Damage resolution pipeline.",
"rustpg_rifts.rs":"Rift/dungeon domain and generation.",
"rustpg_native_entities.rs":"Custom creature render/AI registration.",
"rustpg_scaling.rs":"Mob tiers, HP and scaling.",
"rustpg_creatures_generated.rs":"Generated creature rigs; do not hand-edit.",
}

def link(path:Path)->str:
    rel=path.relative_to(ROOT).as_posix()
    return f"[{rel}]({BASE}{rel})"

files=sorted(SRC.glob("*.rs"))
groups={
"UI / HUD / Bestiary":[p for p in files if any(x in p.name for x in ("ui_","hud","bestiary","view","inspector","bar_vfx","damage_","timeline","presentation"))],
"Combat":[p for p in files if any(x in p.name for x in ("action_combat","guard","combat","combo","hitboxes","weapon_training","powers","elite_encounters"))],
"Rifts / World":[p for p in files if any(x in p.name for x in ("rifts","world_vfx"))],
"Creatures":[p for p in files if any(x in p.name for x in ("native_entities","generated_","scaling","skeletal","mob_wire"))],
"Progression":[p for p in files if any(x in p.name for x in ("profile","progression","attributes","nodes","skillgraph","gear","relic","engine_","immersive","luck","enchant"))],
}
used=set().union(*[set(v) for v in groups.values()])
groups["Other"]=[p for p in files if p not in used and p.name!="lib.rs"]

index=["# SOURCE_INDEX.md","","Clickable index for AIs that cannot browse GitHub directory pages.","",f"Generated from commit `{SHA}`.",""]
for p in files:
    d=DESCRIPTIONS.get(p.name)
    index.append(f"- {link(p)}"+(f" — {d}" if d else ""))
(ROOT/"SOURCE_INDEX.md").write_text("\n".join(index)+"\n",encoding="utf-8")

repo=["# REPO_MAP.md","","Generated architecture map.","",f"- {link(SRC/'lib.rs')} — {DESCRIPTIONS['lib.rs']}",""]
for title,paths in groups.items():
    repo += [f"## {title}",""]
    for p in paths:
        repo.append(f"- {link(p)}"+(f" — {DESCRIPTIONS[p.name]}" if p.name in DESCRIPTIONS else ""))
    repo.append("")
(ROOT/"REPO_MAP.md").write_text("\n".join(repo),encoding="utf-8")

key=[
"project/src/lib.rs","project/src/rustpg_profile.rs","project/src/rustpg_ui_window.rs",
"project/src/rustpg_target_hud.rs","project/src/rustpg_bestiary.rs","project/src/rustpg_nodes.rs",
"project/src/rustpg_action_combat.rs","project/src/rustpg_guard.rs","project/src/rustpg_rifts.rs",
"project/src/rustpg_native_entities.rs","project/src/rustpg_scaling.rs","scripts/generate_creatures.py",
".github/workflows/validate.yml",".github/workflows/build-on-demand.yml",
"VISUAL_TESTING.md","tools/horizon_visual_driver.py",".mcp.json"]
ctx=f"""RUSTPG AI CONTEXT
=================
RustPG 6.0.0-alpha — Starling/Horizon 0.9.0 — game 26.3
Horizon pin: ff303502a4f7982551a0e0d3622d9268efe9e829
Canonical package: releases/RustPG.hmod

Read CLAUDE.md first. If directory listing is unavailable, open SOURCE_INDEX.md and the subsystem maps.

Core invariants:
- public Horizon API only; no hnmod/mixins/whole-game Compile;
- no quests/dailies or fake Block Bar;
- V opens UI; Skill Tree stays paged/fixed;
- Bestiary supports live/third-party entity previews where Horizon exposes them;
- Rifts use Resonance, runtime-level Void instances and no mandatory key;
- combat is server-authoritative; B parry, G dodge;
- creature assets are original RustPG assets.
- cloud build tool: .github/workflows/build-on-demand.yml;
- exact visual harness: real Horizon offscreen renderer through tools/horizon_visual_driver.py;
- Claude Code project MCP: .mcp.json -> horizon-launcher mcp.

Validation:
bash scripts/validate.sh
cargo check --manifest-path project/Cargo.toml --lib
cargo test --manifest-path project/Cargo.toml --lib

Key files:
"""+"\n".join(f"{p}\n  {BASE}{p}" for p in key)+"\n"
(ROOT/"AI_CONTEXT.txt").write_text(ctx,encoding="utf-8")

def write_map(filename,title,paths,notes):
    body=[f"# {title}","",f"Generated from commit `{SHA}`.",""]
    for p in paths:
        d=DESCRIPTIONS.get(p.name,"RustPG subsystem module.")
        body.append(f"- {link(p)} — {d}")
    body += ["","## Notes","",notes,""]
    (ROOT/filename).write_text("\n".join(body),encoding="utf-8")

write_map("UI_MAP.md","UI_MAP.md",groups["UI / HUD / Bestiary"],
"""Flow: lib.rs receives input/payloads; rustpg_ui_window.rs builds the screen model; rustpg_ui_layout.rs owns responsive geometry; rustpg_ui_widgets.rs draws reusable pieces; rustpg_ui_input.rs handles touch/mouse/key interaction; target/boss HUD modules handle combat visuals. Keep Skill Tree paged/fixed and do not restore the old infinite canvas or a large dark Damage HUD card.""")
write_map("COMBAT_MAP.md","COMBAT_MAP.md",groups["Combat"],
"""Combat is server-authoritative. AttackEntityEvent remains the melee authority. rustpg_action_combat.rs owns weapon cadence; rustpg_guard.rs owns parry/dodge/perfect windows; rustpg_hitboxes.rs validates reach; rustpg_combat_pipeline.rs resolves damage. Never accept a client-reported hit.""")
write_map("RIFTS_MAP.md","RIFTS_MAP.md",groups["Rifts / World"]+[SRC/"rustpg_profile.rs",SRC/"rustpg_scaling.rs"],
"""Loop: exploration/combat -> Resonance -> 100 -> manifest Rift -> cross portal -> private Void runtime level -> rooms/waves -> boss -> S-D grade -> rewards -> return -> delete instance. A normal key is not required; future keys/sigils should only modify or force special Rifts.""")
write_map("CREATURES_MAP.md","CREATURES_MAP.md",groups["Creatures"]+[SRC/"rustpg_bestiary.rs"],
"""Pipeline: scripts/generate_creatures.py -> PNG textures + entity/loot JSON + rustpg_creatures_generated.rs -> rustpg_native_entities.rs registers rigs/renderers -> rustpg_scaling.rs applies tier/HP -> Bestiary/UI uses names/descriptions. Do not copy model/texture/sound assets from the reference JAR.""")

print(f"Generated AI context for {len(files)} Rust source files at {SHA[:12]}")
