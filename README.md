# RustPG 5.3.0 — Starling / Horizon 0.9.0

RustPG is now a single **stable public-API** mod. There is no Dev/Core split, no `.hnmod`, no whole-game compile path, and no source module that depends on game-function mixins.

## Current interface

- **V** opens the RustPG screen through the registered key mapping, with a public `input_key` fallback for Android launchers.
- The Skill Tree is a fixed-size paged branch view: no free pan, no zoom, no minimap, no synthetic convergence nodes, and only effects that have an active runtime consumer.
- The combat HUD has no opaque damage panel. Mob preview is compact and requests a GUI preview of the live entity first.
- The Bestiary lists external entity types from the client registry, including before the first kill. It reads the active language’s `entity.<namespace>.<path>` name and optional `.description`, `.desc` or `.lore` entry. Nested path segments use dots. Observed kills and drops remain player-specific.
- Bestiary and target previews use Starling’s GUI entity API with a live instance when available, then the type ID. Starling 0.9.0 draws models supported by its inventory preview pipeline. A renderer supplied only through the world entity-renderer hook needs GUI support in Starling to display its custom geometry here.
- The fake 1%-mining bar was removed because the stable Horizon event exposes crack stages, not continuous percent progress.
- Missions and daily missions were removed from state, UI, serialization and runtime hooks.

## Build

`bash scripts/validate.sh` validates JSON/TOML/module integrity and creates `releases/RustPG.hmod`.
GitHub Actions additionally runs `cargo check` and unit tests against the exact pinned Horizon API.
