# CREATURES_MAP.md

Generated from commit `38f9b94fd16c20fea2f19a300d4d2ef21178e109`.

- [project/src/rustpg_generated_hunter.rs](https://github.com/rc2317220-max/Claude-rpg/blob/38f9b94fd16c20fea2f19a300d4d2ef21178e109/project/src/rustpg_generated_hunter.rs) — RustPG subsystem module.
- [project/src/rustpg_generated_models.rs](https://github.com/rc2317220-max/Claude-rpg/blob/38f9b94fd16c20fea2f19a300d4d2ef21178e109/project/src/rustpg_generated_models.rs) — RustPG subsystem module.
- [project/src/rustpg_mob_wire.rs](https://github.com/rc2317220-max/Claude-rpg/blob/38f9b94fd16c20fea2f19a300d4d2ef21178e109/project/src/rustpg_mob_wire.rs) — RustPG subsystem module.
- [project/src/rustpg_native_entities.rs](https://github.com/rc2317220-max/Claude-rpg/blob/38f9b94fd16c20fea2f19a300d4d2ef21178e109/project/src/rustpg_native_entities.rs) — Custom creature render/AI registration.
- [project/src/rustpg_scaling.rs](https://github.com/rc2317220-max/Claude-rpg/blob/38f9b94fd16c20fea2f19a300d4d2ef21178e109/project/src/rustpg_scaling.rs) — Mob tiers, HP and scaling.
- [project/src/rustpg_skeletal.rs](https://github.com/rc2317220-max/Claude-rpg/blob/38f9b94fd16c20fea2f19a300d4d2ef21178e109/project/src/rustpg_skeletal.rs) — RustPG subsystem module.
- [project/src/rustpg_bestiary.rs](https://github.com/rc2317220-max/Claude-rpg/blob/38f9b94fd16c20fea2f19a300d4d2ef21178e109/project/src/rustpg_bestiary.rs) — Bestiary persistent state.

## Notes

Pipeline: scripts/generate_creatures.py -> PNG textures + entity/loot JSON + rustpg_creatures_generated.rs -> rustpg_native_entities.rs registers rigs/renderers -> rustpg_scaling.rs applies tier/HP -> Bestiary/UI uses names/descriptions. Do not copy model/texture/sound assets from the reference JAR.
