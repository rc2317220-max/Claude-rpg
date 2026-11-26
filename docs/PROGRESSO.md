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
- [x] `.github/workflows/sunrise-preview4-lab.yml` — 200 linhas da base; leitura integral
- [x] `.github/workflows/validate.yml` — 99 linhas da base; leitura integral
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
- [x] `project/src/rustpg_combat_pipeline.rs` — 111 linhas da base; leitura integral
- [ ] `project/src/rustpg_combat_training.rs`
- [ ] `project/src/rustpg_combo.rs`
- [ ] `project/src/rustpg_creatures_generated.rs`
- [ ] `project/src/rustpg_damage_meter.rs`
- [ ] `project/src/rustpg_damage_visuals.rs`
- [x] `project/src/rustpg_elite_encounters.rs` — 77 linhas da base; leitura integral
- [ ] `project/src/rustpg_enchant.rs`
- [ ] `project/src/rustpg_enchant_stack.rs`
- [x] `project/src/rustpg_engine_attributes.rs` — 75 linhas da base; leitura integral
- [ ] `project/src/rustpg_engine_combat.rs`
- [x] `project/src/rustpg_engine_nutrition.rs` — 64 linhas lidas; A26/B06
- [x] `project/src/rustpg_engine_presentation.rs` — 129 linhas da base; leitura integral
- [x] `project/src/rustpg_engine_stamina.rs` — 127 linhas da base; leitura integral
- [x] `project/src/rustpg_engine_vitality.rs` — 68 linhas da base; leitura integral
- [x] `project/src/rustpg_evil_att.rs` — 126 linhas da base; leitura integral
- [ ] `project/src/rustpg_gathering_rules.rs`
- [x] `project/src/rustpg_gear.rs` — 180 linhas da base; leitura integral
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
- [x] `project/src/rustpg_powers.rs` — 94 linhas da base; leitura integral
- [ ] `project/src/rustpg_presentation.rs`
- [ ] `project/src/rustpg_profession_plugins.rs`
- [x] `project/src/rustpg_profile.rs` — 827 linhas da base; leitura integral
- [x] `project/src/rustpg_progression.rs` — 109 linhas da base; leitura integral
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
- [x] `tools/preview4_port.py` — 98 linhas da base; leitura integral
- [ ] `tools/preview4_revamp.py`
- [x] `tools/sunrise_compat.py` — 119 linhas da base; leitura integral
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

## Resumo do personagem
- [x] Implementação inicial: GuiEntity::Existing do jogador local (SDK Preview 4 gui_entity.rs:204–210), oito slots, modelo rotativo, equipamento nativo e relíquias em tooltip, vitais/atributos, próxima patente real, aviso de Fenda condicional. Resumo sem painel lateral para preservar espaço.
- [ ] Validar geometria, pilha de item/tooltip, skin, armadura, mouse e touch em execução real; expandir atributos e melhorar tradução dos nomes nativos.

## Combate — ciclo de efeitos reais
- [x] Implementação: recuperação integra janela de ação, maestria não finita neutralizada, tetos de armas compartilhados com validação de compra, penetração/bonificações antes das mitigações finais, origem direta obrigatória para bônus corpo a corpo e sangramento/roubo de vida corpo a corpo.
- [ ] Validar cadência no SDK, testes de dano, custo e fontes de dano; ticket de acerto único e matriz de dano crítico ainda exigem testes em jogo.

## CI
- [x] Disparar laboratório Preview 4 também quando código e dados base mudam; não depender apenas das mudanças em ferramentas.
- [ ] Verificar resultado do novo laboratório e do estável no commit combinado.


## Segundo ciclo — implementação e validação parcial

- [f4633ef](https://github.com/rc2317220-max/Claude-rpg/commit/f4633ef09db957946be9e4888a2b0e780264e511): Perfil: falhas de leitura preservam dados, codec rejeita cabeçalhos corrompidos, sem recompensa de morte criativo/espectador.
- [aebb77f](https://github.com/rc2317220-max/Claude-rpg/commit/aebb77f028772cbd1a249e4e532baa05475122e9): Fendas: fase Rewarded, retorno separado, falhas de consulta e zero spawns não avançam ondas, limpeza.
- [a7518a5](https://github.com/rc2317220-max/Claude-rpg/commit/a7518a5f569ec712bfbd16b52e2c19a407906707): Resumo: jogador local 3D, oito slots e tooltips nativos/relíquias, patente e aviso de Fenda reais.
- [adab870](https://github.com/rc2317220-max/Claude-rpg/commit/adab870d57d239ad2e6124772705d965ca898cac): Combate: recuperação, maestria, caps de armas, origem direta, ordem de mitigação.
- [b44193a](https://github.com/rc2317220-max/Claude-rpg/commit/b44193aaa6c4dd0fd04898d430d278a5be328e40): CI: mudanças de src/data/assets também disparam Preview 4.

- [Estável anterior](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45745742036): aprovado.
- [Preview anterior](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45745643234): compilação/testes Preview e pacote limpo aprovados; execução cancelada antes de concluir Android.
- [Estável atual](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45748355579): source-validation e compile stable public-API build aprovados; Run Rust unit tests em andamento na consulta final.
- [Preview atual](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45748355456): inspeção de criaturas aprovada; compilação/testes e controle Android em andamento. Nenhum teste de jogo novo aprovado.
- Cobertura integral: 18/149 arquivos (inventário original); não confundir leitura com correção. lib.rs e demais fontes parcialmente examinadas permanecem abertos.
- Resumo: o modelo SDK usa jogador real, mas renderização visual, todos os atributos, layout de patente/XP e traduções ainda exigem teste. Fendas: recibo durável entre reinícios, ondas parcialmente geradas e timeout pendentes. Sem exclusão de sistemas ou saves.

### Achados adicionais (base e estado antes das respectivas correções)
| ID | Gravidade | Evidência | Próximo tratamento |
|---|---|---|---|
| A27 | Alta | rustpg_powers.rs:17,40–42: cooldown associado a dimensão/entidade e removido no unload | UUID estável; teste reconexão |
| A28 | Alta | lib.rs: food_level_changed ignora aumento; nutrition.tick apenas decai | Confirmar hook de ingestão e consumo dos efeitos |
| A29 | Alta | lib.rs:2519–2535: grupo por distância sem dimensão; usa nível; dia e progresso padrão | Dimensão, poder e progresso reais |
| A30 | Alta | lib.rs:2916–2961: pecuária no ramo scales | Separar bônus profissional e relíquia |
| A31 | Alta | rustpg_gear.rs:74–80,105–118: UID saturante pode colidir | checked_add e migração preservadora |
| A32 | Média | rustpg_elite_encounters.rs:47–50: vampírico normal sem regeneração | Confirmar regen base e corrigir restrição |
| A33 | Alta | validate.yml: upload do HMOD existente, publish checkout da branch | Empacotar SHA validado; conferir scripts de build |
| A34 | Alta | lib.rs:1050–1055: cache e poderes podem mudar antes de PROFILE.set | Persistir antes de publicar; rollback de ação |
| A35 | Alta | lib.rs:2604–2618,2710–2733: drops consultam espelho em memória; fallback de attachment em falha | Hidratar proveniência persistente e falhar de modo seguro |

## Próximo item
- [ ] Verificar o resultado de 45748355456 e 45748355579 e corrigir falhas; depois concluir persistência das Fendas, recarga de poderes por UUID, nutrição, proveniência, loot/profissões, resumo e HUD. Prosseguir leitura dos arquivos restantes.

## Terceiro ciclo — recarga de poderes
- [x] A27: recarga por UUID e poder, sobrevive a troca de entidade/dimensão durante a sessão do servidor; teste cobre troca, reutilização de ID por outro jogador e expiração.
- [ ] Verificar compilação e testes desse commit, persistência entre reinícios não incluída nesta correção.
- Actions anteriores: estável 45748355579 (174 testes aprovados); Preview 45748355456 (180 testes aprovados, pacote compilado, runtime falhou: Setup screen not recognized: start).

## Terceiro ciclo — mutações
- [x] A32: mutação Vampírico em tier Normal habilita regeneração conforme o multiplicador já existente; descrição não promete roubo de vida inexistente. Teste diferencia Normal, Vampírico e Regenerativo.
- [ ] Compilar e validar no jogo.

## Terceiro ciclo — nutrição
- [x] A28: aumento real no evento de fome alimenta o perfil; restauração de energia paga uma vez; bônus temporários entram em cura e mitigação; criativo/espectador excluídos; teste de duração e pagamento único.
- [ ] Validar SDK, cancelamento de evento por outros mods e consumo real no Android.

## Terceiro ciclo — caça e pecuária
- [x] A30: o cálculo de cópias dos drops também alcança animais passivos, preservando a restrição de relíquias aos mobs escaláveis. Os bônus de pecuária só são aplicados aos produtos reconhecidos de animais de criação.
- [ ] Compilar e conferir drops reais e ausência de relíquias em animais passivos.

## Terceiro ciclo — escalonamento
- [x] A29 (parcial): poder de grupo agora considera somente jogadores na dimensão do mob, não criativos/espectadores.
- [ ] A29 restante: progresso global durável e dia do mundo; validar escalonamento e compilação.

## Terceiro ciclo — árvore radial, segunda inspeção
- [x] Achado adicional: o teste anterior aceitava uma lista visível vazia; o mínimo de escala podia deixar a árvore totalmente fora da tela. Na câmera padrão, quando o anel não cabe, a primeira raiz é centralizada. A escala continua legível e o pan permite alcançar ramos distantes. Teste exige nó visível para viewport >=30x30 e usa centro efetivo para validar arestas.
- [ ] Executar os testes Preview e verificar visualmente centralização, pan e zoom (sem afirmar que Ajustar exibe todos os nós ao mesmo tempo).

## Terceiro ciclo — laboratório Android
- [x] Baixado e aberto o artifact 15086420613 do run 45748355456. Captura setup-start-latest.png mostra ANR de com.google.android.googlesdksetup; não é evidência de crash do RustPG. Runner desativa somente o assistente Google SDK no AVD descartável e reconhece seu diálogo ANR pelo texto real, preservando capturas de falhas do jogo.
- [ ] Confirmar em nova execução que a preparação e o mundo são alcançados; validar o mod real.

## Terceiro ciclo — encantamento
- [x] Livro parcialmente aceito: avanço acima do teto só para encantamentos que realmente aparecem na saída da bigorna; um encantamento rejeitado não é introduzido por maestria ou transcendência. Teste com afiação e julgamento.
- [ ] Validar integração de bigorna no SDK e jogo.
