# AUDITORIA — RustPG / Sunrise Preview 4

Auditoria **parcial**, base `515457228c1712cac0502dcf3046c43466eef1ee`. 151 arquivos inventariados; 62 revisados integralmente neste ciclo. Linhas contadas sem a quebra final; não confundir achado estático com prova em Android. Alguns achados são riscos que exigem cruzamento de consumidores, não defeitos confirmados em execução. Não completar artificialmente a meta de 60. Nenhum código do jogo editado nesta fase.

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
| `project/src/rustpg_combat.rs` | 242 | C22: player_takes retorna no mínimo 1 mesmo amount zero, podendo criar dano a partir de bloqueio; C23: tipo de arma por sufixo classifica ferramentas desconhecidas como desarmado; C24: cálculo legado divide por health_mult e max HP, perigoso se integrado diretamente ao motor atual que já ajusta atributos; C25: classificação não reconhece generic_kill/out_of_world como exclusões. | Alta | Não conectar fórmula antiga indiscriminadamente; integrar efeitos por hook correto, respeitar zero e bypass. |
| `project/src/rustpg_combat_pipeline.rs` | 111 | C19: ID de extensão permite namespace ou caminho vazio (ex. :); C20: estágio arbitrário pode reintroduzir dano a partir de zero, inclusive após mitigação; C21: registro não captura pânico de extensão, apesar de anunciar proteção contra saída quebrada. | Média | Validar ID completo e contrato do estágio; não reintroduzir dano após cancelamento. |
| `project/src/rustpg_combat_training.rs` | 37 | C08: fallback classifica qualquer dano não listado como espada; flecha de besta também vira arco se esse fallback for usado. | Média | Registrar arma real do ataque; não conceder XP de estilo desconhecido. |
| `project/src/rustpg_combo.rs` | 42 | C09: combo é por entidade/dimensão, não por arma, apesar do comentário; C10: descarta qualquer segundo golpe no mesmo tick, mesmo hits legítimos; teste usa teto 1.25 enquanto implementação limita 1.20. | Média | Identidade de golpe/arma e teste de teto exato. |
| `project/src/rustpg_creatures_generated.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_damage_meter.rs` | 156 | U48: fireball é Fogo antes de teste projétil; U49: cacto/amora classificados veneno; U50: hot_floor ambiente; U51: WITHER não traduzido; U52: formatação decimal ponto, não vírgula; U53: teto 1 milhão diverge da resolução de dano. | Média | Tabela explícita de famílias, unidades localizadas, limites comuns. |
| `project/src/rustpg_damage_visuals.rs` | 53 | U28: tipo elemental agrega fogo/veneno/wither, sem cor própria nesta estrutura; U29: origem/amount não sanitizados nesta camada (verificar wire). | Média | Tipo preciso e validar antes de renderizar. |
| `project/src/rustpg_elite_encounters.rs` | 77 | M04: vampiric em Normal não regenera (confirmado neste módulo, não evil_att); M05: fase de miniboss é calculada mas outgoing só aplica para Boss+; M06: regen não valida regen finita; M07: resolve aplica bônus ofensivo até em dano que bypassa defesa. | Alta | Permitir vampiric, decidir fase por tier coerente, sanear dados e excluir dano especial. |
| `project/src/rustpg_enchant.rs` | 82 | E01: apply_book(current,0) adiciona 1; E02: limite desconhecido vira 1 mesmo para mods com outros níveis; E03: comentário de transcendência ilimitada não corresponde a teto efetivo no enchant_stack. | Média | Distinguir livro inválido e consultar registro real; explicitar teto efetivo. |
| `project/src/rustpg_enchant_stack.rs` | 217 | E04: advance_level_one insere encantamento rejeitado por vanilla se ausente no output; E05: write_maps preserva virtuais de esquerda mesmo se vanilla remove encantamento; E06: i64::MAX truncamento silencioso de nível u64; E07: nível acima de 4351 de Afiamento continua crescendo mas dano extra fica limitado. | Alta | Exigir encantamento aceito e presente em output; preservar progressão válida e explicitar teto. |
| `project/src/rustpg_engine_attributes.rs` | 75 | S06: custo e regeneração expõem unidades sem definição temporal; S07: saída pode atingir 1e300 enquanto stamina limita a 1e9, criando divergência; S08: limites de redução aplicados por piso, sem regra de compra. | Média | Unidades e caps coerentes com engine e catálogo. |
| `project/src/rustpg_engine_combat.rs` | 24 | C11: substring axe também corresponde a pickaxe, classifica picareta como arma pesada. | Média | Classificação por item/tag ou sufixo inequívoco. |
| `project/src/rustpg_engine_nutrition.rs` | 64 | N01: stamina_restore acumula e decai sem método de consumo atômico; N02: bônus antigos são acumulados em nova refeição mesmo após expiração enquanto decaem. Ausência de chamada eat no runtime ainda requer revisão de lib. | Alta | Aplicar refeição autoritativa uma vez; limpar bônus expirados; consumir restauração uma vez. |
| `project/src/rustpg_engine_presentation.rs` | 129 | Sem erro intrínseco na função rank_progress; algoritmo real e bandas Monarca já existem, integração em menu_model deve ser revista. | — | Reutilizar RANK_BANDS e rank_progress sem duplicar limiares. |
| `project/src/rustpg_engine_stamina.rs` | 127 | S01: clamp deixa current NaN; S02: consume(0) retorna true antes de verificar fadiga; S03: recovery_progress divide por mínimo 1 em vez de limiar real quando max<2. | Média | Sanear valores; definir semântica de custo zero; corrigir denominador e teste. |
| `project/src/rustpg_engine_vitality.rs` | 68 | S04: from_vanilla_amount aplica escala fixa de 20, potencial duplicação quando motor já usa max HP real; confirmar chamadas; S05: current/max sem sanitização completa ao carregar perfil. | Alta | Uma fonte de HP; validar perfil e cálculos. |
| `project/src/rustpg_evil_att.rs` | 126 | M01: descrições prometem comportamentos específicos; correspondência de vampirismo, crítico, repulsão e regeneração precisa ser confirmada nos consumidores. O módulo em si só define catálogo, sorteio e berserk. | Pendente | Cruzar com scaling e lib antes de editar. |
| `project/src/rustpg_gathering_rules.rs` | 84 | L05: age aceita estado sem fechamento de colchete; L06: gate recebe apenas creative, exigindo que chamador também negue spectator; L07: amora madura colhida por interação pode não passar pela rota de quebra. | Média | Parser estrito, gate de modo compartilhado e revisar hook de colheita. |
| `project/src/rustpg_gear.rs` | 180 | G04: next_id saturado gera UID repetido; G05: sanitize pode reaproveitar UID existente que aparece depois e truncar bolsa perdendo itens; G06: decode transforma UID 0 em 1; G07: add não valida slot ou raridade antes de armazenar; G08: relíquia arma dá bônus de espada independente do tipo real (ver intenção). | Alta | Alocação checked e sem colisão; validação sem descarte de dados; revisar semântica do slot. |
| `project/src/rustpg_generated_hunter.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_generated_models.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_guard.rs` | 129 | C33: recarga por dimensão e entidade também pode ser zerada como poderes; C34: lista de exclusões omite generic_kill e outros danos especiais, então esquiva pode anulá-los; C35: janela inclusiva dura tick extra; C36: texto em ticks e stamina. | Alta | Identidade persistente, allowlist de ataques elegíveis, intervalo semiaberto, português. |
| `project/src/rustpg_hitboxes.rs` | 44 | C32: alcance zero ainda permite 1 bloco horizontal e 3,5 vertical; alcance negativo também vira zero; broad phase conservadora não pode ser usada como confirmação final. | Média | Usar distância à caixa real, com limite não negativo e narrow phase. |
| `project/src/rustpg_hud.rs` | 24 | U16: hp_fraction aceita NaN; comentário cita rustpg_main inexistente no inventário. | Baixa | Sanear valores; confirmar uso real antes de limpar. |
| `project/src/rustpg_hud_geometry.rs` | 47 | U17: compute não valida requested/offset finitos; U18: testes verificam limites do HUD isolado, não interseção com alvo/touch/hotbar. | Média | Geometria conjunta e testes de interseção em telas pequenas. |
| `project/src/rustpg_immersive.rs` | 76 | G15: cooldown de sequência usa tick absoluto salvo, reinício do mundo pode conflitar; G16: um evento por tick elimina ações legítimas em lote; G17: decode permite claimed inconsistente com best, sem validação semântica. | Média | Identidade de evento e relógio por sessão; validação da trilha. |
| `project/src/rustpg_inspector.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_java_port.rs` | 212 | C41: Attribute::effective usa base original em AddMultipliedBase, dynamic_apply usa resultado aditivo; duas fórmulas divergentes; C42: probability_from_strength(-inf) retorna -1 embora limite matemático seja -inf (contrato de saturação não explícito); C43: scaled_break_speed propaga NaN testado como esperado; C44: regen perde ticks acima de 20 se usada com intervalo maior. | Média | Unificar operações com SDK, validar números e tempo sem alterar balanceamento sem prova. |
| `project/src/rustpg_luck.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_mob_wire.rs` | 72 | U45: versão base não transporta tier/mutação, rework acrescenta apenas ataque; U46: duplicatas de ID aceitas, hp zero recusado e some sem estado explícito; U47: cap 24 pode ocultar alvo se seleção não prioriza mira. | Alta | Wire versionado com tier/mutação, IDs únicos, morte e alvo prioritário. |
| `project/src/rustpg_native_entities.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_native_modifiers.rs` | 43 | S10: teto de bônus HP 2028 diverge do registro ampliado a 1 milhão; S11: migração de base pode alterar atributo de terceiro mod se coincidir numericamente (comentário admite). | Alta | Unificar teto real com catálogo; migrar somente com marcador de propriedade. |
| `project/src/rustpg_node_effects.rs` | 378 | C29: cap genérico 20 torna damage_flat 2048 inalcançável; C30: penetração adicionada ao valor pós-armadura (ordem das outras defesas exige revisar lib); C31: crítico usa roll e não retorna flag; U33: descrição expõe ID e rank hipotético inexistente, formatação fixa 3 decimais; U34: texto afirma integração a partir de flag estática, não teste; L08: rod_wear truncado em 1000 mesmo quando desconto zero; L09: teste de XP de terra retorna 20 e não comprova bloqueio do adaptador. | Alta | Limite por atributo, ordem de dano e resultado tipado; descrições de nó único; testes na fronteira real. |
| `project/src/rustpg_nodes.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_num.rs` | 88 | C12: safe_div promete zero para resultado não finito, mas finite(+inf) devolve 1e300; C13: overflow_count pode gerar bilhões de extras se consumidor não limitar; C14: diminishing pode saturar incorretamente antes de dividir. | Média | Contratos explícitos, limites por chamada e testes extremos. |
| `project/src/rustpg_powers.rs` | 94 | C15: recarga por dimensão/entidade e remove em forget; C16: texto usa ID interno e ticks; C17: duração inclusiva now<=until dá tick extra; C18: ativação de outro poder substitui ativo sem indicação (confirmar regra de design). | Alta | Identidade persistente de jogador, textos em segundos, duração consistente. |
| `project/src/rustpg_presentation.rs` | 12 | Sem defeito intrínseco confirmado; fachada de reexports, verificar consumidores. | — | Não remover sem prova de código morto. |
| `project/src/rustpg_profession_plugins.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_profile.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_progression.rs` | 109 | G01: kill_xp concede mínimo 1 mesmo multiplicador zero; G02: nome de profissão desconhecida expõe ID; G03: backlog após limite de 100 mil níveis só processa em nova chamada. | Média | Retornar zero para recompensa inelegível; nome seguro; budget explícito. |
| `project/src/rustpg_relic_active.rs` | 173 | G09: cooldown usa relógio civil, mudança manual de relógio altera recarga; G10: efeitos ativos por família permitem reativar efeito com outro item da mesma família após troca (revisar intenção); U35: 'Slot' e 'Stamina' sem padronização de idioma. | Média | Persistência monotônica compatível com reload e nome localizado. |
| `project/src/rustpg_relic_engine.rs` | 201 | G11: combine limita a cada etapa, efeito de plugin negativo pode depender da ordem; G12: registro aceita ':' como ID; G13: helpers públicos propagam NaN/inf ou retornam base inválida; G14: snapshot percorre todos plugins por item em cada chamada, verificar custo por tick. | Média | Validar entradas no registro, aplicar limites por estágio e cachear por alteração de equipamento. |
| `project/src/rustpg_rifts.rs` | 233 | R01: DungeonSession não contém flag de recompensa nem conjunto de IDs da onda; R02: portal usa fill destrutivo e remoção por tipo, pode apagar blocos pré-existentes; R03: Grade pode tratar NaN em dano como zero via max; R04: grade C ignora mortes; R05: RoomState::Reward e RoomKind::Exit precisam prova de uso; R06: comandos de efeito dependem de seletor @a respeitar dimensão (confirmar no SDK). | Alta | Sessão idempotente e confirmação de spawn; não destruir construção existente; validar entradas e escopo. |
| `project/src/rustpg_scaling.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_script_vm.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_skeletal.rs` | 93 | M08: default Pose escala zero em vez de identidade; M09: interpolação angular linear percorre caminho longo ao cruzar ±180; M10: cada sample revalida clip e tracks; M11: módulo declara não estar conectado ao renderer. | Média | Pose default identidade; rotação curta; validar/clipes pré-compilados; rastrear integração. |
| `project/src/rustpg_skillgraph.rs` | 136 | U27: utilitários paginados legados não são fonte da radial; signature_layout depende de layout_tree da UI, contrariando independência do módulo. Não confirmado como morto. | Baixa | Separar topologia de renderer após rastrear consumidores. |
| `project/src/rustpg_starling090.rs` | 137 | C37: clean_value impõe cap mínimo 1 mesmo quando atributo tem teto 0,1; C38: reconcile ignora erro de remoção e perde cache, sem retry; C39: effective_in_hooks só lê keys presentes em Stats, ignora bônus externo em atributo ainda não comprado; C40: declare/widen ignora retornos. | Alta | Respeitar teto real e confirmar recusas; iterar catálogo aplicável; diagnóstico no registro. |
| `project/src/rustpg_target_hud.rs` | 223 | U30: armor clamp 1024 diverge de wire que aceita 10000; U31: max inválido vira 1 e HP 0, aparência de alvo morto quando dado ausente; U32: ray_aabb aceita direção zero se origem dentro e min>max sem validação. | Média | Separar amostra ausente de HP real, limites comuns e validar raio. |
| `project/src/rustpg_timeline.rs` | 47 | U36: sample valida todo vetor a cada frame e faz busca linear; interpolação pode gerar infinito em valores extremos válidos. | Baixa | Validar na carga, interpolar de forma finita e usar busca binária. |
| `project/src/rustpg_ui_animation.rs` | 79 | U37: relógio saturante congela animações no limite; U38: easing aceita NaN; U39: pulse usa sin.abs, produz duas oscilações por período. | Baixa | Relógio cíclico e entradas finitas; definir período corretamente. |
| `project/src/rustpg_ui_core.rs` | 92 | U24: scroll_by sem teto de conteúdo; U25: paleta alternativa contradiz cores oficiais HP/SP; consumidor precisa confirmação. | Baixa | Centralizar navegação e paleta após rastrear usos. |
| `project/src/rustpg_ui_input.rs` | 41 | U19: hitboxes expandidas podem sobrepor controles vizinhos (função não conhece outros controles); U20: drag_delta aceita NaN. | Média | Resolver alvos por prioridade/distância; validar gesto. |
| `project/src/rustpg_ui_layout.rs` | 85 | U21: layout bestiário pode extravasar em alturas muito pequenas; U22: viewport usa altura e largura sem limitar lado à largura disponível; U23: testes não verificam todos os limites. | Média | Retângulos finitos dentro do pai; usar mesma geometria para clique e desenho. |
| `project/src/rustpg_ui_system.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_ui_theme.rs` | 117 | U40: cores semânticas HP vermelho e stamina verde contradizem HUD novo; U41: dois conjuntos de tokens (ui_core e theme) com cores diferentes. | Média | Centralizar cores e usar em todos os renderers. |
| `project/src/rustpg_ui_widgets.rs` | 49 | U26: bar.clamp propaga NaN. | Baixa | Sanear ratio e geometria antes de desenhar. |
| `project/src/rustpg_ui_window.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_v4_core.rs` | 188 | G18: desconhecidos com namespace são sempre passivos, mesmo hostis de mods; G19: registro de extensão aceita ID ':'; G20: não recebe modo de jogador, depende inteiramente do adaptador; G21: hp observer ignora dano <=0,05 (não deve ser fonte de dano real). | Média | Classificar por metadados do motor e exigir modo autorizado. |
| `project/src/rustpg_v4_fatigue.rs` | 30 | S09: adjusted_break_speed/damage retornam NaN e negativos em entradas não finitas ou sem exaustão. | Média | Normalizar entradas no limite do engine. |
| `project/src/rustpg_v4_provenance.rs` | 62 | L01: overflowed irreversível bloqueia XP da dimensão inteira; L02: marcar uma posição já conhecida quando cheio também aciona overflow; L03: decoder aceita entradas duplicadas, reduzindo contagem sem erro; L04: pack alias fora dos limites 26/12/26. | Alta | Proveniência por chunk persistente, migração conservadora, validação de coordenadas e duplicatas. |
| `project/src/rustpg_v4_wire.rs` | 87 | U42: dano >1 milhão descartado, mas pipeline permite até 1 bilhão; U43: dano <0,000005 vira 0 na serialização e perde aviso; U44: dimensão ':ab' aceita sem namespace; campos públicos permitem construir notice inválido sem new. | Alta | Alinhar limites com pipeline e formato binário, validar no encode/decode. |
| `project/src/rustpg_view.rs` | — | Não revisado integralmente | Pendente | Leitura integral necessária |
| `project/src/rustpg_weapon_training.rs` | 59 | C26: KNOWN omite armas de mods, não consulta tags; C27: from_held consulta IDs do registro repetidamente por golpe; C28: arco/besta não incluídos em class_of, apesar de serem disciplinas. | Média | Resolver classificação e cache por registro, preservar identidade do projétil. |
| `project/src/rustpg_world_vfx.rs` | 25 | M02: OnceLock impede atualização de chaves se registro for reconstruído; M03: limite de partículas é verificado antes do lote e não inclui as novas; falhas de emissão ignoradas. | Média | Associar chaves ao ciclo de registro e respeitar orçamento restante. |
| `project/src/rustpg_xp_rules.rs` | 45 | Sem falha intrínseca confirmada; whitelist de mineração impede blocos triviais. Proteção de origem e modo depende do adaptador. | — | Preservar testes, cruzar caller. |
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
