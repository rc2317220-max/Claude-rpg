# RustPG 6.2.0-alpha

RustPG runs on the **Starling/Horizon 0.9.0 public API**. [Download RustPG.hmod](releases/RustPG.hmod).

## Rift access — no mandatory key grind

The normal dungeon loop uses **Rift Resonance**, not disposable keys.

- Normal/elite/boss kills and first-time dimension exploration build Resonance.
- At 100 Resonance, `/rustpg rift manifest` stabilizes a personal Rift arch on safe loaded terrain near the player.
- Resonance is consumed only when the owner physically walks through the arch.
- The portal expires after five minutes if ignored; no Resonance is lost.
- Special keys/sigils are deliberately not required by the core loop. They can later become optional modifiers for rare/ranked Rifts rather than a toll to play the system.

Each entry opens a private Horizon **runtime level** named `rustpg:rift/<id>` using `RuntimeGenerator::Void`. Different players can therefore run separate instances without building arenas far away in the normal world. The server creates the physical rooms, corridors and encounters, returns the player to the exact origin, then deletes the instance.

## Dungeon state machine

A deterministic seed creates Rank E–S dungeons with 4–9 rooms. Rooms are real server structures and move through dormant → combat → clear states. Current room types include entrance, hunt, ambush, shrine, trap, elite and boss. Shrine/ambush/trap/boss rooms have distinct physical layouts rather than being labels on identical boxes.

Rift anomalies all have a real consumer:

- **Umbra** — recurring Darkness.
- **Frenzy** — spawned RustPG mobs receive Speed + Strength.
- **Gravitational Pressure** — recurring Slowness + Mining Fatigue.
- **Elite Surge** — extra elite spawn.
- **Fractured Space** — Nausea + reverse-portal instability VFX.
- **Unstable Ground** — magma hazards are built into combat rooms.
- **Arcane Mist** — recurring Blindness.
- **Rift Clock** — 22% tighter completion target for the grade.
- **Battle Echo** — duplicates a wave member.

The final boss closes the run. The server grades S/A/B/C/D from elapsed time, damage received and deaths, then awards general XP and Resonance. Best rank/grade and Rift clears persist in the player profile.

## Creature roster

RustPG now has **12 original custom creatures**. No model, texture or sound asset from the Solo Leveling reference jar is shipped.

Previously available:

- Ashen Sentinel
- Rift Hunter
- Cão de Mana
- Cavaleiro Sepulcral
- Louva-a-Deus da Fenda
- Serpe Cristalina
- Ogro do Eclipse

The creature pipeline remains generated with authored RustPG geometry/material motifs:

- **Espectro do Crepúsculo** (`rustpg:dusk_revenant`) — spectral armored elite; soul-rake animation source.
- **Grifo Tempestuoso** (`rustpg:storm_gryphon`) — articulated wings/talons, aerial mini-boss; thunder-dive clip.
- **Colosso Rúnico** (`rustpg:rune_colossus`) — rune-heart stone boss; heavy earthshatter clip.
- **Diabrete das Brasas** (`rustpg:ember_imp`) — small fast airborne rift creature with wings/tail.
- **Tecelã Abissal** (`rustpg:abyss_weaver`) — articulated eight-leg spider elite with venom-bite clip.

They have original 128–256 px procedural texture atlases, independent HP/tier/attributes, loot tables, Bestiary descriptions and native entity renderer rigs. Boss/miniboss species participate in the dedicated target/boss HUD palettes.

## Action combat 6.0

The uploaded Solo Leveling Fabric mod was studied as a technical reference for its separation of server combat state, action timing, geometry and defensive windows. RustPG reimplements those ideas for Horizon rather than copying its assets or depending on Fabric mixins.

- A real `AttackEntityEvent` remains the authority for every melee attempt.
- Each weapon family has a server-side windup/active/recovery/chain timeline.
- Inputs during recovery are rejected instead of creating phantom hits.
- Sword, axe, mace, spear/trident and unarmed chains have separate cadence and bounded combo length.
- Weapon mastery can shorten anticipation/recovery by at most 28%; it never deletes the active hit slice.
- Combo/action bonus is deliberately small and capped; existing RPG attributes still own the main damage scaling.
- **B** = parry. The first three ticks are a perfect-parry window; a successful perfect parry arms one counter bonus.
- **G** = dodge. The first four ticks are a perfect-dodge window with full prevention for an eligible hit.
- Damage prevention, stamina cost and counter eligibility are all decided server-side.

The stable API does not provide the same animation-replacement freedom as Fabric mixins, so this release improves authoritative combat mechanics/timing without pretending it can replace every vanilla first/third-person attack animation.

## Existing interface rules retained

- **V** opens RustPG through normal key mapping plus the public `input_key` fallback.
- Skill trees remain paged/fixed instead of the old infinite-pan graph.
- Bestiary still scans third-party entity registry entries and requests live GUI entity previews when Horizon supports them.
- No fake 1%-mining bar, missions/dailies, hnmod or abandoned mixin-only systems were reintroduced.

## Canonical release

Every successful push runs structural validation, the standalone Rift/action tests, `cargo check` and the full Rust test suite against the pinned Horizon API. Only after both validation jobs pass does GitHub Actions replace **`releases/RustPG.hmod`** with that validated package. The raw GitHub path therefore remains the permanent "latest build" link.

## Validation

`bash scripts/validate.sh` regenerates original creature data, validates JSON/TOML/module integrity and creates `releases/RustPG.hmod`. GitHub Actions additionally runs standalone HUD/Rift/action-combat tests and `cargo check` + the complete Rust test suite against Horizon commit `ff303502a4f7982551a0e0d3622d9268efe9e829`.

Runtime/visual behavior inside the Android Starling client still needs an in-game pass after installation; CI proves API compilation and tests, not touch/visual feel on the device.
