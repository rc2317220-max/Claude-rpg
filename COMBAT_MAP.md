# COMBAT_MAP.md

O combate 6.0 é server-authoritative.

## Arquivos de combate

- [project/src/rustpg_action_combat.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_action_combat.rs) — Timing autoritativo do novo combate: windup, active, recovery e chains.
- [project/src/rustpg_combat.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_combat.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_combat_pipeline.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_combat_pipeline.rs) — Pipeline de resolução de dano.
- [project/src/rustpg_combat_training.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_combat_training.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_combo.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_combo.rs) — Rastreamento de combos confirmados.
- [project/src/rustpg_elite_encounters.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_elite_encounters.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_guard.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_guard.rs) — Parry/dodge autoritativos, perfect parry, counter e perfect dodge.
- [project/src/rustpg_hitboxes.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_hitboxes.rs) — Validação geométrica/alcance de golpes.
- [project/src/rustpg_native_modifiers.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_native_modifiers.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_powers.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_powers.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_weapon_training.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_weapon_training.rs) — Detecção de família de arma e progressão por arma.

## Regras

- `AttackEntityEvent` é a autoridade do ataque corpo a corpo.
- `rustpg_action_combat.rs`: windup/active/recovery/chain por arma.
- `rustpg_guard.rs`: parry/dodge e perfect windows.
- `rustpg_hitboxes.rs`: alcance/validação geométrica.
- `rustpg_combat_pipeline.rs`: resolução final do dano.
- Não aceitar dano/hit declarado pelo cliente.
- Ao portar ideias do JAR de Solo Leveling, traduzir arquitetura/algoritmos para a API pública; não copiar assets.
