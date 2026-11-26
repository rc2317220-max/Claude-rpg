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