# PROGRESSO — auditoria integral RustPG

Base imutável: `515457228c1712cac0502dcf3046c43466eef1ee`. Trabalho isolado em `rustpg-auditoria-integral` para preservar main. Cada checkbox de leitura significa revisão integral efetiva, não download. Correção e validação são estados distintos. Não apagar sistemas inteiros nem dados de jogadores. A versão efetiva passa pelos quatro scripts de port.

## Inventário de leitura (151 arquivos)

- [x] `.github/workflows/agent-terminal.yml` — 46 linhas, revisão integral
- [x] `.github/workflows/ai-tooling.yml` — 57 linhas, revisão integral
- [x] `.github/workflows/asset-studio.yml` — 83 linhas, revisão integral
- [x] `.github/workflows/blockbench-desktop.yml` — 59 linhas, revisão integral
- [x] `.github/workflows/blockbench-model-jobs.yml` — 157 linhas, revisão integral
- [x] `.github/workflows/build-on-demand.yml` — 112 linhas, revisão integral
- [x] `.github/workflows/divine-powers.yml` — 52 linhas, revisão integral
- [x] `.github/workflows/extract-project.yml` — 68 linhas, revisão integral
- [x] `.github/workflows/goblin-boss-reference.yml` — 39 linhas, revisão integral
- [x] `.github/workflows/goblin-family.yml` — 64 linhas, revisão integral
- [x] `.github/workflows/goblin-vision-agent.yml` — 70 linhas, revisão integral
- [x] `.github/workflows/import-starling-preview4-reference.yml` — 49 linhas, revisão integral
- [x] `.github/workflows/rift-mantis-forge.yml` — 42 linhas, revisão integral
- [x] `.github/workflows/rustpg-migrate.yml` — 85 linhas, revisão integral
- [x] `.github/workflows/sentinel-v4-recovery.yml` — 57 linhas, revisão integral
- [x] `.github/workflows/storm-gryphon-forge.yml` — 42 linhas, revisão integral
- [x] `.github/workflows/sunrise-java-bridge.yml` — 40 linhas, revisão integral
- [x] `.github/workflows/sunrise-preview4-lab.yml` — 200 linhas, revisão integral
- [x] `.github/workflows/validate.yml` — 99 linhas, revisão integral
- [x] `AI_AGENT_PROTOCOL.md` — 146 linhas, revisão integral
- [x] `UI_MAP.md` — 28 linhas, revisão integral
- [x] `project/data/rustpg/horizon/entity_type/abyss_weaver.json` — 10 linhas, revisão integral
- [x] `project/data/rustpg/horizon/entity_type/ashen_sentinel.json` — 10 linhas, revisão integral
- [x] `project/data/rustpg/horizon/entity_type/crystal_wyrm.json` — 10 linhas, revisão integral
- [x] `project/data/rustpg/horizon/entity_type/dusk_revenant.json` — 10 linhas, revisão integral
- [x] `project/data/rustpg/horizon/entity_type/eclipse_ogre.json` — 10 linhas, revisão integral
- [x] `project/data/rustpg/horizon/entity_type/ember_imp.json` — 10 linhas, revisão integral
- [x] `project/data/rustpg/horizon/entity_type/grave_knight.json` — 10 linhas, revisão integral
- [x] `project/data/rustpg/horizon/entity_type/mana_hound.json` — 10 linhas, revisão integral
- [x] `project/data/rustpg/horizon/entity_type/rift_hunter.json` — 10 linhas, revisão integral
- [x] `project/data/rustpg/horizon/entity_type/rift_mantis.json` — 10 linhas, revisão integral
- [x] `project/data/rustpg/horizon/entity_type/rune_colossus.json` — 10 linhas, revisão integral
- [x] `project/data/rustpg/horizon/entity_type/storm_gryphon.json` — 10 linhas, revisão integral
- [x] `project/data/rustpg/horizon/item/abyss_heart.json` — 16 linhas, revisão integral
- [x] `project/data/rustpg/horizon/item/hunter_token.json` — 16 linhas, revisão integral
- [x] `project/data/rustpg/horizon/item/rift_blade.json` — 24 linhas, revisão integral
- [x] `project/data/rustpg/horizon/item/sentinel_charm.json` — 16 linhas, revisão integral
- [x] `project/data/rustpg/loot_table/entities/abyss_weaver.json` — 14 linhas, revisão integral
- [x] `project/data/rustpg/loot_table/entities/ashen_sentinel.json` — 14 linhas, revisão integral
- [x] `project/data/rustpg/loot_table/entities/crystal_wyrm.json` — 14 linhas, revisão integral
- [x] `project/data/rustpg/loot_table/entities/dusk_revenant.json` — 14 linhas, revisão integral
- [x] `project/data/rustpg/loot_table/entities/eclipse_ogre.json` — 14 linhas, revisão integral
- [x] `project/data/rustpg/loot_table/entities/ember_imp.json` — 14 linhas, revisão integral
- [x] `project/data/rustpg/loot_table/entities/grave_knight.json` — 14 linhas, revisão integral
- [x] `project/data/rustpg/loot_table/entities/mana_hound.json` — 14 linhas, revisão integral
- [x] `project/data/rustpg/loot_table/entities/rift_hunter.json` — 14 linhas, revisão integral
- [x] `project/data/rustpg/loot_table/entities/rift_mantis.json` — 14 linhas, revisão integral
- [x] `project/data/rustpg/loot_table/entities/rune_colossus.json` — 14 linhas, revisão integral
- [x] `project/data/rustpg/loot_table/entities/storm_gryphon.json` — 14 linhas, revisão integral
- [x] `project/data/rustpg/plugins/activities_advanced.rustpg` — 8 linhas, revisão integral
- [x] `project/data/rustpg/plugins/extra.rustpg` — 5 linhas, revisão integral
- [x] `project/data/rustpg/plugins/professions.rustpg` — 15 linhas, revisão integral
- [x] `project/data/rustpg/plugins_vm/bonus.rustpgx` — 10 linhas, revisão integral
- [x] `project/data/rustpg/recipe/abyss_heart.json` — 13 linhas, revisão integral
- [x] `project/data/rustpg/recipe/hunter_token.json` — 13 linhas, revisão integral
- [x] `project/data/rustpg/recipe/rift_blade.json` — 13 linhas, revisão integral
- [x] `project/data/rustpg/recipe/sentinel_charm.json` — 13 linhas, revisão integral
- [x] `project/src/lib.rs` — 4133 linhas, revisão integral
- [x] `project/src/rustpg_action_combat.rs` — 50 linhas, revisão integral
- [x] `project/src/rustpg_actions.rs` — 67 linhas, revisão integral
- [x] `project/src/rustpg_attributes.rs` — 741 linhas, revisão integral
- [x] `project/src/rustpg_bar_vfx.rs` — 54 linhas, revisão integral
- [x] `project/src/rustpg_bestiary.rs` — 51 linhas, revisão integral
- [x] `project/src/rustpg_boss_ui.rs` — 166 linhas, revisão integral
- [x] `project/src/rustpg_combat.rs` — 242 linhas, revisão integral
- [x] `project/src/rustpg_combat_pipeline.rs` — 111 linhas, revisão integral
- [x] `project/src/rustpg_combat_training.rs` — 37 linhas, revisão integral
- [x] `project/src/rustpg_combo.rs` — 42 linhas, revisão integral
- [x] `project/src/rustpg_creatures_generated.rs` — 1017 linhas, validação estrutural automatizada integral
- [x] `project/src/rustpg_damage_meter.rs` — 156 linhas, revisão integral
- [x] `project/src/rustpg_damage_visuals.rs` — 53 linhas, revisão integral
- [x] `project/src/rustpg_elite_encounters.rs` — 77 linhas, revisão integral
- [x] `project/src/rustpg_enchant.rs` — 82 linhas, revisão integral
- [x] `project/src/rustpg_enchant_stack.rs` — 217 linhas, revisão integral
- [x] `project/src/rustpg_engine_attributes.rs` — 75 linhas, revisão integral
- [x] `project/src/rustpg_engine_combat.rs` — 24 linhas, revisão integral
- [x] `project/src/rustpg_engine_nutrition.rs` — 64 linhas, revisão integral
- [x] `project/src/rustpg_engine_presentation.rs` — 129 linhas, revisão integral
- [x] `project/src/rustpg_engine_stamina.rs` — 127 linhas, revisão integral
- [x] `project/src/rustpg_engine_vitality.rs` — 68 linhas, revisão integral
- [x] `project/src/rustpg_evil_att.rs` — 126 linhas, revisão integral
- [x] `project/src/rustpg_gathering_rules.rs` — 84 linhas, revisão integral
- [x] `project/src/rustpg_gear.rs` — 180 linhas, revisão integral
- [x] `project/src/rustpg_generated_hunter.rs` — 113 linhas, revisão integral
- [x] `project/src/rustpg_generated_models.rs` — 100 linhas, revisão integral
- [x] `project/src/rustpg_guard.rs` — 129 linhas, revisão integral
- [x] `project/src/rustpg_hitboxes.rs` — 44 linhas, revisão integral
- [x] `project/src/rustpg_hud.rs` — 24 linhas, revisão integral
- [x] `project/src/rustpg_hud_geometry.rs` — 47 linhas, revisão integral
- [x] `project/src/rustpg_immersive.rs` — 76 linhas, revisão integral
- [x] `project/src/rustpg_inspector.rs` — 392 linhas, revisão integral
- [x] `project/src/rustpg_java_port.rs` — 212 linhas, revisão integral
- [x] `project/src/rustpg_luck.rs` — 459 linhas, revisão integral
- [x] `project/src/rustpg_mob_wire.rs` — 72 linhas, revisão integral
- [x] `project/src/rustpg_native_entities.rs` — 255 linhas, revisão integral
- [x] `project/src/rustpg_native_modifiers.rs` — 43 linhas, revisão integral
- [x] `project/src/rustpg_node_effects.rs` — 378 linhas, revisão integral
- [x] `project/src/rustpg_nodes.rs` — 347 linhas, revisão integral
- [x] `project/src/rustpg_num.rs` — 88 linhas, revisão integral
- [x] `project/src/rustpg_powers.rs` — 94 linhas, revisão integral
- [x] `project/src/rustpg_presentation.rs` — 12 linhas, revisão integral
- [x] `project/src/rustpg_profession_plugins.rs` — 207 linhas, revisão integral
- [x] `project/src/rustpg_profile.rs` — 827 linhas, revisão integral
- [x] `project/src/rustpg_progression.rs` — 109 linhas, revisão integral
- [x] `project/src/rustpg_relic_active.rs` — 173 linhas, revisão integral
- [x] `project/src/rustpg_relic_engine.rs` — 201 linhas, revisão integral
- [x] `project/src/rustpg_rifts.rs` — 233 linhas, revisão integral
- [x] `project/src/rustpg_scaling.rs` — 439 linhas, revisão integral
- [x] `project/src/rustpg_script_vm.rs` — 165 linhas, revisão integral
- [x] `project/src/rustpg_skeletal.rs` — 93 linhas, revisão integral
- [x] `project/src/rustpg_skillgraph.rs` — 136 linhas, revisão integral
- [x] `project/src/rustpg_starling090.rs` — 137 linhas, revisão integral
- [x] `project/src/rustpg_target_hud.rs` — 223 linhas, revisão integral
- [x] `project/src/rustpg_timeline.rs` — 47 linhas, revisão integral
- [x] `project/src/rustpg_ui_animation.rs` — 79 linhas, revisão integral
- [x] `project/src/rustpg_ui_core.rs` — 92 linhas, revisão integral
- [x] `project/src/rustpg_ui_input.rs` — 41 linhas, revisão integral
- [x] `project/src/rustpg_ui_layout.rs` — 85 linhas, revisão integral
- [x] `project/src/rustpg_ui_system.rs` — 1437 linhas, revisão integral
- [x] `project/src/rustpg_ui_theme.rs` — 117 linhas, revisão integral
- [x] `project/src/rustpg_ui_widgets.rs` — 49 linhas, revisão integral
- [x] `project/src/rustpg_ui_window.rs` — 2663 linhas, revisão integral
- [x] `project/src/rustpg_v4_core.rs` — 188 linhas, revisão integral
- [x] `project/src/rustpg_v4_fatigue.rs` — 30 linhas, revisão integral
- [x] `project/src/rustpg_v4_provenance.rs` — 62 linhas, revisão integral
- [x] `project/src/rustpg_v4_wire.rs` — 87 linhas, revisão integral
- [x] `project/src/rustpg_view.rs` — 361 linhas, revisão integral
- [x] `project/src/rustpg_weapon_training.rs` — 59 linhas, revisão integral
- [x] `project/src/rustpg_world_vfx.rs` — 25 linhas, revisão integral
- [x] `project/src/rustpg_xp_rules.rs` — 45 linhas, revisão integral
- [x] `scripts/ai_bootstrap.sh` — 71 linhas, revisão integral
- [x] `scripts/asset_studio_setup.sh` — 31 linhas, revisão integral
- [x] `scripts/audit_assets.py` — 39 linhas, revisão integral
- [x] `scripts/audit_skill_trees.py` — 50 linhas, revisão integral
- [x] `scripts/blockbench_mcp.sh` — 6 linhas, revisão integral
- [x] `scripts/build_creature_model.py` — 160 linhas, revisão integral
- [x] `scripts/generate_ai_context.py` — 164 linhas, revisão integral
- [x] `scripts/generate_creature_portraits.py` — 123 linhas, revisão integral
- [x] `scripts/generate_creatures.py` — 423 linhas, revisão integral
- [x] `scripts/generate_ui_assets.py` — 201 linhas, revisão integral
- [x] `scripts/package_hmod.sh` — 6 linhas, revisão integral
- [x] `scripts/package_native.py` — 77 linhas, revisão integral
- [x] `scripts/paint_ashen_sentinel.py` — 60 linhas, revisão integral
- [x] `scripts/paint_rift_hunter.py` — 60 linhas, revisão integral
- [x] `scripts/rustpg_build.sh` — 27 linhas, revisão integral
- [x] `scripts/rustpg_migrate.py` — 216 linhas, revisão integral
- [x] `scripts/validate.sh` — 40 linhas, revisão integral
- [x] `tools/preview4_port.py` — 98 linhas, revisão integral
- [x] `tools/preview4_revamp.py` — 251 linhas, revisão integral
- [x] `tools/sunrise_compat.py` — 119 linhas, revisão integral
- [x] `tools/sunrise_rpg_rework.py` — 275 linhas, revisão integral

## Entregas e correções

- [ ] F1: auditoria individual, 60 achados verificáveis, 3 por sistema; inventário de textos
- [ ] F2: ordem topológica dos setores radiais
- [ ] F2: visibilidade das arestas e corte na borda dos nós
- [ ] F2: escala do centro e legibilidade dos nós
- [ ] F2: teste geométrico de todas as árvores
- [ ] F2: Voltar único na árvore e bestiário
- [ ] F2: retrato 3D local, oito slots e atributos reais
- [ ] F2: próxima patente, progresso e ressonância
- [ ] F2: nomes dos nós, valores, traduções, ocultar IDs
- [ ] F3: limites de atributos de arma e compras
- [ ] F3: penetração antes da mitigação
- [ ] F3: recuperação e janela de ataque
- [ ] F3: progressão mundial de mobs e grupo por dimensão
- [ ] F3: nutrição no consumo
- [ ] F3: críticos, resistência, projéteis e alcance no combate real
- [ ] F3: recargas por jogador
- [ ] F3: sangramento e roubo de vida exclusivamente melee
- [ ] F3: mutação vampírica
- [ ] F3: Fendas recompensa idempotente, morte/saída, ondas, logout
- [ ] F3: recompensas apenas sobrevivência/aventura
- [ ] F3: proveniência persistente, drops, overflow
- [ ] F3: criação/caça de passivos
- [ ] F3: compatibilidade de encantamentos múltiplos
- [ ] F3: preservação de perfil em erro de leitura
- [ ] F3: migração de nós e redistribuição acessível
- [ ] F3: geometria de cliques bestiário
- [ ] F3: status e unidades
- [ ] F3: HUD responsivo, tiers/mutações
- [ ] F3: tecla remapeável e instruções
- [ ] F3: patente Monarca
- [ ] F3: relíquias e inspetor sem IDs internos
- [ ] F3: todos os achados adicionais da auditoria
- [ ] F4: avisos, dead_code, divisão lib.rs, versão única
- [ ] F4: limpeza READMEs e releases (preservar dados e sistemas)
- [ ] Validação: Actions, pacote dividido e Android, evidência por sistema

## Registro

- Inventário obtido da árvore Git completa, sem truncamento. Nenhum arquivo marcado como revisado ainda.
- Protocolos revisados e corrigidos neste commit. Ainda sem mudança de código. Próximo item: leitura dos quatro scripts e auditoria do código efetivo.

- Auditoria parcial: 33 arquivos revisados; demais explicitamente pendentes em AUDITORIA.md. Ainda não há correções de código nem teste desta branch.

- Segundo lote: 11 módulos de combate, fendas, encantamentos, equipamentos e relíquias revisados; 44/151. Corrigir somente após completar a auditoria, conforme Fase 1.

- Terceiro lote: 18 módulos revisados integralmente, incluindo wire de dano, escalas, mutações, defesa, renderização e progressão. Total 62/151.

- Quarto lote: 42 arquivos, incluindo perfil, nós, visualizações, escalonamento, sorte e todos os dados. Total 104/151.

- Quinto lote: 17 scripts completos. Total 121/151. A correção de UI_MAP precisa atingir generate_ai_context.py, que hoje recria a orientação antiga em toda validação.

- Sexto lote: 19 workflows completos. Total 140/151. Restam 11 módulos Rust (incluindo lib.rs e tabelas geradas). O laboratório atual não valida o par split em Android; publicar com rótulo pós-runtime não equivale a testar o par.

- Sétimo lote: 7 módulos completos, incluindo catálogo de atributos, inspetor, plugins, VM e renderização de criaturas. Total 147/151. Os quatro arquivos restantes exigem leitura integral em blocos.

- Oitavo lote: UI system, UI window e tabelas geradas das criaturas. Total 150/151. Resta lib.rs, 4.133 linhas, que reúne os consumidores e adaptadores reais.

- Nono lote: lib.rs integral (1–4133). Todos os 151 arquivos inventariados revisados; tabelas geradas verificadas automaticamente. Fase 1 concluída quanto à leitura, achados e documentação; nenhum código de jogo alterado até aqui. Próximo: Fase 2, geometria radial do pacote Preview 4.

- Fase 2: geometria radial e navegação dedicadas implementadas em tools/preview4_radial.py. Setores seguem ordem do catálogo, arestas só entre nós visíveis, extremidades na borda dos losangos, escala mínima legível e centro compartilhado. Testes novos percorrem catálogo e hitboxes. Correção de gerador de documentação e gatilho CI da branch incluídos. Checkboxes de correção continuam abertos até resultado real do Actions.

- Revisão do teste radial: assinatura in_tree recebe &str, links montados sem captura de empréstimo; árvores com dois setores ocupam arco de 120° para não cruzar o centro. CI anterior iniciou; nova execução substitui a anterior.

- Fase 2: Resumo e Bestiário: integração do retrato Existing(player.entity()) confirmado no SDK fixado (existing_gui_entity trata Target::Player), leitura de slots 36..39 e mão selecionada, tooltip real do motor e relíquias/atributos. Geometria compartilhada de linhas do Bestiário, patente real, Monarca e aviso de Fenda condicional. Testes novos; aguarda CI. A raridade textual nativa ainda não é garantida pelo tooltip e deve ser validada no jogo.

- Fase 2: nomes próprios por estágio de atributo, descrição de compra única, números localizados, ocultação dos identificadores na descrição de nós e ações do inspetor, raridade/slot/efeitos legíveis nas relíquias. IDs de persistência e valores não foram alterados. Validação pendente; textos das demais rotas ainda não concluídos.

- Proteção de persistência: removidos 13 fallbacks de PROFILE.get que convertiam falha em perfil novo; login não regrava perfil automaticamente; validação de cabeçalho, versão e campos obrigatórios no codec, teste de corrupção e migração antiga real. Não foi feita migração destrutiva. Ledger, recuperação de dados parciais e sanidade completa de itens ainda precisam revisão. CI pendente.
