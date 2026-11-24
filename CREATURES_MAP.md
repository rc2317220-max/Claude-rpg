# CREATURES_MAP.md

Generated from commit `caa352a4f68b46fb18d4c7c63b66f3a7f224ad4c`.

- [project/src/rustpg_generated_hunter.rs](https://github.com/rc2317220-max/Claude-rpg/blob/caa352a4f68b46fb18d4c7c63b66f3a7f224ad4c/project/src/rustpg_generated_hunter.rs) — RustPG subsystem module.
- [project/src/rustpg_generated_models.rs](https://github.com/rc2317220-max/Claude-rpg/blob/caa352a4f68b46fb18d4c7c63b66f3a7f224ad4c/project/src/rustpg_generated_models.rs) — RustPG subsystem module.
- [project/src/rustpg_mob_wire.rs](https://github.com/rc2317220-max/Claude-rpg/blob/caa352a4f68b46fb18d4c7c63b66f3a7f224ad4c/project/src/rustpg_mob_wire.rs) — RustPG subsystem module.
- [project/src/rustpg_native_entities.rs](https://github.com/rc2317220-max/Claude-rpg/blob/caa352a4f68b46fb18d4c7c63b66f3a7f224ad4c/project/src/rustpg_native_entities.rs) — Custom creature render/AI registration.
- [project/src/rustpg_scaling.rs](https://github.com/rc2317220-max/Claude-rpg/blob/caa352a4f68b46fb18d4c7c63b66f3a7f224ad4c/project/src/rustpg_scaling.rs) — Mob tiers, HP and scaling.
- [project/src/rustpg_skeletal.rs](https://github.com/rc2317220-max/Claude-rpg/blob/caa352a4f68b46fb18d4c7c63b66f3a7f224ad4c/project/src/rustpg_skeletal.rs) — RustPG subsystem module.
- [project/src/rustpg_bestiary.rs](https://github.com/rc2317220-max/Claude-rpg/blob/caa352a4f68b46fb18d4c7c63b66f3a7f224ad4c/project/src/rustpg_bestiary.rs) — Bestiary persistent state.

## Notes

Pipeline: scripts/generate_creatures.py -> PNG textures + entity/loot JSON + rustpg_creatures_generated.rs -> rustpg_native_entities.rs registers rigs/renderers -> rustpg_scaling.rs applies tier/HP -> Bestiary/UI uses names/descriptions. Do not copy model/texture/sound assets from the reference JAR.
