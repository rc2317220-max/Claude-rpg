# CREATURES_MAP.md

## Arquivos de criaturas

- [project/src/rustpg_bestiary.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_bestiary.rs) — Estado e registros persistentes do Bestiário.
- [project/src/rustpg_generated_hunter.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_generated_hunter.rs) — Geometria gerada do Rift Hunter.
- [project/src/rustpg_generated_models.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_generated_models.rs) — Geometria gerada do Ashen Sentinel.
- [project/src/rustpg_mob_wire.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_mob_wire.rs) — Pacotes/replicação de estado de mobs para HUD.
- [project/src/rustpg_native_entities.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_native_entities.rs) — Registro/render nativo das criaturas e rigs.
- [project/src/rustpg_scaling.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_scaling.rs) — Scaling, tiers e HP base de mobs.
- [project/src/rustpg_skeletal.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_skeletal.rs) — Módulo Rust do subsistema.

## Pipeline

`scripts/generate_creatures.py` → texturas PNG + JSON de entity_type/loot + `rustpg_creatures_generated.rs` → `rustpg_native_entities.rs` registra rigs/renderers → `rustpg_scaling.rs` aplica tier/HP → Bestiário/UI lê nomes e descrições.

Criaturas próprias atuais: Ashen Sentinel, Rift Hunter, Cão de Mana, Cavaleiro Sepulcral, Louva-a-Deus da Fenda, Serpe Cristalina, Ogro do Eclipse, Espectro do Crepúsculo, Grifo Tempestuoso, Colosso Rúnico, Diabrete das Brasas e Tecelã Abissal.

Não copiar modelos/texturas/assets do JAR de referência.
