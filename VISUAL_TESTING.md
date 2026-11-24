# RustPG visual testing

RustPG has two different kinds of validation. Do not confuse them.

## 1. Build correctness

GitHub Actions can build and test the mod without Minecraft running. Use **Actions → RustPG build on demand → Run workflow**. It produces a downloadable `RustPG.hmod` artifact. With `publish_latest=true`, the same validated file becomes `releases/RustPG.hmod`.

Every ordinary push is already checked by `.github/workflows/validate.yml`.

## Agent bootstrap

Before checking visual capability, run:

```bash
python3 tools/ai_doctor.py
bash scripts/ai_bootstrap.sh
source .ai-tools/env.sh
python3 tools/ai_doctor.py
```

The bootstrap supplies the pinned `horizon-ctl`; a real prepared game executable/resources are still intentionally private and must be supplied by the environment.

## Review the UI and creature labs

Each UI pass samples ten menu pages and seven HUD states at four resolutions. The lab draws the same Rust layers as the client, including sprite crops, tints and the geometry of the description panel. Text uses a system font, and the background is a simulated landscape. The creature lab captures the RustPG meshes and textures with Horizon's Vulkan pixel renderer. The UI exporter can place these captures in its bestiary and target portraits.

From a checkout with the pinned Horizon SDK, run the following commands. Keep separate output directories for the baseline and each candidate.

```bash
export HORIZON_SOURCE="$PWD/.ai-tools/horizon"
export RUSTPG_ENTITY_PREVIEWS="$PWD/.agent/artifacts/rustpg-entities"
bash scripts/run_entity_visual_lab.sh "$RUSTPG_ENTITY_PREVIEWS"
export RUSTPG_UI_LAB_OUT="$PWD/visual-artifacts/candidate/svg"
bash scripts/run_ui_visual_export.sh
bash scripts/render_ui_visual_lab.sh "$RUSTPG_UI_LAB_OUT" visual-artifacts/candidate/png
python3 tools/make_visual_contact_sheets.py visual-artifacts/candidate/png \
  "$RUSTPG_ENTITY_PREVIEWS" visual-artifacts/candidate/sheets
```

Open any SVG sheet directly, then open the individual PNG at its native resolution to judge typography, clipping and the thickness of the HUD bars. For a UI-only run, skip the entity lab and unset `RUSTPG_ENTITY_PREVIEWS`. The portraits then show the unavailable-preview state. Exported SVGs contain their textures and can be opened on another machine.

## 2. Pixel-accurate in-game validation

For exact visuals, do **not** build an HTML imitation. Horizon itself already exposes Agent Control.

The real client supports:

- `horizon-launcher mcp` for Claude Code/Codex/other MCP agents;
- game UI/widget inspection;
- keyboard, mouse and touch injection;
- movement, attacks and entity interaction;
- screenshots from the real renderer;
- `--offscreen`, which renders without showing a window.

RustPG therefore uses the actual Horizon renderer as its visual harness.

### Important: offscreen vs headless

Use **offscreen** for visual tests. Horizon's `--headless` mode intentionally has no window, GPU **or renderer**, so screenshots cannot exist there. Offscreen keeps the renderer and screenshots work.

## One-time environment preparation

A machine/Cloud development environment must have:

1. Horizon Launcher / `horizon-ctl`;
2. legally prepared Minecraft resources (from the user's own installation or Mojang through an account that owns Java Edition);
3. a native Horizon client built with this RustPG source.

Do **not** commit Minecraft assets, Microsoft tokens, launcher sessions or account secrets to this repository.

For a normal Horizon installation, the native build command is:

```bash
horizon-launcher native-build project --instance rustpg-visual --trust-native-code
```

The build log/launcher identifies the derived Horizon executable. That executable contains RustPG's native code and must travel with its generated `horizon-native-mods/rustpg/` package directory.

## Claude Code integration

The repository contains a portable `.mcp.json` entrypoint:

```text
.mcp.json -> tools/horizon_mcp_entrypoint.sh -> horizon-launcher OR pinned horizon-ctl
```

Any agent client that supports project-level stdio MCP can use the same entrypoint. Once Horizon Launcher is installed in PATH, restart Claude Code and use `/mcp`. The Horizon server then exposes the launcher's tools plus the running game's `game_*` tools.

An agent can literally:

1. build RustPG;
2. launch the real game offscreen;
3. create/open a test world;
4. press V;
5. inspect every game widget exposed by Horizon;
6. take a screenshot;
7. summon a RustPG creature;
8. look at / attack it;
9. inspect the target HUD and boss bar;
10. edit the source and repeat.

## Automated visual driver

`tools/horizon_visual_driver.py` records a standard visual pass.

Against an already running controlled client:

```bash
python3 tools/horizon_visual_driver.py --attach --output visual-artifacts
```

Or against a native Horizon executable containing RustPG:

```bash
python3 tools/horizon_visual_driver.py \
  --game /path/to/native-horizon-client \
  --game-dir .visual-game \
  --output visual-artifacts
```

It captures, when available:

- `00-world.png`;
- `01-rustpg-menu.png` after the real V key event;
- `01-rustpg-ui-widgets.json`;
- `02-target-hud.png` with a real RustPG creature;
- `03-after-attack.png`;
- entity/state/messages/events/log evidence;
- `visual-report.json`.

These PNGs are from Horizon itself, so they are the closest thing to "the AI playing the game". Differences that come specifically from Android GPU drivers, screen density, touch launcher overlays or device aspect ratio still require a final Android check.

## Codespaces

The included `.devcontainer/devcontainer.json` installs Rust/Python plus Mesa Vulkan software rendering tools. This is useful for compilation and can support offscreen rendering when Horizon/resources are prepared in the Codespace. A GitHub-hosted runner cannot legally manufacture the user's Minecraft resources or account session, so the exact game visual loop is intentionally not part of the mandatory public CI gate.

## Compare two laboratory runs

Download the two `agent-visual` archives from the Actions runs and extract them into separate directories. Keep their `ui-png` and `rustpg-entities` subdirectories. Generate the local comparison with:

```bash
python3 tools/compare_visual_runs.py runs/previous runs/current comparison.html
```

Open `comparison.html` and select a page, HUD state or creature pose. It includes the images from both runs and opens offline. The interface captures replay the mod's drawing layers against a simulated background. The creature images use Horizon’s Vulkan entity renderer. In the complete run, the laboratory embeds those same creature captures in the Bestiary and HUD previews.
