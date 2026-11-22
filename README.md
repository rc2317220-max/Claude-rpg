# RustPG 5.3.0 — Starling / Horizon 0.9.0

RustPG is now a single **stable public-API** mod. There is no Dev/Core split, no `.hnmod`, no whole-game compile path, and no source module that depends on game-function mixins.

## Current interface

- **V** opens the RustPG screen through the registered key mapping, with a public `input_key` fallback for Android launchers.
- The Skill Tree is a fixed-size paged branch view: no free pan, no zoom, no minimap, no synthetic convergence nodes, and only effects that have an active runtime consumer.
- The combat HUD has no opaque damage panel. Mob preview is compact and attempts the real live entity renderer first.
- The Bestiary uses live third-party entity renderers when an entity of the recorded kind is available, then falls back to the registered kind renderer. Descriptions are factual observations generated from registry id, live display name, kills, level, classifications and known drops.
- The fake 1%-mining bar was removed because the stable Horizon event exposes crack stages, not continuous percent progress.
- Missions and daily missions were removed from state, UI, serialization and runtime hooks.

## Build

`bash scripts/validate.sh` validates JSON/TOML/module integrity and creates `releases/RustPG.hmod`.
GitHub Actions additionally runs `cargo check` and unit tests against the exact pinned Horizon API.
