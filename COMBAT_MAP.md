# COMBAT_MAP.md

Generated from commit `f8f8b8cc163a678dcfadff19720baca22281b39b`.

- [project/src/rustpg_action_combat.rs](https://github.com/rc2317220-max/Claude-rpg/blob/f8f8b8cc163a678dcfadff19720baca22281b39b/project/src/rustpg_action_combat.rs) — Server-authoritative weapon action timing.
- [project/src/rustpg_combat.rs](https://github.com/rc2317220-max/Claude-rpg/blob/f8f8b8cc163a678dcfadff19720baca22281b39b/project/src/rustpg_combat.rs) — RustPG subsystem module.
- [project/src/rustpg_combat_pipeline.rs](https://github.com/rc2317220-max/Claude-rpg/blob/f8f8b8cc163a678dcfadff19720baca22281b39b/project/src/rustpg_combat_pipeline.rs) — Damage resolution pipeline.
- [project/src/rustpg_combat_training.rs](https://github.com/rc2317220-max/Claude-rpg/blob/f8f8b8cc163a678dcfadff19720baca22281b39b/project/src/rustpg_combat_training.rs) — RustPG subsystem module.
- [project/src/rustpg_combo.rs](https://github.com/rc2317220-max/Claude-rpg/blob/f8f8b8cc163a678dcfadff19720baca22281b39b/project/src/rustpg_combo.rs) — RustPG subsystem module.
- [project/src/rustpg_elite_encounters.rs](https://github.com/rc2317220-max/Claude-rpg/blob/f8f8b8cc163a678dcfadff19720baca22281b39b/project/src/rustpg_elite_encounters.rs) — RustPG subsystem module.
- [project/src/rustpg_engine_combat.rs](https://github.com/rc2317220-max/Claude-rpg/blob/f8f8b8cc163a678dcfadff19720baca22281b39b/project/src/rustpg_engine_combat.rs) — RustPG subsystem module.
- [project/src/rustpg_guard.rs](https://github.com/rc2317220-max/Claude-rpg/blob/f8f8b8cc163a678dcfadff19720baca22281b39b/project/src/rustpg_guard.rs) — Parry/dodge/perfect defense windows.
- [project/src/rustpg_hitboxes.rs](https://github.com/rc2317220-max/Claude-rpg/blob/f8f8b8cc163a678dcfadff19720baca22281b39b/project/src/rustpg_hitboxes.rs) — RustPG subsystem module.
- [project/src/rustpg_powers.rs](https://github.com/rc2317220-max/Claude-rpg/blob/f8f8b8cc163a678dcfadff19720baca22281b39b/project/src/rustpg_powers.rs) — RustPG subsystem module.
- [project/src/rustpg_weapon_training.rs](https://github.com/rc2317220-max/Claude-rpg/blob/f8f8b8cc163a678dcfadff19720baca22281b39b/project/src/rustpg_weapon_training.rs) — RustPG subsystem module.

## Notes

Combat is server-authoritative. AttackEntityEvent remains the melee authority. rustpg_action_combat.rs owns weapon cadence; rustpg_guard.rs owns parry/dodge/perfect windows; rustpg_hitboxes.rs validates reach; rustpg_combat_pipeline.rs resolves damage. Never accept a client-reported hit.
