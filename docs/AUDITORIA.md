# Auditoria técnica — RustPG / Sunrise Preview 4

**Base:** [commit 5154572](https://github.com/rc2317220-max/Claude-rpg/commit/515457228c1712cac0502dcf3046c43466eef1ee). **Estado:** auditoria parcial, com evidência estática e logs; nenhum teste de jogo aprovado nesta sessão. Fonte de requisitos: conversa «Corrija o build» e instruções atuais. Repositório privado; links exigem acesso.

## Arquitetura e prioridade
Fluxo: project/src e project/data → sunrise_compat.py → sunrise_rpg_rework.py → preview4_port.py / preview4_revamp.py (confirmar ordem exata no workflow) → estágio gerado → compilação SDK → pacote e dependências visuais → runtime Android. A correção deve entrar na fonte que efetivamente chega ao estágio; não confundir código base com código produzido. Prioridade: integridade de perfil e recompensas, build/teste real, combate e progressão, interface, desempenho e limpeza. Gravidade crítica = perda de dados/duplicação; alta = mecânica bloqueada ou regressão relevante; média = UX, manutenção ou risco localizado.

## Evidências estáticas e achados confirmados
Confirmados na fonte indicada, não necessariamente reproduzidos em jogo. A06 é controle existente, com funcionamento pendente; A18/A26 são riscos derivados da implementação; A23 é falha observada em log.

### Registro A — evidência direta
| ID | Gravidade | Arquivo/evidência | Causa / problema | Solução e validação |
|---|---|---|---|---|
| A01 | Alta | [tools/sunrise_rpg_rework.py:86](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_rpg_rework.py#L86) e [tools/sunrise_rpg_rework.py:127](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_rpg_rework.py#L127) | Raízes encadeadas em ordem de definição; setores em BTreeMap alfabético. Cruzamentos são possíveis. | Ordenar setores pela cadeia de pré-requisitos; testar todos os grafos e arestas. |
| A02 | Alta | [tools/sunrise_rpg_rework.py:155](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_rpg_rework.py#L155) | Arestas usam galaxy_nodes completo, embora nós sejam filtrados por viewport. | Usar o mesmo conjunto de nós para desenho e clique; validar pan/zoom e clipping. |
| A03 | Alta | [tools/sunrise_rpg_rework.py:169](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_rpg_rework.py#L169); [tools/preview4_revamp.py:154](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/preview4_revamp.py#L154) | Centro usa zoom puro, nós usam fit × zoom. | Extrair transformação única; verificar distância mínima centro/nó. |
| A04 | Alta | [tools/preview4_revamp.py:156](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/preview4_revamp.py#L156); [tools/sunrise_rpg_rework.py:180](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_rpg_rework.py#L180) | Diâmetro mínimo 3 px, ícone só a partir de 11 px. | Limite legível e pan para árvores maiores; teste em 320 px e resolução real. |
| A05 | Média | [tools/sunrise_rpg_rework.py:161](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_rpg_rework.py#L161) | Arestas terminam no centro e são desenhadas antes dos nós; seta pode ser coberta. | Interceptar contorno do losango; testar ponta visível e vínculo correto. |
| A06 | Média | [tools/preview4_revamp.py:132](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/preview4_revamp.py#L132) | Voltar da árvore já está declarado na renderização e em menu_hits; ação tree_back define page=0. Não é prova de funcionamento no dispositivo. | Testar mouse e toque; eliminar controles concorrentes; aplicar ao Bestiário. |
| A07 | Alta | [project/src/lib.rs:768](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L768) | next_rank vazio e rank_progress=0 fixos. | Derivar da progressão efetiva e testar limiares e patente final. |
| A08 | Alta | [project/src/rustpg_action_combat.rs:37](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_action_combat.rs#L37) | Recuperação não integra next_input; dano vinculado a janela de apenas 2 ticks. | Cadência completa e contexto de golpe único; testes de limites temporais. |
| A09 | Alta | [project/src/rustpg_node_effects.rs:221](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_node_effects.rs#L221) | attr limita todo atributo de arma a 20 antes do teto 2048 de damage_flat. | Teto por atributo e compra validada pelo mesmo cálculo; testar limites. |
| A10 | Alta | [project/src/lib.rs:2484](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L2484) | WorldProgress::default e dia 0 no cálculo do nível. | Usar progresso persistente e tempo real do mundo; testar evolução. |
| A11 | Crítica | [project/src/lib.rs:1330](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L1330) | Login converte erro de leitura em Profile::new e grava sem distinguir ausência de corrupção. | Falhar de modo seguro; backup e migração; teste de erro de leitura sem escrita. |
| A12 | Crítica | [project/src/lib.rs:368](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L368) | Recompensa é persistida antes de teleportar; retorno falho deixa sessão ativa. | ID de recompensa persistente e idempotente; teste de falha repetida e reinício. |
| A13 | Alta | [project/src/lib.rs:418](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L418) | Ramo Active trata jogador dentro da dimensão; saída pode impedir transição. | Máquina de estados com abort/retorno/limpeza e timeout; testar morte e desconexão. |
| A14 | Alta | [project/src/lib.rs:430](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L430) | spawned só é exibido; estado vira Active mesmo com zero inimigos. | Confirmar spawn/IDs de onda e tratar falha; impedir vitória vazia. |
| A15 | Alta | [project/src/rustpg_v4_provenance.rs:12](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_v4_provenance.rs#L12) | overflowed torna toda consulta positiva, nunca é limpo e é serializado. | Proveniência por chunk persistente, sem bloquear dimensão inteira; teste 50 mil+. |
| A16 | Média | [project/src/rustpg_v4_provenance.rs:36](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_v4_provenance.rs#L36) | decode aceita entradas duplicadas e reduz silenciosamente contagem real. | Rejeitar duplicatas e testar snapshots inválidos. |
| A17 | Média | [project/src/rustpg_action_combat.rs:36](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_action_combat.rs#L36) | timing sanitiza mastery não finita; cálculo de dano não usa a mesma sanitização. | Normalizar entrada uma vez; testar NaN/inf e limites. |
| A18 | Média | [project/src/lib.rs:581](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L581) | Cache do layout antigo usa somente ID e quantidade; troca de catálogo com mesmo número pode reaproveitar geometria obsoleta. | Chave por revisão/hash do grafo ou eliminar cache antigo só após comprovar ausência de consumidores. |
| A19 | Média | [tools/sunrise_rpg_rework.py:103](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_rpg_rework.py#L103) | Nome de nó expandido usa sufixo numérico · 2 etc. | Nomes próprios pt-BR e descrição da unidade; testar catálogo inteiro. |
| A20 | Média | [project/src/lib.rs:606](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L606) | Pré-requisitos exibem rank e podem cair em ID interno (map_or). | Texto localizado, nome público e fallback amigável; teste de strings. |
| A21 | Média | [AI_AGENT_PROTOCOL.md:67](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/AI_AGENT_PROTOCOL.md#L67); [UI_MAP.md:29](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/UI_MAP.md#L29) | Documentação exige árvore fixa/paginada; contradiz o estágio Preview 4. | Documentar radial, pan e zoom; atualizar gerador dos mapas. |
| A22 | Alta | [project/src/lib.rs:1](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L1); [project/src/lib.rs:94](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L94); [tools/preview4_revamp.py:6](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/preview4_revamp.py#L6) | Versões de origem e estágio divergentes; allow(dead_code) oculta avisos. | Fonte única de versão; relatórios de warnings; refatoração em módulos sem perda funcional. |
| A23 | Alta | [Actions 45675432926](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45675432926) | native_candidate, official_preview e creature_inspection passaram; native_runtime falhou: no world open / no player yet. | Falhar cedo se abertura não confirmada, registrar diagnóstico de carregamento e repetir teste real. |
| A24 | Alta | [tools/sunrise_compat.py:5](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_compat.py#L5); [tools/preview4_revamp.py:9](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/preview4_revamp.py#L9) | Transformações por texto e replace sem contagem em vários pontos; mudança de âncora pode não aplicar ou abortar. | Manifesto de transformações, contagem exata, diffs gerados e testes de integração. |
| A25 | Média | [project/src/lib.rs:1330](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L1330) | Resultado de PROFILE.set descartado no login. | Registrar/propagar falha e não anunciar sucesso; teste de armazenamento indisponível. |
| A26 | Média | [project/src/rustpg_engine_nutrition.rs:40](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_engine_nutrition.rs#L40) | Bônus após expiração decaem, mas novo eat pode acumular resíduo; tick reduz stamina_restore sem consumo explícito neste módulo. | Definir semântica de consumo e expiração; confirmar integração no servidor. |

## Hipóteses e requisitos pendentes
Itens B são relatos anteriores, requisitos ou hipóteses adicionais; não são defeitos comprovados no commit. As rotas indicam arquivos a investigar. Cada item deve receber fonte, teste e commit antes de ser fechado.

| ID | Gravidade potencial | Arquivo/área | Investigação e critério |
|---|---|---|---|
| B01 | Alta | lib.rs / SDK / UI | Renderizar jogador local com skin e armadura reais via GuiEntity::Existing; confirmar suporte ao jogador local, fallback e rotação sem duplicação. |
| B02 | Alta | lib.rs, rustpg_gear.rs, rustpg_relic_engine.rs | Oito slots: cabeça, peito, pernas, pés, arma, anel, colar, amuleto; tooltips de raridade e bônus; verificar sincronização e item vazio. |
| B03 | Alta | rustpg_ui_window.rs / rustpg_ui_layout.rs | Bestiário: comparar geometria de desenho e hit-test; Voltar no canto; testar seleção, scroll e toque. |
| B04 | Média | rustpg_bestiary.rs / lib.rs | Entrada somente por derrota, detectar criaturas de outros mods, renderização viva e fallback para entidade ausente. |
| B05 | Alta | lib.rs / rustpg_node_effects.rs | Penetração: verificar ordem de chamadas na pipeline completa; aplicar antes de mitigação final sem ignorar barreira ou relíquias. |
| B06 | Alta | lib.rs / rustpg_engine_nutrition.rs | Nutrition::eat sem chamada efetiva relatada; mapear eventos de alimento e consumo; teste de ingestão e drenagem. |
| B07 | Alta | rustpg_combat.rs / lib.rs | Crítico, resistências, bônus de projétil e alcance: confirmar consumidores reais; comparar dano medido com atributos. |
| B08 | Alta | lib.rs / rustpg_powers.rs | Grupo por distância sem dimensão e recarga vinculada a entidade; testar troca de dimensão, logout e reconexão. |
| B09 | Alta | lib.rs / rustpg_action_combat.rs | Sangramento e roubo de vida associados por janela temporal; assegurar origem do dano, atacante, arma, vítima e token de golpe. |
| B10 | Média | rustpg_evil_att.rs | Mutação vampiric em mob Normal: conferir regeneração por tier/mutação. |
| B11 | Alta | lib.rs / rustpg_rifts.rs | Consulta de entidades falha ou onda vazia; distinguir erro de ausência, persistir IDs; testar logout e encerramento da instância. |
| B12 | Alta | lib.rs living_drops | Criativo/espectador: confirmar bloqueio de todas as recompensas e caminhos alternativos. |
| B13 | Alta | lib.rs / rustpg_v4_provenance.rs | Marcação de blocos comuns, reinício e bônus de drops; persistência e teste de duplicação. |
| B14 | Alta | lib.rs / rustpg_gathering_rules.rs | Caça e criação de animais passivos possivelmente excluídos por scales(kind); testar por espécie. |
| B15 | Alta | rustpg_enchant_stack.rs | Livros múltiplos e incompatibilidade: validar cada encantamento e consumo atômico. |
| B16 | Crítica | rustpg_profile.rs | decode permissivo relatado; validar esquema, versão, dados truncados, NaN e limites; recuperar backup, nunca zerar automaticamente. |
| B17 | Alta | rustpg_profile.rs / rustpg_nodes.rs | Migração de max_rank e remoção de nó; manter histórico de moeda/valor pago; respec conserva pontos. |
| B18 | Alta | lib.rs / rustpg_view.rs | Respec no menu e comando /rustpg respec; testar autorização e reembolso. |
| B19 | Média | lib.rs / rustpg_view.rs | Status só oito maiores atributos relatado; agrupar unidades e mostrar valores efetivos, base e bônus. |
| B20 | Alta | tools/preview4_revamp.py / HUD | HUD abaixo de 260×150 e alvo em 320px; comparar suppress vanilla e desenho em todas as resoluções. |
| B21 | Média | rustpg_mob_wire.rs / lib.rs | Tier e mutação no wire; confirmar dados reais no cliente e fallback. |
| B22 | Alta | lib.rs / rustpg_ui_input.rs | Tecla V remapeável, fallback touch; evitar duplicidade e consumo indevido de input. |
| B23 | Média | rustpg_progression.rs / UI | Régua de patente até Monarca, valores reais de progresso e aviso de Fenda apenas com ressonância suficiente. |
| B24 | Média | rustpg_relic_engine.rs / rustpg_ui_window.rs | Relíquias: raridade, slots e unidades pt-BR; 1,2% deve aparecer corretamente. |
| B25 | Média | rustpg_inspector.rs | Não exibir ações internas, IDs e comandos de debug ao jogador. |
| B26 | Média | rustpg_view.rs / catálogo / scripts | Auditar todos os textos finais: acentos, inglês, rank único, números, traduções, descrições. |
| B27 | Alta | rustpg_engine_stamina.rs / rustpg_v4_fatigue.rs | Testar sprint, salto, ataque pesado, regen, zero e penalidade sem burlar exaustão com pulo. |
| B28 | Alta | rustpg_native_entities.rs / generated / assets | Testar hitboxes, colisões, golpes, IA, animações, VFX e comportamento de chefes no jogo. |
| B29 | Alta | rustpg_profession_plugins.rs / rustpg_script_vm.rs | Validar permissões de plugins, entradas, orçamento de execução, erros e integração real. |
| B30 | Alta | rustpg_v4_wire.rs / rustpg_mob_wire.rs | Investigar pacotes truncados, tamanho, versão, NaN e IDs reutilizados; fuzz e teste de desconexão. |
| B31 | Alta | lib.rs / runtime | Investigar locks em callbacks, custo por tick, varreduras O(n²), mapas sem expiração, alocações por frame; medir p95/p99. |
| B32 | Alta | tools/preview4_port.py / build-on-demand.yml | Verificar cadeia de build, SDK fixado, manifesto, pacote nativo e dependências visuais, hash e reprodução. |
| B33 | Média | scripts/generate_ai_context.py / validate.sh | Regeneração não deve reintroduzir documentação antiga; testar geração determinística e checks. |
| B34 | Alta | project/data / assets | Validar todos os IDs cruzados, tabelas loot, receitas, colisões de namespace, UV, textura e modelos. |
| B35 | Alta | lib.rs / project/src | Analisar transações de compra e equipar, concorrência, duplicação e efeitos reaplicados após reconectar. |
| B36 | Média | rustpg_ui_window.rs / rustpg_ui_system.rs | Investigar clipping de tooltips, texto longo, escala de interface, arrastar versus clicar, navegação sem mouse. |
| B37 | Alta | lib.rs / rustpg_scaling.rs | Medir escalonamento de HP/dano/defesa e persistência por entidade; testar bosses e monstros comuns. |
| B38 | Alta | .github/workflows / .agent/run.sh | Verificar permissões mínimas, entradas de comandos, cancelamentos concorrentes, segredos sem vazamento e evidência por SHA. |
| B39 | Média | project/src/lib.rs / releases / README | Modularizar e remover somente código comprovadamente morto; preservar saves, versões e artefatos necessários. |

## Cobertura e limites
Inventário Git: 73 arquivos project/src, 36 project/data, 17 scripts/, 19 workflows, 4 transformadores = 149 arquivos. Leitura integral exibida: rustpg_action_combat.rs (50 linhas), rustpg_engine_nutrition.rs (64), rustpg_v4_provenance.rs (62), UI_MAP.md (29 incluindo linha final conforme retorno). Demais fontes acima consultadas por trechos/filtros; não marcar leitura integral. Total de 65 registros A+B não equivale a 65 bugs confirmados. Não existe aprovação de runtime neste ponto.

## Checklist de execução
- [ ] A01 — [tools/sunrise_rpg_rework.py:86](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_rpg_rework.py#L86) e [tools/sunrise_rpg_rework.py:127](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_rpg_rework.py#L127): correção/validação
- [ ] A02 — [tools/sunrise_rpg_rework.py:155](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_rpg_rework.py#L155): correção/validação
- [ ] A03 — [tools/sunrise_rpg_rework.py:169](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_rpg_rework.py#L169); [tools/preview4_revamp.py:154](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/preview4_revamp.py#L154): correção/validação
- [ ] A04 — [tools/preview4_revamp.py:156](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/preview4_revamp.py#L156); [tools/sunrise_rpg_rework.py:180](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_rpg_rework.py#L180): correção/validação
- [ ] A05 — [tools/sunrise_rpg_rework.py:161](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_rpg_rework.py#L161): correção/validação
- [ ] A06 — [tools/preview4_revamp.py:132](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/preview4_revamp.py#L132): correção/validação
- [ ] A07 — [project/src/lib.rs:768](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L768): correção/validação
- [ ] A08 — [project/src/rustpg_action_combat.rs:37](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_action_combat.rs#L37): correção/validação
- [ ] A09 — [project/src/rustpg_node_effects.rs:221](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_node_effects.rs#L221): correção/validação
- [ ] A10 — [project/src/lib.rs:2484](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L2484): correção/validação
- [ ] A11 — [project/src/lib.rs:1330](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L1330): correção/validação
- [ ] A12 — [project/src/lib.rs:368](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L368): correção/validação
- [ ] A13 — [project/src/lib.rs:418](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L418): correção/validação
- [ ] A14 — [project/src/lib.rs:430](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L430): correção/validação
- [ ] A15 — [project/src/rustpg_v4_provenance.rs:12](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_v4_provenance.rs#L12): correção/validação
- [ ] A16 — [project/src/rustpg_v4_provenance.rs:36](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_v4_provenance.rs#L36): correção/validação
- [ ] A17 — [project/src/rustpg_action_combat.rs:36](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_action_combat.rs#L36): correção/validação
- [ ] A18 — [project/src/lib.rs:581](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L581): correção/validação
- [ ] A19 — [tools/sunrise_rpg_rework.py:103](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_rpg_rework.py#L103): correção/validação
- [ ] A20 — [project/src/lib.rs:606](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L606): correção/validação
- [ ] A21 — [AI_AGENT_PROTOCOL.md:67](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/AI_AGENT_PROTOCOL.md#L67); [UI_MAP.md:29](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/UI_MAP.md#L29): correção/validação
- [ ] A22 — [project/src/lib.rs:1](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L1); [project/src/lib.rs:94](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L94); [tools/preview4_revamp.py:6](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/preview4_revamp.py#L6): correção/validação
- [ ] A23 — [Actions 45675432926](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45675432926): correção/validação
- [ ] A24 — [tools/sunrise_compat.py:5](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_compat.py#L5); [tools/preview4_revamp.py:9](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/preview4_revamp.py#L9): correção/validação
- [ ] A25 — [project/src/lib.rs:1330](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs#L1330): correção/validação
- [ ] A26 — [project/src/rustpg_engine_nutrition.rs:40](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_engine_nutrition.rs#L40): correção/validação
- [ ] B01 — lib.rs / SDK / UI: correção/validação
- [ ] B02 — lib.rs, rustpg_gear.rs, rustpg_relic_engine.rs: correção/validação
- [ ] B03 — rustpg_ui_window.rs / rustpg_ui_layout.rs: correção/validação
- [ ] B04 — rustpg_bestiary.rs / lib.rs: correção/validação
- [ ] B05 — lib.rs / rustpg_node_effects.rs: correção/validação
- [ ] B06 — lib.rs / rustpg_engine_nutrition.rs: correção/validação
- [ ] B07 — rustpg_combat.rs / lib.rs: correção/validação
- [ ] B08 — lib.rs / rustpg_powers.rs: correção/validação
- [ ] B09 — lib.rs / rustpg_action_combat.rs: correção/validação
- [ ] B10 — rustpg_evil_att.rs: correção/validação
- [ ] B11 — lib.rs / rustpg_rifts.rs: correção/validação
- [ ] B12 — lib.rs living_drops: correção/validação
- [ ] B13 — lib.rs / rustpg_v4_provenance.rs: correção/validação
- [ ] B14 — lib.rs / rustpg_gathering_rules.rs: correção/validação
- [ ] B15 — rustpg_enchant_stack.rs: correção/validação
- [ ] B16 — rustpg_profile.rs: correção/validação
- [ ] B17 — rustpg_profile.rs / rustpg_nodes.rs: correção/validação
- [ ] B18 — lib.rs / rustpg_view.rs: correção/validação
- [ ] B19 — lib.rs / rustpg_view.rs: correção/validação
- [ ] B20 — tools/preview4_revamp.py / HUD: correção/validação
- [ ] B21 — rustpg_mob_wire.rs / lib.rs: correção/validação
- [ ] B22 — lib.rs / rustpg_ui_input.rs: correção/validação
- [ ] B23 — rustpg_progression.rs / UI: correção/validação
- [ ] B24 — rustpg_relic_engine.rs / rustpg_ui_window.rs: correção/validação
- [ ] B25 — rustpg_inspector.rs: correção/validação
- [ ] B26 — rustpg_view.rs / catálogo / scripts: correção/validação
- [ ] B27 — rustpg_engine_stamina.rs / rustpg_v4_fatigue.rs: correção/validação
- [ ] B28 — rustpg_native_entities.rs / generated / assets: correção/validação
- [ ] B29 — rustpg_profession_plugins.rs / rustpg_script_vm.rs: correção/validação
- [ ] B30 — rustpg_v4_wire.rs / rustpg_mob_wire.rs: correção/validação
- [ ] B31 — lib.rs / runtime: correção/validação
- [ ] B32 — tools/preview4_port.py / build-on-demand.yml: correção/validação
- [ ] B33 — scripts/generate_ai_context.py / validate.sh: correção/validação
- [ ] B34 — project/data / assets: correção/validação
- [ ] B35 — lib.rs / project/src: correção/validação
- [ ] B36 — rustpg_ui_window.rs / rustpg_ui_system.rs: correção/validação
- [ ] B37 — lib.rs / rustpg_scaling.rs: correção/validação
- [ ] B38 — .github/workflows / .agent/run.sh: correção/validação
- [ ] B39 — project/src/lib.rs / releases / README: correção/validação

## Portões de validação
- [ ] Leitura de cada arquivo da primeira à última linha; registrar cobertura, número de linhas e SHA.
- [ ] Testes de grafo: ordem dos setores, arestas, centro, visibilidade, hitbox, tamanho mínimo, pan, zoom.
- [ ] Testes de persistência: corrupção, backup, moeda, respec, reload, recompensa Fenda idempotente.
- [ ] Testes de combate: cooldown, recuperação, acerto único, crítico, projétil, mitigação e dimensões.
- [ ] Testes de UI em 240/320/560 px e dispositivo Android: Voltar, mouse/toque, retrato 3D, oito slots, atributos reais, HUD e textos.
- [ ] Testes de desempenho: FPS, p95/p99 de tick e frame, memória, 1/20/100 mobs, mapa grande e sessão longa; comparar baseline.
- [ ] Compilar pacote final e executar dentro de Sunrise Preview 4; anexar SHA, link Actions, logs e screenshots reais.
- [ ] Atualizar docs/PROGRESSO.md e docs/AUDITORIA.md por commit; não declarar sistema funcional só por compilação.

## Registro de mudanças
[Checklist inicial no GitHub](https://github.com/rc2317220-max/Claude-rpg/commit/50f8b5a0eec24c4369011a5c3a4942dbd6a9c480). Ainda sem validação de correções de código. [Última execução inspecionada](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45675432926) refere-se a outro SHA, c0f8687, e falhou no runtime. Não usar como evidência de funcionamento da base auditada.

## Atualização desta sessão
- [21ffa5e](https://github.com/rc2317220-max/Claude-rpg/commit/21ffa5ee9111fc6f2ab70b04f2f4abcbd1814d7f): documentação radial e gerador atualizados.
- [2c81b4b](https://github.com/rc2317220-max/Claude-rpg/commit/2c81b4ba43698737af98bdf45ccbe9527774ee24): setores na ordem do catálogo, linhas entre nós visíveis, terminação no losango, escala comum do hub, diâmetro mínimo 11 px, testes de geometria. Ajustar prioriza legibilidade; árvores grandes exigem pan.
- [2d2c029](https://github.com/rc2317220-max/Claude-rpg/commit/2d2c0296c02e24bef247842306fc1d26dd6c66c3): Voltar único nas duas telas, catálogo rotulado MAPA, hit-test Bestiário no mesmo layout e linha y+36, teste de navegação. B03 confirmado no código original (rustpg_ui_window.rs 1808–1830 e 2520–2528).
- [Actions Preview 4 do último commit](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45745643234) e [validação estável](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45745643272): disparados, resultado final ainda não obtido. Não marcar A01–A06/B03 resolvidos antes de CI e teste visual real.
- [PROGRESSO.md](https://github.com/rc2317220-max/Claude-rpg/blob/main/docs/PROGRESSO.md) permanece como checklist de continuidade. Resumo 3D, atributos e demais sistemas ainda não foram corrigidos.

## Inventário por arquivo (base congelada)

Não confundir tamanho do blob com leitura; os arquivos pendentes não foram avaliados, portanto não significam ausência de bugs. Contagem de linhas apenas quando observada; demais ficam em aberto.

| Arquivo | Bytes | Linhas conhecidas | Leitura | Achados / referências |
|---|---:|---:|---|---|
| [.github/workflows/agent-terminal.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/agent-terminal.yml) | 1048 | — | Pendente | — |
| [.github/workflows/ai-tooling.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/ai-tooling.yml) | 2292 | — | Pendente | — |
| [.github/workflows/asset-studio.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/asset-studio.yml) | 2913 | — | Pendente | — |
| [.github/workflows/blockbench-desktop.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/blockbench-desktop.yml) | 2016 | — | Pendente | — |
| [.github/workflows/blockbench-model-jobs.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/blockbench-model-jobs.yml) | 7402 | — | Pendente | — |
| [.github/workflows/build-on-demand.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/build-on-demand.yml) | 4322 | — | Pendente | — |
| [.github/workflows/divine-powers.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/divine-powers.yml) | 2528 | — | Pendente | — |
| [.github/workflows/extract-project.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/extract-project.yml) | 2848 | — | Pendente | — |
| [.github/workflows/goblin-boss-reference.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/goblin-boss-reference.yml) | 1207 | — | Pendente | — |
| [.github/workflows/goblin-family.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/goblin-family.yml) | 2323 | — | Pendente | — |
| [.github/workflows/goblin-vision-agent.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/goblin-vision-agent.yml) | 2374 | — | Pendente | — |
| [.github/workflows/import-starling-preview4-reference.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/import-starling-preview4-reference.yml) | 2258 | — | Pendente | — |
| [.github/workflows/rift-mantis-forge.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/rift-mantis-forge.yml) | 1446 | — | Pendente | — |
| [.github/workflows/rustpg-migrate.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/rustpg-migrate.yml) | 3426 | — | Pendente | — |
| [.github/workflows/sentinel-v4-recovery.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/sentinel-v4-recovery.yml) | 2302 | — | Pendente | — |
| [.github/workflows/storm-gryphon-forge.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/storm-gryphon-forge.yml) | 1475 | — | Pendente | — |
| [.github/workflows/sunrise-java-bridge.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/sunrise-java-bridge.yml) | 1393 | — | Pendente | — |
| [.github/workflows/sunrise-preview4-lab.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/sunrise-preview4-lab.yml) | 9713 | 200 | Integral | A23, B32, B38 |
| [.github/workflows/validate.yml](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/.github/workflows/validate.yml) | 3979 | 99 | Integral | — |
| [project/data/rustpg/horizon/entity_type/abyss_weaver.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/entity_type/abyss_weaver.json) | 215 | — | Pendente | — |
| [project/data/rustpg/horizon/entity_type/ashen_sentinel.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/entity_type/ashen_sentinel.json) | 219 | — | Pendente | — |
| [project/data/rustpg/horizon/entity_type/crystal_wyrm.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/entity_type/crystal_wyrm.json) | 214 | — | Pendente | — |
| [project/data/rustpg/horizon/entity_type/dusk_revenant.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/entity_type/dusk_revenant.json) | 214 | — | Pendente | — |
| [project/data/rustpg/horizon/entity_type/eclipse_ogre.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/entity_type/eclipse_ogre.json) | 216 | — | Pendente | — |
| [project/data/rustpg/horizon/entity_type/ember_imp.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/entity_type/ember_imp.json) | 217 | — | Pendente | — |
| [project/data/rustpg/horizon/entity_type/grave_knight.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/entity_type/grave_knight.json) | 219 | — | Pendente | — |
| [project/data/rustpg/horizon/entity_type/mana_hound.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/entity_type/mana_hound.json) | 218 | — | Pendente | — |
| [project/data/rustpg/horizon/entity_type/rift_hunter.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/entity_type/rift_hunter.json) | 218 | — | Pendente | — |
| [project/data/rustpg/horizon/entity_type/rift_mantis.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/entity_type/rift_mantis.json) | 215 | — | Pendente | — |
| [project/data/rustpg/horizon/entity_type/rune_colossus.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/entity_type/rune_colossus.json) | 216 | — | Pendente | — |
| [project/data/rustpg/horizon/entity_type/storm_gryphon.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/entity_type/storm_gryphon.json) | 216 | — | Pendente | — |
| [project/data/rustpg/horizon/item/abyss_heart.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/item/abyss_heart.json) | 331 | — | Pendente | — |
| [project/data/rustpg/horizon/item/hunter_token.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/item/hunter_token.json) | 327 | — | Pendente | — |
| [project/data/rustpg/horizon/item/rift_blade.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/item/rift_blade.json) | 565 | — | Pendente | — |
| [project/data/rustpg/horizon/item/sentinel_charm.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/horizon/item/sentinel_charm.json) | 329 | — | Pendente | — |
| [project/data/rustpg/loot_table/entities/abyss_weaver.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/loot_table/entities/abyss_weaver.json) | 204 | — | Pendente | — |
| [project/data/rustpg/loot_table/entities/ashen_sentinel.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/loot_table/entities/ashen_sentinel.json) | 203 | — | Pendente | — |
| [project/data/rustpg/loot_table/entities/crystal_wyrm.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/loot_table/entities/crystal_wyrm.json) | 208 | — | Pendente | — |
| [project/data/rustpg/loot_table/entities/dusk_revenant.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/loot_table/entities/dusk_revenant.json) | 204 | — | Pendente | — |
| [project/data/rustpg/loot_table/entities/eclipse_ogre.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/loot_table/entities/eclipse_ogre.json) | 203 | — | Pendente | — |
| [project/data/rustpg/loot_table/entities/ember_imp.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/loot_table/entities/ember_imp.json) | 206 | — | Pendente | — |
| [project/data/rustpg/loot_table/entities/grave_knight.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/loot_table/entities/grave_knight.json) | 204 | — | Pendente | — |
| [project/data/rustpg/loot_table/entities/mana_hound.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/loot_table/entities/mana_hound.json) | 208 | — | Pendente | — |
| [project/data/rustpg/loot_table/entities/rift_hunter.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/loot_table/entities/rift_hunter.json) | 204 | — | Pendente | — |
| [project/data/rustpg/loot_table/entities/rift_mantis.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/loot_table/entities/rift_mantis.json) | 204 | — | Pendente | — |
| [project/data/rustpg/loot_table/entities/rune_colossus.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/loot_table/entities/rune_colossus.json) | 204 | — | Pendente | — |
| [project/data/rustpg/loot_table/entities/storm_gryphon.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/loot_table/entities/storm_gryphon.json) | 201 | — | Pendente | — |
| [project/data/rustpg/plugins/activities_advanced.rustpg](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/plugins/activities_advanced.rustpg) | 271 | — | Pendente | — |
| [project/data/rustpg/plugins/extra.rustpg](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/plugins/extra.rustpg) | 236 | — | Pendente | — |
| [project/data/rustpg/plugins/professions.rustpg](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/plugins/professions.rustpg) | 434 | — | Pendente | — |
| [project/data/rustpg/plugins_vm/bonus.rustpgx](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/plugins_vm/bonus.rustpgx) | 248 | — | Pendente | — |
| [project/data/rustpg/recipe/abyss_heart.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/recipe/abyss_heart.json) | 245 | — | Pendente | — |
| [project/data/rustpg/recipe/hunter_token.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/recipe/hunter_token.json) | 245 | — | Pendente | — |
| [project/data/rustpg/recipe/rift_blade.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/recipe/rift_blade.json) | 243 | — | Pendente | — |
| [project/data/rustpg/recipe/sentinel_charm.json](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/data/rustpg/recipe/sentinel_charm.json) | 252 | — | Pendente | — |
| [project/src/lib.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/lib.rs) | 236840 | 4133 | Parcial | A07, A10–A14, A18, A20, A22, A25 |
| [project/src/rustpg_action_combat.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_action_combat.rs) | 3446 | 50 | Integral | A08, A17 |
| [project/src/rustpg_actions.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_actions.rs) | 3288 | — | Pendente | — |
| [project/src/rustpg_attributes.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_attributes.rs) | 25560 | — | Pendente | — |
| [project/src/rustpg_bar_vfx.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_bar_vfx.rs) | 2074 | — | Pendente | — |
| [project/src/rustpg_bestiary.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_bestiary.rs) | 1452 | — | Pendente | — |
| [project/src/rustpg_boss_ui.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_boss_ui.rs) | 8279 | — | Pendente | — |
| [project/src/rustpg_combat.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_combat.rs) | 7890 | — | Pendente | — |
| [project/src/rustpg_combat_pipeline.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_combat_pipeline.rs) | 4473 | 111 | Integral | — |
| [project/src/rustpg_combat_training.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_combat_training.rs) | 1517 | — | Pendente | — |
| [project/src/rustpg_combo.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_combo.rs) | 2332 | — | Pendente | — |
| [project/src/rustpg_creatures_generated.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_creatures_generated.rs) | 106041 | — | Pendente | — |
| [project/src/rustpg_damage_meter.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_damage_meter.rs) | 7224 | — | Pendente | — |
| [project/src/rustpg_damage_visuals.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_damage_visuals.rs) | 1284 | — | Pendente | — |
| [project/src/rustpg_elite_encounters.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_elite_encounters.rs) | 3695 | 77 | Integral | — |
| [project/src/rustpg_enchant.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_enchant.rs) | 2982 | — | Pendente | — |
| [project/src/rustpg_enchant_stack.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_enchant_stack.rs) | 10013 | — | Pendente | — |
| [project/src/rustpg_engine_attributes.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_engine_attributes.rs) | 2755 | 75 | Integral | — |
| [project/src/rustpg_engine_combat.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_engine_combat.rs) | 696 | — | Pendente | — |
| [project/src/rustpg_engine_nutrition.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_engine_nutrition.rs) | 1958 | 64 | Integral | A26, B06 |
| [project/src/rustpg_engine_presentation.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_engine_presentation.rs) | 3767 | 129 | Integral | — |
| [project/src/rustpg_engine_stamina.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_engine_stamina.rs) | 4420 | 127 | Integral | — |
| [project/src/rustpg_engine_vitality.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_engine_vitality.rs) | 2067 | 68 | Integral | — |
| [project/src/rustpg_evil_att.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_evil_att.rs) | 3799 | 126 | Integral | — |
| [project/src/rustpg_gathering_rules.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_gathering_rules.rs) | 3746 | — | Pendente | — |
| [project/src/rustpg_gear.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_gear.rs) | 8190 | 180 | Integral | — |
| [project/src/rustpg_generated_hunter.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_generated_hunter.rs) | 11748 | — | Pendente | — |
| [project/src/rustpg_generated_models.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_generated_models.rs) | 10447 | — | Pendente | — |
| [project/src/rustpg_guard.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_guard.rs) | 6996 | — | Pendente | — |
| [project/src/rustpg_hitboxes.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_hitboxes.rs) | 2162 | — | Pendente | — |
| [project/src/rustpg_hud.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_hud.rs) | 647 | — | Pendente | — |
| [project/src/rustpg_hud_geometry.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_hud_geometry.rs) | 2131 | — | Pendente | — |
| [project/src/rustpg_immersive.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_immersive.rs) | 3822 | — | Pendente | — |
| [project/src/rustpg_inspector.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_inspector.rs) | 22320 | — | Pendente | — |
| [project/src/rustpg_java_port.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_java_port.rs) | 9309 | — | Pendente | — |
| [project/src/rustpg_luck.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_luck.rs) | 18614 | — | Pendente | — |
| [project/src/rustpg_mob_wire.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_mob_wire.rs) | 3570 | — | Pendente | — |
| [project/src/rustpg_native_entities.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_native_entities.rs) | 13869 | — | Pendente | — |
| [project/src/rustpg_native_modifiers.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_native_modifiers.rs) | 1847 | — | Pendente | — |
| [project/src/rustpg_node_effects.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_node_effects.rs) | 17895 | — | Parcial | A09, B05 |
| [project/src/rustpg_nodes.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_nodes.rs) | 13025 | — | Parcial | A01, A19, B17 |
| [project/src/rustpg_num.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_num.rs) | 2552 | — | Pendente | — |
| [project/src/rustpg_powers.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_powers.rs) | 4924 | 94 | Integral | — |
| [project/src/rustpg_presentation.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_presentation.rs) | 663 | — | Pendente | — |
| [project/src/rustpg_profession_plugins.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_profession_plugins.rs) | 9012 | — | Pendente | — |
| [project/src/rustpg_profile.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_profile.rs) | 33625 | 827 | Integral | — |
| [project/src/rustpg_progression.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_progression.rs) | 3200 | — | Integral | B23 |
| [project/src/rustpg_relic_active.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_relic_active.rs) | 7712 | — | Pendente | — |
| [project/src/rustpg_relic_engine.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_relic_engine.rs) | 9270 | — | Pendente | — |
| [project/src/rustpg_rifts.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_rifts.rs) | 16019 | — | Pendente | — |
| [project/src/rustpg_scaling.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_scaling.rs) | 13988 | — | Pendente | — |
| [project/src/rustpg_script_vm.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_script_vm.rs) | 8568 | — | Pendente | — |
| [project/src/rustpg_skeletal.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_skeletal.rs) | 4338 | — | Pendente | — |
| [project/src/rustpg_skillgraph.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_skillgraph.rs) | 5574 | — | Pendente | — |
| [project/src/rustpg_starling090.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_starling090.rs) | 6196 | — | Pendente | — |
| [project/src/rustpg_target_hud.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_target_hud.rs) | 6361 | — | Pendente | — |
| [project/src/rustpg_timeline.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_timeline.rs) | 2090 | — | Pendente | — |
| [project/src/rustpg_ui_animation.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_ui_animation.rs) | 2317 | — | Pendente | — |
| [project/src/rustpg_ui_core.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_ui_core.rs) | 2385 | — | Pendente | — |
| [project/src/rustpg_ui_input.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_ui_input.rs) | 1433 | — | Pendente | — |
| [project/src/rustpg_ui_layout.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_ui_layout.rs) | 3038 | — | Pendente | — |
| [project/src/rustpg_ui_system.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_ui_system.rs) | 46047 | — | Pendente | — |
| [project/src/rustpg_ui_theme.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_ui_theme.rs) | 3823 | — | Pendente | — |
| [project/src/rustpg_ui_widgets.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_ui_widgets.rs) | 1020 | — | Pendente | — |
| [project/src/rustpg_ui_window.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_ui_window.rs) | 98654 | — | Parcial | A05, A06, B03, B36 |
| [project/src/rustpg_v4_core.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_v4_core.rs) | 8896 | — | Pendente | — |
| [project/src/rustpg_v4_fatigue.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_v4_fatigue.rs) | 1386 | — | Pendente | — |
| [project/src/rustpg_v4_provenance.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_v4_provenance.rs) | 2673 | 62 | Integral | A15, A16 |
| [project/src/rustpg_v4_wire.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_v4_wire.rs) | 4239 | — | Pendente | — |
| [project/src/rustpg_view.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_view.rs) | 12199 | — | Parcial | B18, B19, B24–B26 |
| [project/src/rustpg_weapon_training.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_weapon_training.rs) | 2818 | — | Pendente | — |
| [project/src/rustpg_world_vfx.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_world_vfx.rs) | 1118 | — | Pendente | — |
| [project/src/rustpg_xp_rules.rs](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/project/src/rustpg_xp_rules.rs) | 2387 | — | Pendente | — |
| [scripts/ai_bootstrap.sh](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/ai_bootstrap.sh) | 2171 | — | Pendente | — |
| [scripts/asset_studio_setup.sh](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/asset_studio_setup.sh) | 1356 | — | Pendente | — |
| [scripts/audit_assets.py](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/audit_assets.py) | 1332 | — | Pendente | — |
| [scripts/audit_skill_trees.py](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/audit_skill_trees.py) | 2253 | — | Pendente | — |
| [scripts/blockbench_mcp.sh](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/blockbench_mcp.sh) | 246 | — | Pendente | — |
| [scripts/build_creature_model.py](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/build_creature_model.py) | 8472 | — | Pendente | — |
| [scripts/generate_ai_context.py](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/generate_ai_context.py) | 8835 | — | Parcial | A21, B33 |
| [scripts/generate_creature_portraits.py](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/generate_creature_portraits.py) | 5345 | — | Pendente | — |
| [scripts/generate_creatures.py](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/generate_creatures.py) | 30855 | — | Pendente | — |
| [scripts/generate_ui_assets.py](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/generate_ui_assets.py) | 8547 | — | Pendente | — |
| [scripts/package_hmod.sh](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/package_hmod.sh) | 234 | — | Pendente | — |
| [scripts/package_native.py](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/package_native.py) | 4357 | — | Pendente | — |
| [scripts/paint_ashen_sentinel.py](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/paint_ashen_sentinel.py) | 2223 | — | Pendente | — |
| [scripts/paint_rift_hunter.py](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/paint_rift_hunter.py) | 2237 | — | Pendente | — |
| [scripts/rustpg_build.sh](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/rustpg_build.sh) | 949 | — | Pendente | — |
| [scripts/rustpg_migrate.py](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/rustpg_migrate.py) | 11182 | — | Pendente | — |
| [scripts/validate.sh](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/scripts/validate.sh) | 2330 | — | Pendente | — |
| [tools/preview4_port.py](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/preview4_port.py) | 5755 | 98 | Integral | A24, B32 |
| [tools/preview4_revamp.py](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/preview4_revamp.py) | 17141 | 251 | Parcial | A03, A04, A06, A22, A24 |
| [tools/sunrise_compat.py](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_compat.py) | 9539 | 119 | Integral | A24 |
| [tools/sunrise_rpg_rework.py](https://github.com/rc2317220-max/Claude-rpg/blob/515457228c1712cac0502dcf3046c43466eef1ee/tools/sunrise_rpg_rework.py) | 18109 | 275 | Parcial | A01–A05, A19, A24 |


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

## Terceira rodada — evidências adicionais na base 82c393f

| Arquivo | Linhas lidas integralmente | Achado / estado | Gravidade | Correção / evidência |
|---|---:|---|---|---|
| `.github/workflows/agent-terminal.yml` | 46 | Permissões read e execução por push | — | Ver registro de commits e checklist; validação pendente |
| `.github/workflows/validate.yml` | 99 | Checkout do publish usa branch, não SHA; A33 parcialmente confirmado | Alta | Ver registro de commits e checklist; validação pendente |
| `project/src/rustpg_powers.rs` | 94 | Recarga vinculada a entidade removida no unload; A27 | Alta | Ver registro de commits e checklist; validação pendente |
| `project/src/rustpg_elite_encounters.rs` | 77 | Normal Vampírico impedido de regenerar; A32 | Média | Ver registro de commits e checklist; validação pendente |
| `project/src/rustpg_evil_att.rs` | 126 | Descrição prometia dreno não implementado | Média | Ver registro de commits e checklist; validação pendente |
| `project/src/rustpg_engine_nutrition.rs` | 64 | Sem ingestão conectada; A28 | Alta | Ver registro de commits e checklist; validação pendente |
| `project/src/rustpg_v4_provenance.rs` | 62 | Overflow global permanente; A15/A16, não corrigido | Alta | Ver registro de commits e checklist; validação pendente |
| `project/src/rustpg_engine_stamina.rs` | 127 | Exaustão com limiar e teste; sem alteração nesta rodada | — | Ver registro de commits e checklist; validação pendente |
| `project/src/rustpg_engine_vitality.rs` | 68 | Domínio testável; ponte nativa em lib.rs | — | Ver registro de commits e checklist; validação pendente |
| `project/src/rustpg_engine_attributes.rs` | 75 | Conversão de Stats e limites; sem alteração nesta rodada | — | Ver registro de commits e checklist; validação pendente |
| `project/src/rustpg_combat_pipeline.rs` | 111 | Pipeline tipado e limites; sem alteração nesta rodada | — | Ver registro de commits e checklist; validação pendente |
| `project/src/rustpg_enchant_stack.rs` | 217 | Bigorna aceita parte do livro e código inseria rejeitados; B15 | Alta | Ver registro de commits e checklist; validação pendente |
| `project/src/rustpg_gear.rs` | 180 | A31, A38: saturação UID e truncamento de bolsa | Alta | Ver registro de commits e checklist; validação pendente |
| `scripts/package_hmod.sh` | 6 | Validação básica do ZIP | — | Ver registro de commits e checklist; validação pendente |
| `scripts/package_native.py` | 77 | Regenera assets e empacota fonte; valida ZIP e manifesto | — | Ver registro de commits e checklist; validação pendente |
| `scripts/rustpg_build.sh` | 27 | SDK opcional e trap de restauração | — | Ver registro de commits e checklist; validação pendente |
| `scripts/validate.sh` | 40 | Gera pacote antes do upload; A33 não é pacote pré-existente; risco separado no checkout do publish | Alta | Ver registro de commits e checklist; validação pendente |
| `tools/sunrise_rpg_rework.py` | 290 | A01/A02/A05 já alterados; ordem, contorno e visibilidade | — | Ver registro de commits e checklist; validação pendente |
| `tools/preview4_revamp.py` | 343 | A36: teste aceitava nenhum nó visível; corrigido | Alta | Ver registro de commits e checklist; validação pendente |
| `tools/preview4_android_lab.sh` | 214 | A37: ANR confirmado em screenshot do artifact 15086420613; laboratório bloqueado antes do jogo | Alta | [bf8e78f](https://github.com/rc2317220-max/Claude-rpg/commit/bf8e78fc0212b05ffe95074f64f6610c8daeb654) |

### Achados novos confirmados (não somar hipóteses como bugs)
| ID | Gravidade | Evidência | Implementação |
|---|---|---|---|
| A36 | Alta | preview4_revamp.py: fit mínimo 0,5; loop de nós visíveis aceitava lista vazia | [877797f](https://github.com/rc2317220-max/Claude-rpg/commit/877797f92b996b49afa75936c189ae618cca78af); câmera padrão ancora primeira raiz se anel excede área; teste não-vazio |
| A37 | Alta | Captura setup-start-latest.png do artifact 15086420613; ANR com.google.android.googlesdksetup | [bf8e78f](https://github.com/rc2317220-max/Claude-rpg/commit/bf8e78fc0212b05ffe95074f64f6610c8daeb654); somente AVD descartável |
| A38 | Alta | rustpg_gear.rs: sanitize truncava bolsas legadas maiores que BAG_LIMIT | [7dddc1c](https://github.com/rc2317220-max/Claude-rpg/commit/7dddc1c1774f5c7f809ab139614fc62310107457); preservar itens válidos, bloquear novos até liberar espaço |

### Correções registradas nesta rodada
- [154c5bf](https://github.com/rc2317220-max/Claude-rpg/commit/154c5bfe1759dd77d4b47749e04d705d8e001a27) — A27 — cooldown de poderes por UUID
- [490986f](https://github.com/rc2317220-max/Claude-rpg/commit/490986fddc6972cbd88683aa71771292c1671578) — A32 — regeneração Vampírico e descrição
- [c54882b](https://github.com/rc2317220-max/Claude-rpg/commit/c54882b42d1e690201c95f985f92cd15aeec21f3) — A28 — nutrição conectada ao evento de fome e efeitos
- [4b2a143](https://github.com/rc2317220-max/Claude-rpg/commit/4b2a143c94581227c7bbf407b79301efab56d869) — A30 — drops de caça e pecuária
- [f21b228](https://github.com/rc2317220-max/Claude-rpg/commit/f21b2281761a985c65689e3a8b0cc25cfaecf2d4) — A29 parcial — dimensão e modo de jogo
- [877797f](https://github.com/rc2317220-max/Claude-rpg/commit/877797f92b996b49afa75936c189ae618cca78af) — A36 — câmera radial vazia
- [bf8e78f](https://github.com/rc2317220-max/Claude-rpg/commit/bf8e78fc0212b05ffe95074f64f6610c8daeb654) — A37 — ANR Google SDK no AVD
- [8c61fba](https://github.com/rc2317220-max/Claude-rpg/commit/8c61fba87873afdeeb65dd5b1193b34aca1ab63c) — B15 — bigorna rejeita encantamento incompatível
- [7dddc1c](https://github.com/rc2317220-max/Claude-rpg/commit/7dddc1c1774f5c7f809ab139614fc62310107457) — A31/A38 — IDs de relíquias e preservação de inventário

Os itens ainda não têm validação completa; não marcar checklist combinado de implementação/validação como encerrado. [Estável](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45752511889), [Preview 4](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45752511849).

## Quarta rodada — evidência e correções (base a8d5e54)

| Arquivo lido integralmente | Linhas da versão lida | Resultado |
|---|---:|---|
| rustpg_gathering_rules.rs | 84 | Classificação de recursos; proteção de blocos recolocáveis |
| rustpg_xp_rules.rs | 45 | Regras de XP separadas dos bônus de saque |
| rustpg_luck.rs | 459 | Bônus de recursos e classificação de proveniência |
| rustpg_v4_provenance.rs | 62 | A15/A35, overflow global legado e formato persistente |
| rustpg_scaling.rs | 439 | A10/A29, cálculo de poder e marcos |
| rustpg_progression.rs | 109 | Progressão e patente consultadas |
| rustpg_guard.rs | 129 | A34, preparação e publicação da ação |
| rustpg_powers.rs | 102 | A34, recarga e estado de poder |
| rustpg_bestiary.rs | 51 | Modelo de descoberta; divergência com listagem externa no menu requer decisão/teste |
| rustpg_nodes.rs | 347 | Catálogo, fórmulas, dependências e integridade |
| rustpg_view.rs | 363 | Formatação de atributos, inventário, equipamento e textos |

| ID | Gravidade | Evidência | Implementação |
|---|---|---|---|
| A39 | Alta | Run 45752511849, artifacts 15087084688 e 15086897708: ANR Pixel Launcher e falha DNS piston-meta.mojang.com | d5e4d72; diagnóstico DNS e tratamento de ANR no AVD descartável |
| A40 | Alta | sunrise_compat.py dependia da antiga condição literal do hook block_drops | 0f5dd45; delimitar hook e contar substituições |
| A41 | Crítica potencial | SDK Preview pinado, changes/attachment-declarations.toml e parts/attachment_types.rs exigem declaração de proprietário para attachments | 69e77c3; registrar os tipos declarados pelo mod; admissão no runtime não confirmada |
| A42 | Média | rustpg_nodes.rs, node_effects.rs, view.rs, inspector.rs e ui_window.rs expunham rank, IDs e frações internas nos textos | b7e57cf; nomes, unidades e compra única; outros textos ainda pendentes |

- [34036ab](https://github.com/rc2317220-max/Claude-rpg/commit/34036ab7028612dc457612f075fcda40cfac3da8) — A15/A35/B14: proveniência persistente, proteção de recursos e migração CRP4→CRP5 (parcial)
- [d5e4d72](https://github.com/rc2317220-max/Claude-rpg/commit/d5e4d720613dbf3036613111c72a5f2f9ba4eb11) — A37/A39: DNS e ANR do laboratório Android
- [0f5dd45](https://github.com/rc2317220-max/Claude-rpg/commit/0f5dd451fa70b053f59c18a348b2ba75995e5c7b) — A40: âncora de block_drops e contagem das conversões
- [69e77c3](https://github.com/rc2317220-max/Claude-rpg/commit/69e77c373912e34ae087b4c97908bb34ea21911e) — A41: declarar attachments exigidos pelo SDK Preview
- [ea9ba2e](https://github.com/rc2317220-max/Claude-rpg/commit/ea9ba2eacb7f9fe8548a0a61fe8a462ca5f903e3) — A10/A29: tempo, poder e marcos de mundo por nível
- [312c6b1](https://github.com/rc2317220-max/Claude-rpg/commit/312c6b18e5c30b0e5d6f280b62c11963d7b38339) — A34: salvar perfil antes de publicar ações e cache
- [b7e57cf](https://github.com/rc2317220-max/Claude-rpg/commit/b7e57cfdbd0030fc54b8f183fd1f1193006625f7) — Textos: nomes e unidades nas habilidades, relíquias e painel de inspeção

CI anterior: 179 testes estável e 185 Preview passaram, mas Android falhou na preparação. - [Estável 45802522385](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45802522385) e [Preview 45802522406](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45802522406): pendentes na última consulta; ambos referem-se a b7e57cfdbd0030fc54b8f183fd1f1193006625f7. Sem afirmar compilação dos commits novos.

Inventário: 34/149; demais arquivos não lidos integralmente. Não considerar a auditoria concluída.


## Quinta rodada — Fendas (base c32507a)

| Arquivo | Linhas lidas | Problema confirmado | Gravidade | Correção |
|---|---:|---|---|---|
| project/src/rustpg_rifts.rs | 233 na base c32507a | Estado de sessão não possuía recibo persistente nem identidade dos inimigos | Alta | Registro de retorno e WaveTracker; testes de codec e máquina de onda |
| project/src/lib.rs | trechos 260–590, 1280–1320, login, 2890–2995, 3200–3260 | A43: spawn parcial aceito; A44: desaparecimento/descarga contado como vitória; A45: abertura sem prazo; A46: limpeza após logout sem registro durável | Alta | Rastrear IDs, confirmar morte pelo evento documentado, retomar spawn, retornar e limpar com registro persistente; prazo técnico para abertura |
| project/src/rustpg_profile.rs | trechos 1–160, codec e testes | A47: recibo de recompensa não persistido junto ao perfil | Crítica | Serial monotônico e recibo atômico com recompensa; teste de reentrada após decode |

SDK Preview: living_drops.rs e living_death_post.rs lidos integralmente. Os quatro adaptadores foram consultados para âncoras relativas ao sistema; a antiga substituição opcional de entities_in não é mais usada nesta função; patch de roster no revamp preservado. Testes novos ainda sem execução confirmada.


### Continuação da quinta rodada

| Arquivo | Linhas lidas | Achado | Gravidade | Tratamento |
|---|---:|---|---|---|
| project/src/rustpg_rifts.rs | 233 na base c32507a | A48: IDs repetidos após morte não devem avançar contador de spawn | Alta | WaveTracker separa IDs vistos e pendentes |
| project/src/lib.rs | parcial | A48: SDK estável e Preview definem spawn_mob como não persistente; mobs podem despawnar sem evento de morte | Alta | marcar persistência via MobRef, com retentativa periódica |
| project/src/lib.rs | parcial | A49: lock durante delete e interpretação de leitura ausente como troca de dimensão | Alta | liberar lock antes de host, distinguir desconhecido, manter backoff |
| project/src/rustpg_hud_geometry.rs | 47 | escala/configuração não finita pode propagar NaN; investigar proteção e colisão com alvo | Média | normalizar valores e comparar geometria efetiva |
| project/src/rustpg_ui_input.rs | 41 | touch_target assume bounds e rect finitos; sem evidência de entrada inválida no runtime | Baixa (risco) | validar limites na origem |
| project/src/rustpg_timeline.rs | 47 | nenhum defeito novo comprovado no uso atual | — | preservar |
| project/data/rustpg/horizon/entity_type/abyss_weaver.json | 10 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/horizon/entity_type/ashen_sentinel.json | 10 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/horizon/entity_type/crystal_wyrm.json | 10 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/horizon/entity_type/dusk_revenant.json | 10 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/horizon/entity_type/eclipse_ogre.json | 10 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/horizon/entity_type/ember_imp.json | 10 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/horizon/entity_type/grave_knight.json | 10 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/horizon/entity_type/mana_hound.json | 10 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/horizon/entity_type/rift_hunter.json | 10 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/horizon/entity_type/rift_mantis.json | 10 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/horizon/entity_type/rune_colossus.json | 10 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/horizon/entity_type/storm_gryphon.json | 10 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/horizon/item/abyss_heart.json | 16 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/horizon/item/hunter_token.json | 16 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/horizon/item/rift_blade.json | 24 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/horizon/item/sentinel_charm.json | 16 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/loot_table/entities/abyss_weaver.json | 14 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |
| project/data/rustpg/loot_table/entities/ashen_sentinel.json | 14 | JSON válido; template/atributos/saque estáticos; sem verificação de runtime | — | preservar até rastrear geradores e SDK |


### Migração de compras
| ID | Evidência | Gravidade | Implementação |
|---|---|---|---|
| A50 | rustpg_profile.rs: decoder limitava rank e retain descartava recibo por diferença de rank ou ID ausente | Crítica | guardar rank bruto, teto só efetivo; reembolso exacto e sem apagar desconhecidos |
| A51 | rustpg_profile.rs: compra registrava quantidade profissional mas não o pool; respec consultava catálogo atual e descartava desconhecidos | Alta | recibo com moedas por pool, versão de codec 8; aceitar v7, preservar origem não identificável |
| A52 | rustpg_view::PAGES não inclui attributes; único hit respec estava nessa página; comando não registrado | Alta | Status com botão/confirmar, comando com confirmação e salvamento antes de atualizar cache |
| A53 | rustpg_profile.rs: reembolso em blocos repetia preços iniciais e podia custar um milhão de iterações por nó | Alta | soma fechada saturante em u128, sem reembolso parcial; teste de equivalência |

### Commits e evidência da quinta rodada

- Fendas: [4085bce](https://github.com/rc2317220-max/Claude-rpg/commit/4085bce788541972a8020bdc9d5bdb3df45079c6), [edb184c](https://github.com/rc2317220-max/Claude-rpg/commit/edb184ce2ba9eac1c6c2ff50c675dbce0d006c0e).
- Perfil/respec: [ec12ccf](https://github.com/rc2317220-max/Claude-rpg/commit/ec12ccf1a0a86af696882e05842780ec7596bf3f), [62d6a07](https://github.com/rc2317220-max/Claude-rpg/commit/62d6a075cd816cad8103e36929c275372cd9e9d7).
- Testes isolados em [45807294477](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45807294477/job/139865946616): 12 aprovados. Último código aguarda validação [estável](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45807569034) e [Preview](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45807569177). Não encerrar testes integrados nem fases até confirmação.
