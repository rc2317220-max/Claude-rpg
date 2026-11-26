# Progresso da auditoria RustPG

Base inventariada: `515457228c1712cac0502dcf3046c43466eef1ee`. Caixas só são marcadas após leitura integral ou correção com evidência; compilação e teste em jogo são estados separados.

## Inventário de leitura

- [ ] `.github/workflows/agent-terminal.yml`
- [ ] `.github/workflows/ai-tooling.yml`
- [ ] `.github/workflows/asset-studio.yml`
- [ ] `.github/workflows/blockbench-desktop.yml`
- [ ] `.github/workflows/blockbench-model-jobs.yml`
- [ ] `.github/workflows/build-on-demand.yml`
- [ ] `.github/workflows/divine-powers.yml`
- [ ] `.github/workflows/extract-project.yml`
- [ ] `.github/workflows/goblin-boss-reference.yml`
- [ ] `.github/workflows/goblin-family.yml`
- [ ] `.github/workflows/goblin-vision-agent.yml`
- [ ] `.github/workflows/import-starling-preview4-reference.yml`
- [ ] `.github/workflows/rift-mantis-forge.yml`
- [ ] `.github/workflows/rustpg-migrate.yml`
- [ ] `.github/workflows/sentinel-v4-recovery.yml`
- [ ] `.github/workflows/storm-gryphon-forge.yml`
- [ ] `.github/workflows/sunrise-java-bridge.yml`
- [ ] `.github/workflows/sunrise-preview4-lab.yml`
- [ ] `.github/workflows/validate.yml`
- [ ] `project/data/rustpg/horizon/entity_type/abyss_weaver.json`
- [ ] `project/data/rustpg/horizon/entity_type/ashen_sentinel.json`
- [ ] `project/data/rustpg/horizon/entity_type/crystal_wyrm.json`
- [ ] `project/data/rustpg/horizon/entity_type/dusk_revenant.json`
- [ ] `project/data/rustpg/horizon/entity_type/eclipse_ogre.json`
- [ ] `project/data/rustpg/horizon/entity_type/ember_imp.json`
- [ ] `project/data/rustpg/horizon/entity_type/grave_knight.json`
- [ ] `project/data/rustpg/horizon/entity_type/mana_hound.json`
- [ ] `project/data/rustpg/horizon/entity_type/rift_hunter.json`
- [ ] `project/data/rustpg/horizon/entity_type/rift_mantis.json`
- [ ] `project/data/rustpg/horizon/entity_type/rune_colossus.json`
- [ ] `project/data/rustpg/horizon/entity_type/storm_gryphon.json`
- [ ] `project/data/rustpg/horizon/item/abyss_heart.json`
- [ ] `project/data/rustpg/horizon/item/hunter_token.json`
- [ ] `project/data/rustpg/horizon/item/rift_blade.json`
- [ ] `project/data/rustpg/horizon/item/sentinel_charm.json`
- [ ] `project/data/rustpg/loot_table/entities/abyss_weaver.json`
- [ ] `project/data/rustpg/loot_table/entities/ashen_sentinel.json`
- [ ] `project/data/rustpg/loot_table/entities/crystal_wyrm.json`
- [ ] `project/data/rustpg/loot_table/entities/dusk_revenant.json`
- [ ] `project/data/rustpg/loot_table/entities/eclipse_ogre.json`
- [ ] `project/data/rustpg/loot_table/entities/ember_imp.json`
- [ ] `project/data/rustpg/loot_table/entities/grave_knight.json`
- [ ] `project/data/rustpg/loot_table/entities/mana_hound.json`
- [ ] `project/data/rustpg/loot_table/entities/rift_hunter.json`
- [ ] `project/data/rustpg/loot_table/entities/rift_mantis.json`
- [ ] `project/data/rustpg/loot_table/entities/rune_colossus.json`
- [ ] `project/data/rustpg/loot_table/entities/storm_gryphon.json`
- [ ] `project/data/rustpg/plugins/activities_advanced.rustpg`
- [ ] `project/data/rustpg/plugins/extra.rustpg`
- [ ] `project/data/rustpg/plugins/professions.rustpg`
- [ ] `project/data/rustpg/plugins_vm/bonus.rustpgx`
- [ ] `project/data/rustpg/recipe/abyss_heart.json`
- [ ] `project/data/rustpg/recipe/hunter_token.json`
- [ ] `project/data/rustpg/recipe/rift_blade.json`
- [ ] `project/data/rustpg/recipe/sentinel_charm.json`
- [ ] `project/src/lib.rs`
- [x] `project/src/rustpg_action_combat.rs` — 50 linhas lidas; achados A08/A17
- [ ] `project/src/rustpg_actions.rs`
- [ ] `project/src/rustpg_attributes.rs`
- [ ] `project/src/rustpg_bar_vfx.rs`
- [ ] `project/src/rustpg_bestiary.rs`
- [ ] `project/src/rustpg_boss_ui.rs`
- [ ] `project/src/rustpg_combat.rs`
- [ ] `project/src/rustpg_combat_pipeline.rs`
- [ ] `project/src/rustpg_combat_training.rs`
- [ ] `project/src/rustpg_combo.rs`
- [ ] `project/src/rustpg_creatures_generated.rs`
- [ ] `project/src/rustpg_damage_meter.rs`
- [ ] `project/src/rustpg_damage_visuals.rs`
- [ ] `project/src/rustpg_elite_encounters.rs`
- [ ] `project/src/rustpg_enchant.rs`
- [ ] `project/src/rustpg_enchant_stack.rs`
- [ ] `project/src/rustpg_engine_attributes.rs`
- [ ] `project/src/rustpg_engine_combat.rs`
- [x] `project/src/rustpg_engine_nutrition.rs` — 64 linhas lidas; A26/B06
- [ ] `project/src/rustpg_engine_presentation.rs`
- [ ] `project/src/rustpg_engine_stamina.rs`
- [ ] `project/src/rustpg_engine_vitality.rs`
- [ ] `project/src/rustpg_evil_att.rs`
- [ ] `project/src/rustpg_gathering_rules.rs`
- [ ] `project/src/rustpg_gear.rs`
- [ ] `project/src/rustpg_generated_hunter.rs`
- [ ] `project/src/rustpg_generated_models.rs`
- [ ] `project/src/rustpg_guard.rs`
- [ ] `project/src/rustpg_hitboxes.rs`
- [ ] `project/src/rustpg_hud.rs`
- [ ] `project/src/rustpg_hud_geometry.rs`
- [ ] `project/src/rustpg_immersive.rs`
- [ ] `project/src/rustpg_inspector.rs`
- [ ] `project/src/rustpg_java_port.rs`
- [ ] `project/src/rustpg_luck.rs`
- [ ] `project/src/rustpg_mob_wire.rs`
- [ ] `project/src/rustpg_native_entities.rs`
- [ ] `project/src/rustpg_native_modifiers.rs`
- [ ] `project/src/rustpg_node_effects.rs`
- [ ] `project/src/rustpg_nodes.rs`
- [ ] `project/src/rustpg_num.rs`
- [ ] `project/src/rustpg_powers.rs`
- [ ] `project/src/rustpg_presentation.rs`
- [ ] `project/src/rustpg_profession_plugins.rs`
- [ ] `project/src/rustpg_profile.rs`
- [ ] `project/src/rustpg_progression.rs`
- [ ] `project/src/rustpg_relic_active.rs`
- [ ] `project/src/rustpg_relic_engine.rs`
- [ ] `project/src/rustpg_rifts.rs`
- [ ] `project/src/rustpg_scaling.rs`
- [ ] `project/src/rustpg_script_vm.rs`
- [ ] `project/src/rustpg_skeletal.rs`
- [ ] `project/src/rustpg_skillgraph.rs`
- [ ] `project/src/rustpg_starling090.rs`
- [ ] `project/src/rustpg_target_hud.rs`
- [ ] `project/src/rustpg_timeline.rs`
- [ ] `project/src/rustpg_ui_animation.rs`
- [ ] `project/src/rustpg_ui_core.rs`
- [ ] `project/src/rustpg_ui_input.rs`
- [ ] `project/src/rustpg_ui_layout.rs`
- [ ] `project/src/rustpg_ui_system.rs`
- [ ] `project/src/rustpg_ui_theme.rs`
- [ ] `project/src/rustpg_ui_widgets.rs`
- [ ] `project/src/rustpg_ui_window.rs`
- [ ] `project/src/rustpg_v4_core.rs`
- [ ] `project/src/rustpg_v4_fatigue.rs`
- [x] `project/src/rustpg_v4_provenance.rs` — 62 linhas lidas; A15/A16
- [ ] `project/src/rustpg_v4_wire.rs`
- [ ] `project/src/rustpg_view.rs`
- [ ] `project/src/rustpg_weapon_training.rs`
- [ ] `project/src/rustpg_world_vfx.rs`
- [ ] `project/src/rustpg_xp_rules.rs`
- [ ] `scripts/ai_bootstrap.sh`
- [ ] `scripts/asset_studio_setup.sh`
- [ ] `scripts/audit_assets.py`
- [ ] `scripts/audit_skill_trees.py`
- [ ] `scripts/blockbench_mcp.sh`
- [ ] `scripts/build_creature_model.py`
- [ ] `scripts/generate_ai_context.py`
- [ ] `scripts/generate_creature_portraits.py`
- [ ] `scripts/generate_creatures.py`
- [ ] `scripts/generate_ui_assets.py`
- [ ] `scripts/package_hmod.sh`
- [ ] `scripts/package_native.py`
- [ ] `scripts/paint_ashen_sentinel.py`
- [ ] `scripts/paint_rift_hunter.py`
- [ ] `scripts/rustpg_build.sh`
- [ ] `scripts/rustpg_migrate.py`
- [ ] `scripts/validate.sh`
- [ ] `tools/preview4_port.py`
- [ ] `tools/preview4_revamp.py`
- [ ] `tools/sunrise_compat.py`
- [ ] `tools/sunrise_rpg_rework.py`

## Fases e correções

- [x] Corrigir documentação AI_AGENT_PROTOCOL.md e UI_MAP.md e gerador (árvore radial); alteração documental, sem teste runtime.
- [ ] Rastrear código final dos quatro scripts e validar âncoras
- [ ] docs/AUDITORIA.md: cobertura por arquivo e linhas
- [ ] Árvore radial: ordem, recorte, escala, legibilidade, setas e testes
- [ ] Navegação Voltar, mouse, toque e remapeamento
- [ ] Resumo: jogador 3D, oito slots, atributos, patente e Fenda
- [ ] Textos, nomes, unidades e localização pt-BR
- [ ] Perfil: não sobrescrever dados corrompidos; migração e respec
- [ ] Combate: limites, mitigação, recuperação, contexto, crítico e projéteis
- [ ] Nutrição, stamina, poderes e dimensões
- [ ] Fendas: recompensa idempotente, morte, logout, falha e ondas
- [ ] Loot, proveniência persistente, profissões e encantamentos
- [ ] HUD, bestiário, relíquias e inspeção
- [ ] Criaturas, IA, animação, VFX, desempenho e assets
- [ ] Unificar versões, modularizar lib.rs, verificar código morto e duplicatas
- [ ] Testes unitários, integração, compilação e artefato Preview 4
- [ ] Teste real Android: menu, árvore, HUD, resumo, combate e persistência
- [ ] Registrar commits, links Actions e resultados reais

## Estado
Nenhum item marcado como corrigido nesta abertura. Inventário não equivale a leitura integral.

- Auditoria inicial: f11228c35e18745986a5fe16100a18ff6f9e64d2; 65 registros, cobertura ainda parcial. Correções de código e validações pendentes.

## Ciclo de correção radial
- [x] Implementado no estágio: ordem de catálogo, arestas entre nós visíveis, terminação no losango, centro com escala comum, mínimo 11 px, teste de geometria em três resoluções e quatro zooms.
- [ ] Validar o teste Rust e o pacote final no Actions; confirmar visual e usabilidade no Android. Não marcar A01–A05 como encerrados até passar.

## Ciclo de navegação
- [x] Implementação: Voltar único no cabeçalho de árvore/Bestiário, catálogo com rótulo MAPA, hit-test do Bestiário alinhado ao layout e teste de geometria.
- [ ] Compilar e confirmar navegação com mouse e toque no Android; não encerrar A06/B03 antes da prova.

## Encerramento parcial da sessão
- Documentação radial: 21ffa5ee9111fc6f2ab70b04f2f4abcbd1814d7f.
- Implementação radial: 2c81b4ba43698737af98bdf45ccbe9527774ee24.
- Implementação de navegação/Bestiário: 2d2c0296c02e24bef247842306fc1d26dd6c66c3.
- Inventário individual e auditoria: 1e6d19d9714f0e42bd5d555a7b28bd31736e0ae8.
- Página: https://chatgpt.com/space/page_cc20582bc5008191ba64f6fdcb4b6217
- Actions do último commit de código: https://github.com/rc2317220-max/Claude-rpg/actions/runs/45745643234 e https://github.com/rc2317220-max/Claude-rpg/actions/runs/45745643272 — em execução na última consulta, sem conclusão. Execução anterior 45675432926: runtime falhou sem mundo aberto.
- Risco adicional a verificar: o filtro de push do sunrise-preview4-lab.yml não lista project/src/** nem project/data/**; mudanças na base podem deixar de disparar o laboratório.
- [ ] Próximo item: verificar log e resultado do Actions 45745643234; corrigir eventuais falhas antes de fechar A01–A06/B03. Continuar cobertura integral por arquivo, perfil e Fendas críticos, resumo 3D e restante da auditoria.

## Proteção de perfil e recompensas
- [x] Implementação: interromper todas as rotas que convertiam erro de PROFILE.get em perfil novo; recusar cabeçalho inválido/duplicado/futuro; bloquear recompensas de mortes em criativo/espectador (inclusive modo desconhecido); testes de regressão do codec.
- [ ] Validar integração e saves legados; recuperação de campos secundários e migração de compras ainda pendentes.

## Fendas — ciclo crítico
- [x] Implementação: fase Rewarded separa concessão da tentativa de retorno; falha de consulta não equivale a onda vazia; onda com zero spawns não avança; saída de dimensão encaminha para limpeza; logout não descarta sessão antes da confirmação de remoção.
- [ ] Validar falha de teleporte, morte, logout, consultas e spawn no jogo. Persistência do recibo de recompensa entre reinícios e rastreamento individual de IDs das ondas ainda não implementados.
