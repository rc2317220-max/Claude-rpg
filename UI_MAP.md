# UI_MAP.md

Generated from commit `e2e5219d10fb1b02706a90e30d9bc2a22a20718f`.

- [project/src/rustpg_bar_vfx.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_bar_vfx.rs) — RustPG subsystem module.
- [project/src/rustpg_bestiary.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_bestiary.rs) — Bestiary persistent state.
- [project/src/rustpg_damage_meter.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_damage_meter.rs) — RustPG subsystem module.
- [project/src/rustpg_damage_visuals.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_damage_visuals.rs) — RustPG subsystem module.
- [project/src/rustpg_engine_presentation.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_engine_presentation.rs) — RustPG subsystem module.
- [project/src/rustpg_hud.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_hud.rs) — RustPG subsystem module.
- [project/src/rustpg_hud_geometry.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_hud_geometry.rs) — RustPG subsystem module.
- [project/src/rustpg_inspector.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_inspector.rs) — RustPG subsystem module.
- [project/src/rustpg_presentation.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_presentation.rs) — RustPG subsystem module.
- [project/src/rustpg_target_hud.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_target_hud.rs) — Target/damage HUD.
- [project/src/rustpg_timeline.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_timeline.rs) — RustPG subsystem module.
- [project/src/rustpg_ui_animation.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_ui_animation.rs) — RustPG subsystem module.
- [project/src/rustpg_ui_core.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_ui_core.rs) — RustPG subsystem module.
- [project/src/rustpg_ui_input.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_ui_input.rs) — Touch/mouse/key UI input.
- [project/src/rustpg_ui_layout.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_ui_layout.rs) — Responsive UI/HUD geometry.
- [project/src/rustpg_ui_system.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_ui_system.rs) — UI system orchestration.
- [project/src/rustpg_ui_theme.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_ui_theme.rs) — RustPG subsystem module.
- [project/src/rustpg_ui_widgets.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_ui_widgets.rs) — Reusable widgets.
- [project/src/rustpg_ui_window.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_ui_window.rs) — Main RustPG window model.
- [project/src/rustpg_view.rs](https://github.com/rc2317220-max/Claude-rpg/blob/e2e5219d10fb1b02706a90e30d9bc2a22a20718f/project/src/rustpg_view.rs) — Text/pages exposed by UI and commands.

## Notes

Flow: lib.rs receives input/payloads; rustpg_ui_window.rs builds the screen model; rustpg_ui_layout.rs owns responsive geometry; rustpg_ui_widgets.rs draws reusable pieces; rustpg_ui_input.rs handles touch/mouse/key interaction; target/boss HUD modules handle combat visuals. Keep Skill Tree paged/fixed and do not restore the old infinite canvas or a large dark Damage HUD card.
