# REPO_MAP.md

Mapa de arquitetura do RustPG. Gerado para navegação humana/IA.

## Entrada e estado

- [project/src/lib.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/lib.rs) — Ponto de integração do mod: hooks Horizon, attachments, payloads, comandos, HUD, combate, progressão e ciclo das Fendas.
- [project/src/rustpg_profile.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_profile.rs) — Perfil persistente do jogador: níveis, stats, Bestiário, Ressonância de Fendas e saves.
- [project/src/rustpg_starling090.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_starling090.rs) — Módulo Rust do subsistema.
## UI, HUD e Bestiário

- [project/src/rustpg_bar_vfx.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_bar_vfx.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_bestiary.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_bestiary.rs) — Estado e registros persistentes do Bestiário.
- [project/src/rustpg_boss_ui.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_boss_ui.rs) — Estilo e fases visuais da barra de chefes.
- [project/src/rustpg_damage_meter.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_damage_meter.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_damage_visuals.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_damage_visuals.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_hud.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_hud.rs) — Modelo do HUD principal.
- [project/src/rustpg_hud_geometry.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_hud_geometry.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_inspector.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_inspector.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_presentation.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_presentation.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_target_hud.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_target_hud.rs) — HUD de alvo/dano.
- [project/src/rustpg_timeline.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_timeline.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_ui_animation.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_ui_animation.rs) — Transições e animações da interface.
- [project/src/rustpg_ui_core.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_ui_core.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_ui_input.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_ui_input.rs) — Entrada, clique/toque, navegação e interação da UI.
- [project/src/rustpg_ui_layout.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_ui_layout.rs) — Geometria/layout responsivo da interface e HUD.
- [project/src/rustpg_ui_system.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_ui_system.rs) — Orquestração da UI do Sistema.
- [project/src/rustpg_ui_widgets.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_ui_widgets.rs) — Widgets reutilizáveis da interface.
- [project/src/rustpg_ui_window.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_ui_window.rs) — Modelo principal da janela RustPG: abas, linhas, árvores, Bestiário e dados exibidos.
- [project/src/rustpg_view.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_view.rs) — Texto/páginas exibidas pela interface e comandos.
## Combate

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
## Fendas e mundo

- [project/src/rustpg_profile.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_profile.rs) — Perfil persistente do jogador: níveis, stats, Bestiário, Ressonância de Fendas e saves.
- [project/src/rustpg_rifts.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_rifts.rs) — Domínio de Fendas/dungeons: ranks, salas, modificadores, geração, avaliação e sessão.
- [project/src/rustpg_scaling.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_scaling.rs) — Scaling, tiers e HP base de mobs.
- [project/src/rustpg_world_vfx.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_world_vfx.rs) — VFX relacionados ao mundo.
## Criaturas

- [project/src/rustpg_bestiary.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_bestiary.rs) — Estado e registros persistentes do Bestiário.
- [project/src/rustpg_generated_hunter.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_generated_hunter.rs) — Geometria gerada do Rift Hunter.
- [project/src/rustpg_generated_models.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_generated_models.rs) — Geometria gerada do Ashen Sentinel.
- [project/src/rustpg_mob_wire.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_mob_wire.rs) — Pacotes/replicação de estado de mobs para HUD.
- [project/src/rustpg_native_entities.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_native_entities.rs) — Registro/render nativo das criaturas e rigs.
- [project/src/rustpg_scaling.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_scaling.rs) — Scaling, tiers e HP base de mobs.
- [project/src/rustpg_skeletal.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_skeletal.rs) — Módulo Rust do subsistema.
## Progressão, atributos e equipamento

- [project/src/rustpg_attributes.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_attributes.rs) — Catálogo de atributos.
- [project/src/rustpg_enchant.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_enchant.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_enchant_stack.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_enchant_stack.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_engine_attributes.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_engine_attributes.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_engine_combat.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_engine_combat.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_engine_nutrition.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_engine_nutrition.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_engine_presentation.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_engine_presentation.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_engine_stamina.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_engine_stamina.rs) — Estado e regras de stamina.
- [project/src/rustpg_engine_vitality.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_engine_vitality.rs) — Vitalidade.
- [project/src/rustpg_gear.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_gear.rs) — Relíquias/equipamentos do Sistema.
- [project/src/rustpg_immersive.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_immersive.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_luck.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_luck.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_nodes.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_nodes.rs) — Definição dos nodes e requisitos da Skill Tree.
- [project/src/rustpg_profile.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_profile.rs) — Perfil persistente do jogador: níveis, stats, Bestiário, Ressonância de Fendas e saves.
- [project/src/rustpg_progression.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_progression.rs) — Curvas e regras de progressão.
- [project/src/rustpg_relic_active.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_relic_active.rs) — Módulo Rust do subsistema.
- [project/src/rustpg_relic_engine.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_relic_engine.rs) — Efeitos de relíquias.
- [project/src/rustpg_skillgraph.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_skillgraph.rs) — Estruturas auxiliares da árvore de habilidades.
## Outros módulos

- [project/src/rustpg_actions.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_actions.rs)
- [project/src/rustpg_evil_att.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_evil_att.rs)
- [project/src/rustpg_gathering_rules.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_gathering_rules.rs)
- [project/src/rustpg_java_port.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_java_port.rs)
- [project/src/rustpg_node_effects.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_node_effects.rs)
- [project/src/rustpg_num.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_num.rs)
- [project/src/rustpg_profession_plugins.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_profession_plugins.rs)
- [project/src/rustpg_script_vm.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_script_vm.rs)
- [project/src/rustpg_v4_core.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_v4_core.rs)
- [project/src/rustpg_v4_fatigue.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_v4_fatigue.rs)
- [project/src/rustpg_v4_provenance.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_v4_provenance.rs)
- [project/src/rustpg_v4_wire.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_v4_wire.rs)
- [project/src/rustpg_xp_rules.rs](https://github.com/rc2317220-max/Claude-rpg/blob/da5c7daf5750874cf6853d346f13a488cd450253/project/src/rustpg_xp_rules.rs)
