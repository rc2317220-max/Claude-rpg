# AUDITORIA — RustPG / Sunrise Preview 4

Auditoria **parcial**, base `515457228c1712cac0502dcf3046c43466eef1ee`. 151 arquivos inventariados; 33 revisados integralmente neste ciclo. Linhas contadas sem a quebra final; não confundir achado estático com prova em Android. Alguns achados são riscos que exigem cruzamento de consumidores, não defeitos confirmados em execução. Não completar artificialmente a meta de 60. Nenhum código do jogo editado nesta fase.

| Arquivo | Linhas lidas | Problemas / evidência | Gravidade | Correção proposta |
|---|---:|---|---|---|
| `.github/workflows/agent-terminal.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/ai-tooling.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/asset-studio.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/blockbench-desktop.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/blockbench-model-jobs.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/build-on-demand.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/divine-powers.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/extract-project.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/goblin-boss-reference.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/goblin-family.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/goblin-vision-agent.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/import-starling-preview4-reference.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/rift-mantis-forge.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/rustpg-migrate.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/sentinel-v4-recovery.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/storm-gryphon-forge.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/sunrise-java-bridge.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/sunrise-preview4-lab.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `.github/workflows/validate.yml` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `AI_AGENT_PROTOCOL.md` | 146 | D01: confunde regras de Horizon com Preview 4; árvore fixa obsoleta. | Média | Corrigido na branch: separar plataformas e fontes efetivas. |
| `UI_MAP.md` | 28 | D02: orientação fixa contradiz radial; mapa gerado de commit antigo. | Média | Corrigido: explicitar pipeline e geometria única. |
| `project/data/rustpg/horizon/entity_type/abyss_weaver.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/horizon/entity_type/ashen_sentinel.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/horizon/entity_type/crystal_wyrm.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/horizon/entity_type/dusk_revenant.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/horizon/entity_type/eclipse_ogre.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/horizon/entity_type/ember_imp.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/horizon/entity_type/grave_knight.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/horizon/entity_type/mana_hound.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/horizon/entity_type/rift_hunter.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/horizon/entity_type/rift_mantis.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/horizon/entity_type/rune_colossus.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/horizon/entity_type/storm_gryphon.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/horizon/item/abyss_heart.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/horizon/item/hunter_token.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/horizon/item/rift_blade.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/horizon/item/sentinel_charm.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/loot_table/entities/abyss_weaver.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/loot_table/entities/ashen_sentinel.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/loot_table/entities/crystal_wyrm.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/loot_table/entities/dusk_revenant.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/loot_table/entities/eclipse_ogre.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/loot_table/entities/ember_imp.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/loot_table/entities/grave_knight.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/loot_table/entities/mana_hound.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/loot_table/entities/rift_hunter.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/loot_table/entities/rift_mantis.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/loot_table/entities/rune_colossus.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/loot_table/entities/storm_gryphon.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/plugins/activities_advanced.rustpg` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/plugins/extra.rustpg` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/plugins/professions.rustpg` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/plugins_vm/bonus.rustpgx` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/recipe/abyss_heart.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/recipe/hunter_token.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/recipe/rift_blade.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/data/rustpg/recipe/sentinel_charm.json` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/lib.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_action_combat.rs` | 50 | C03: recovery não bloqueia novo ataque; C04: bônus expira em 2 ticks independente de timing; C05: timing saneia NaN mas cálculo de bônus não usa mastery saneada; C06: tick+delta sem saturação; step ignorado em base. | Alta | Integrar recuperação e janela com hooks reais; sanear entrada; saturar tempo. |
| `project/src/rustpg_actions.rs` | 67 | C07: protocolo dito canônico aceita números com zeros à esquerda e +; não é quebra de autoridade, mas permite várias codificações da mesma ação. | Baixa | Exigir forma decimal canônica e testar. |
| `project/src/rustpg_attributes.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_bar_vfx.rs` | 54 | U12: condição target>ghost+0.012 durante cura pode falhar após dano; U13: branch display-target é sempre zero logo após atribuição; comentários anunciam easing da camada principal que não existe. | Média | Detectar mudança contra valor anterior; remover código redundante, manter HP autoritativo. |
| `project/src/rustpg_bestiary.rs` | 51 | U14: zoom/drag aceitam NaN, envenenando estado de rotação; U15: first_seen legado não significa desbloqueio e precisa ficar fora da UI. | Média | Validar entradas finitas e testar; revisar consumidores. |
| `project/src/rustpg_boss_ui.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_combat.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_combat_pipeline.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_combat_training.rs` | 37 | C08: fallback classifica qualquer dano não listado como espada; flecha de besta também vira arco se esse fallback for usado. | Média | Registrar arma real do ataque; não conceder XP de estilo desconhecido. |
| `project/src/rustpg_combo.rs` | 42 | C09: combo é por entidade/dimensão, não por arma, apesar do comentário; C10: descarta qualquer segundo golpe no mesmo tick, mesmo hits legítimos; teste usa teto 1.25 enquanto implementação limita 1.20. | Média | Identidade de golpe/arma e teste de teto exato. |
| `project/src/rustpg_creatures_generated.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_damage_meter.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_damage_visuals.rs` | 53 | U28: tipo elemental agrega fogo/veneno/wither, sem cor própria nesta estrutura; U29: origem/amount não sanitizados nesta camada (verificar wire). | Média | Tipo preciso e validar antes de renderizar. |
| `project/src/rustpg_elite_encounters.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_enchant.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_enchant_stack.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_engine_attributes.rs` | 75 | S06: custo e regeneração expõem unidades sem definição temporal; S07: saída pode atingir 1e300 enquanto stamina limita a 1e9, criando divergência; S08: limites de redução aplicados por piso, sem regra de compra. | Média | Unidades e caps coerentes com engine e catálogo. |
| `project/src/rustpg_engine_combat.rs` | 24 | C11: substring axe também corresponde a pickaxe, classifica picareta como arma pesada. | Média | Classificação por item/tag ou sufixo inequívoco. |
| `project/src/rustpg_engine_nutrition.rs` | 64 | N01: stamina_restore acumula e decai sem método de consumo atômico; N02: bônus antigos são acumulados em nova refeição mesmo após expiração enquanto decaem. Ausência de chamada eat no runtime ainda requer revisão de lib. | Alta | Aplicar refeição autoritativa uma vez; limpar bônus expirados; consumir restauração uma vez. |
| `project/src/rustpg_engine_presentation.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_engine_stamina.rs` | 127 | S01: clamp deixa current NaN; S02: consume(0) retorna true antes de verificar fadiga; S03: recovery_progress divide por mínimo 1 em vez de limiar real quando max<2. | Média | Sanear valores; definir semântica de custo zero; corrigir denominador e teste. |
| `project/src/rustpg_engine_vitality.rs` | 68 | S04: from_vanilla_amount aplica escala fixa de 20, potencial duplicação quando motor já usa max HP real; confirmar chamadas; S05: current/max sem sanitização completa ao carregar perfil. | Alta | Uma fonte de HP; validar perfil e cálculos. |
| `project/src/rustpg_evil_att.rs` | 126 | M01: descrições prometem comportamentos específicos; correspondência de vampirismo, crítico, repulsão e regeneração precisa ser confirmada nos consumidores. O módulo em si só define catálogo, sorteio e berserk. | Pendente | Cruzar com scaling e lib antes de editar. |
| `project/src/rustpg_gathering_rules.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_gear.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_generated_hunter.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_generated_models.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_guard.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_hitboxes.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_hud.rs` | 24 | U16: hp_fraction aceita NaN; comentário cita rustpg_main inexistente no inventário. | Baixa | Sanear valores; confirmar uso real antes de limpar. |
| `project/src/rustpg_hud_geometry.rs` | 47 | U17: compute não valida requested/offset finitos; U18: testes verificam limites do HUD isolado, não interseção com alvo/touch/hotbar. | Média | Geometria conjunta e testes de interseção em telas pequenas. |
| `project/src/rustpg_immersive.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_inspector.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_java_port.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_luck.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_mob_wire.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_native_entities.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_native_modifiers.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_node_effects.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_nodes.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_num.rs` | 88 | C12: safe_div promete zero para resultado não finito, mas finite(+inf) devolve 1e300; C13: overflow_count pode gerar bilhões de extras se consumidor não limitar; C14: diminishing pode saturar incorretamente antes de dividir. | Média | Contratos explícitos, limites por chamada e testes extremos. |
| `project/src/rustpg_powers.rs` | 94 | C15: recarga por dimensão/entidade e remove em forget; C16: texto usa ID interno e ticks; C17: duração inclusiva now<=until dá tick extra; C18: ativação de outro poder substitui ativo sem indicação (confirmar regra de design). | Alta | Identidade persistente de jogador, textos em segundos, duração consistente. |
| `project/src/rustpg_presentation.rs` | 12 | Sem defeito intrínseco confirmado; fachada de reexports, verificar consumidores. | — | Não remover sem prova de código morto. |
| `project/src/rustpg_profession_plugins.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_profile.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_progression.rs` | 109 | G01: kill_xp concede mínimo 1 mesmo multiplicador zero; G02: nome de profissão desconhecida expõe ID; G03: backlog após limite de 100 mil níveis só processa em nova chamada. | Média | Retornar zero para recompensa inelegível; nome seguro; budget explícito. |
| `project/src/rustpg_relic_active.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_relic_engine.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_rifts.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_scaling.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_script_vm.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_skeletal.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_skillgraph.rs` | 136 | U27: utilitários paginados legados não são fonte da radial; signature_layout depende de layout_tree da UI, contrariando independência do módulo. Não confirmado como morto. | Baixa | Separar topologia de renderer após rastrear consumidores. |
| `project/src/rustpg_starling090.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_target_hud.rs` | 223 | U30: armor clamp 1024 diverge de wire que aceita 10000; U31: max inválido vira 1 e HP 0, aparência de alvo morto quando dado ausente; U32: ray_aabb aceita direção zero se origem dentro e min>max sem validação. | Média | Separar amostra ausente de HP real, limites comuns e validar raio. |
| `project/src/rustpg_timeline.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_ui_animation.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_ui_core.rs` | 92 | U24: scroll_by sem teto de conteúdo; U25: paleta alternativa contradiz cores oficiais HP/SP; consumidor precisa confirmação. | Baixa | Centralizar navegação e paleta após rastrear usos. |
| `project/src/rustpg_ui_input.rs` | 41 | U19: hitboxes expandidas podem sobrepor controles vizinhos (função não conhece outros controles); U20: drag_delta aceita NaN. | Média | Resolver alvos por prioridade/distância; validar gesto. |
| `project/src/rustpg_ui_layout.rs` | 85 | U21: layout bestiário pode extravasar em alturas muito pequenas; U22: viewport usa altura e largura sem limitar lado à largura disponível; U23: testes não verificam todos os limites. | Média | Retângulos finitos dentro do pai; usar mesma geometria para clique e desenho. |
| `project/src/rustpg_ui_system.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_ui_theme.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_ui_widgets.rs` | 49 | U26: bar.clamp propaga NaN. | Baixa | Sanear ratio e geometria antes de desenhar. |
| `project/src/rustpg_ui_window.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_v4_core.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_v4_fatigue.rs` | 30 | S09: adjusted_break_speed/damage retornam NaN e negativos em entradas não finitas ou sem exaustão. | Média | Normalizar entradas no limite do engine. |
| `project/src/rustpg_v4_provenance.rs` | 62 | L01: overflowed irreversível bloqueia XP da dimensão inteira; L02: marcar uma posição já conhecida quando cheio também aciona overflow; L03: decoder aceita entradas duplicadas, reduzindo contagem sem erro; L04: pack alias fora dos limites 26/12/26. | Alta | Proveniência por chunk persistente, migração conservadora, validação de coordenadas e duplicatas. |
| `project/src/rustpg_v4_wire.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_view.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_weapon_training.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_world_vfx.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_xp_rules.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/ai_bootstrap.sh` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/asset_studio_setup.sh` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/audit_assets.py` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/audit_skill_trees.py` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/blockbench_mcp.sh` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/build_creature_model.py` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/generate_ai_context.py` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/generate_creature_portraits.py` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/generate_creatures.py` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/generate_ui_assets.py` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/package_hmod.sh` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/package_native.py` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/paint_ashen_sentinel.py` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/paint_rift_hunter.py` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/rustpg_build.sh` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/rustpg_migrate.py` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `scripts/validate.sh` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `tools/preview4_port.py` | 98 | P01: copytree dirs_exist_ok=True reutiliza arquivos antigos; P02: substituições de manifesto sem validação de contagem/pin; P03: empacota antes de check e não publica atomicamente; P04: laboratório usa pacote único, release usa split — teste não reproduz distribuição. | Alta | Exigir estágio novo, verificar manifesto/pin, publicação atômica e testar par final. |
| `tools/preview4_revamp.py` | 251 | U01: fit com mínimo de 3 px oculta ícones; U02: limites fixos de pan/zoom não derivam da extensão; U03: centros usam escalas diferentes (rework); P05: split troca somente gui/ e entity/, mas move todas as texturas, incluindo item/block; P06: substituição rustpg:entity/ indiscriminada em JSON pode alterar IDs não gráficos; P07: versão exata dos visuais obriga sincronizar releases; U04: teste de botões verifica posições sem simular clique; U05: áreas estreitas dos botões não são protegidas. | Alta | Layout radial único; validar referências de textura individualmente; separar versão compatível de arte; testes de entrada. |
| `tools/sunrise_compat.py` | 119 | P08: visitor encerra em 4096 e retorna lista como se completa; P09: conversões silenciosas de drops descartam falhas; P10: replace globais sem verificar contagem; P11: comentário de input contradiz política de rework. | Alta | Resultado explícito completo/truncado; tratar refusas; contratos por substituição. |
| `tools/sunrise_rpg_rework.py` | 275 | U06: BTreeMap ordena setores por ID, enquanto raízes são ligadas por ordem de catálogo; U07: desenha arestas até nós invisíveis; U08: centro usa zoom bruto; U09: linha até centro do filho cobre seta; U10: ícone só para w>=11; U11: nomes duplicados por etapa e número; C01: tier derivado do tipo, não do estado individual; P12: várias substituições silenciosas; C02: alteração de ranks sem migração explícita no script. | Alta | Ordenar por prerequisitos reais; bordas/escala comum; migrar compras; transportar tier real. |

## Rastreabilidade da interface

`preview4_port.py` aplica compat → rework → atlas → goblins → revamp → embed → split apenas em release. A geometria radial é introduzida no rework e modificada no revamp. O problema de ordem e recorte é confirmado no código, não depende de imagem.

## Estado de testes

A execução anterior [45663491730](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45663491730) compilou e obteve 174/175 testes. [45665120688](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45665120688) estava em andamento na consulta. Nenhum teste novo desta auditoria foi executado.
