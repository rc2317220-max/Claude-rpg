# AI_START_HERE.md

Use this file when your environment cannot browse GitHub directory trees, cannot `git clone`, or has no GitHub connector.

## Current source of truth

The current RustPG work is on **main**.

Do not assume version 5.0 from an old/stale page. Read these files first:

- https://raw.githubusercontent.com/rc2317220-max/Claude-rpg/main/CLAUDE.md
- https://raw.githubusercontent.com/rc2317220-max/Claude-rpg/main/COMO_USAR_FERRAMENTAS.md
- https://raw.githubusercontent.com/rc2317220-max/Claude-rpg/main/AI_CONTEXT.txt
- https://raw.githubusercontent.com/rc2317220-max/Claude-rpg/main/SOURCE_INDEX.md

## One-file UI/HUD source bundle

For GUI/UI/HUD/Bestiary/Skill Tree work, open this single raw file:

https://raw.githubusercontent.com/rc2317220-max/Claude-rpg/main/AI_UI_SOURCE_BUNDLE.txt

It contains the current contents of:

- project/src/lib.rs
- project/src/rustpg_ui_window.rs
- project/src/rustpg_ui_layout.rs
- project/src/rustpg_ui_system.rs
- project/src/rustpg_ui_widgets.rs
- project/src/rustpg_ui_input.rs
- project/src/rustpg_ui_animation.rs
- project/src/rustpg_hud.rs
- project/src/rustpg_target_hud.rs
- project/src/rustpg_boss_ui.rs
- project/src/rustpg_bestiary.rs
- project/src/rustpg_view.rs
- project/src/rustpg_inspector.rs
- project/src/rustpg_nodes.rs
- project/src/rustpg_skillgraph.rs

Every section is delimited by `BEGIN FILE` / `END FILE` and keeps the original path.

If you cannot write to GitHub, you can still read this bundle and produce exact replacement files or unified diffs. Do not ask the user to manually list UI paths before reading it.

## Tooling / build / visual testing

Read:

https://raw.githubusercontent.com/rc2317220-max/Claude-rpg/main/COMO_USAR_FERRAMENTAS.md

and:

https://raw.githubusercontent.com/rc2317220-max/Claude-rpg/main/VISUAL_TESTING.md

The repository already contains:

- `.github/workflows/build-on-demand.yml`
- `scripts/rustpg_build.sh`
- `.mcp.json`
- `tools/horizon_visual_driver.py`
- `.devcontainer/devcontainer.json`

If those tools are unavailable in your current chat/runtime, say which capability is missing, but still perform source analysis using the raw bundle above.

## Canonical latest HMOD

https://raw.githubusercontent.com/rc2317220-max/Claude-rpg/main/releases/RustPG.hmod
