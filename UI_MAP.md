# UI_MAP.md

Use este mapa para qualquer trabalho de interface/HUD/Bestiário/Skill Tree.

## Arquivos de UI

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

## Fluxo

1. `lib.rs` recebe input/payloads e mantém o estado de UI.
2. `rustpg_ui_window.rs` monta o modelo de tela.
3. `rustpg_ui_layout.rs` decide posições/tamanhos.
4. `rustpg_ui_widgets.rs` desenha elementos reutilizáveis.
5. `rustpg_ui_input.rs` trata interação.
6. `rustpg_target_hud.rs` e `rustpg_boss_ui.rs` cuidam do combate visual.
7. `rustpg_bestiary.rs` + `lib.rs` resolvem dados/preview de entidades.

Não reintroduzir canvas infinito da Skill Tree nem fundo escuro grande no Damage HUD.
