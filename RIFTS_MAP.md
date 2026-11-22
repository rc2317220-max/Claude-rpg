# RIFTS_MAP.md

## Arquivos de Fendas

- [project/src/rustpg_profile.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_profile.rs) — Perfil persistente do jogador: níveis, stats, Bestiário, Ressonância de Fendas e saves.
- [project/src/rustpg_rifts.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_rifts.rs) — Domínio de Fendas/dungeons: ranks, salas, modificadores, geração, avaliação e sessão.
- [project/src/rustpg_scaling.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_scaling.rs) — Scaling, tiers e HP base de mobs.
- [project/src/rustpg_world_vfx.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_world_vfx.rs) — VFX relacionados ao mundo.

## Loop atual

Exploração/combate → Ressonância → 100 → manifestar Fenda → atravessar portal → runtime level privada Void → salas/ondas → boss → nota S-D → recompensa → retorno → delete da instância.

A chave comum NÃO é requisito. Chaves/sigilos futuros só devem modificar/forçar Fendas especiais.

Anomalias atuais têm efeito real: Umbra, Frenesi, Gravidade Densa, Surto de Elite, Espaço Fraturado, Solo Instável, Névoa Arcana, Relógio da Fenda e Eco de Batalha.
