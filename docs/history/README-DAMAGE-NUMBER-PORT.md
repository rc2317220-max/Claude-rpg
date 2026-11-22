# RustPG 4.0.1 — Damage Number reference adaptation

## Source studied

Uploaded file: `1.20.1-damage_number-2.0.5-all(1).jar`

- Forge / JavaFML 47+ for Minecraft 1.20.1; mod ID `damage_number`, author `xypp`, license MIT (per `META-INF/mods.toml`).
- Inspected its class signatures and bytecode with `javap`; no third-party Java classes, bytecode or textures are included in the new hmod.
- The reference is **primarily an in-game combat meter**, not a world-space mob-health addon: it tracks per-player accumulated damage (`userDamage`), combos (`damageCount`), a timeout (`keepUntil`), a recent hit queue, damage types / colors, layout settings and a `damage_number clear` command.
- Its `LivingDamageEvent` processing and optional damage mixins feed a Forge channel with `emit` and `total` messages; the client draws the meter in a GUI overlay.

## Reimplemented in Rust for Starling

1. `src/rustpg_damage_meter.rs`: new independent Rust combat telemetry module. Finite damage validation, damage family classification, combos with inactivity expiration, bounded six-entry recent-hit history and safe tick handling.
2. `src/rustpg_v4_wire.rs`: version 2 of a compact server-to-client combat event. Includes validated hit amount, impacted entity, world position, damage source attacker ID and damage family. Clients can never use it to grant XP.
3. `src/lib.rs`: bridge from `living_damage_post` and `HurtSource.causing` to the wire; local attacker identification on the client; world-space floaters with category colors; separate own-damage meter rendered as a compact panel. Server sends only events within its original 64-block range and has a queue limit.
4. New settings in native Mods > RustPG: `damage_meter`, `damage_meter_list`, `damage_meter_scale`, plus existing `damage_numbers`; PT-BR and EN-US translations.
5. Does not modify existing saved RPG profile formats, quests, item tables, or XP calculations.

## Important limitations / honest status

- No Rust compiler or Starling launcher is installed here: **the package is NOT confirmed to compile or run**. Syntax/asset checks do not replace building/testing in Starling.
- This is NOT a direct loading of a Java Forge `.jar`, and not a complete cloning of every visual/configuration feature in the original jar. The original has additional rank lists, decoration APIs, and GUI customization; only the listed behaviors were adapted.
- Final damage comes from Starling's post-damage event; generic critical hits are *not* reliably identifiable with this hook. Do not color a hit as a "critical" merely because it was large.
- Current mob HP bar continues to show the targeted living entity; per-mob 3D HP bars across the scene are not added in this iteration.
- Like the previous foundation, combat events can still be sent to players in other dimensions when the numeric positions coincide, since the existing server bridge lacks a dimension filter. A separate gate is needed before multiplayer release.
- Hits recorded by Starling but not attributable to an attacker are shown as world numbers but excluded from your personal combo.
- Backup worlds before testing; keep only one mod with ID `rustpg` enabled.

## Manual verification in Starling 0.9.0

1. Install the `.hmod` alone, compile it and capture *full errors with code location* if any occur.
2. Mods > RustPG: enable damage numbers and damage meter; set the scale; disable the recent hit list and confirm the panel shrinks.
3. Attack a cow and a zombie several times: numbers should appear at the mobs; `DANO CAUSADO` should increase only for your hits. Use sword and bow. A projectile owned by the player should be attributed to the owner.
4. Wait more than four seconds: status should switch to TOTAL; the panel fades after the summary period. Attack again to start a fresh combo.
5. Stand next to an environmental damage event (mob on fire/lava): the world number may show, but the personal combo must NOT increment if you did not cause the hit.
6. Test R and `/rpg`, and verify that saves and skill tree remain functional. Check in Android landscape whether the panel overlaps launcher buttons.
7. Multiplayer / Nether and End should be considered **unverified** until dimension isolation is implemented and tested.

## Source reference and credit

Conceptual reference: **Damage Number 2.0.5** by **xypp**, MIT. The Rust module and protocol in this package are new code written for RustPG. No source or artwork was copied from the JAR.
