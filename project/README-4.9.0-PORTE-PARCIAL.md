# Claude RPG 4.9.0 — porte PARCIAL de referências para Starling 0.9.0

## Estado real

Esta versão **não implementa todos os recursos de todos os 17 mods**. O pacote é código-fonte experimental `.hmod` para compilação pelo Starling, não um binário nativo já compilado. Foi criada a partir do Claude RPG 4.8.0 do usuário, preservando `id = "claude_rpg"`, saves legados, o manifesto e demais módulos. Não há garantias de compilação nem funcionamento em Android até testar na versão 0.9.0.

## Mudanças implementadas no código Rust (não testadas no jogo)

1. **Combate defensivo:** `src/crpg_guard.rs`, `crpg_actions.rs` e `lib.rs` — eventos `defend:parry` e `defend:dodge` enviados por atalhos registrados, validados somente no servidor; custo de stamina, janela, cooldown e redução de um ataque. Aparar não evita projéteis ou dano ambiental. Não há hitboxes/animações 3D de Epic Fight. Os atalhos G/V podem conflitar com comandos de outros mods e devem ser remapeados.
2. **Elites e chefes:** `src/crpg_elite_encounters.rs` — resistência da escala existente, fases definidas por HP atual, bônus de dano ao mudar de fase, mutação berserker e regeneração limitada; integra `living_damage_pre` e amostragem de mobs próximos no servidor. Não recria entidades/AI/arenas do Remnant.
3. **Progressão imersiva:** `src/crpg_immersive.rs`, `crpg_profile.rs` — 13 eventos de profissões, sequência de ações com marcos 5/10/20/50 e XP uma única vez, serializados no mesmo perfil. Evita duplicação de marcos no reload. Contagem por tick de servidor para não duplicar hooks.
4. **Profissões PMMO configuráveis:** novos eventos `mine`, `woodcut`, `harvest`, `kill` nos registros de plugins e VM limitada; `activities_advanced.crpg` premia ações confirmadas. Não recria a configuração universal de permissões de PMMO.
5. **Default Skill Trees + Puffish:** `src/crpg_puffish_graph.rs` derivado dos **dados fornecidos** no ZIP: 85 nós de mineração, 148 de combate, 287 arestas normais e 6 exclusivas, mantidas posições gráficas e ligações de exclusão. Dependências orientadas por BFS para progressão acíclica. O jogador não pode comprar dois nós excludentes. Dados de conversão em `notes/puffish_conversion_metadata.json`. **Atributos mapeados são equivalências propostas, não cópias exatas da semântica da skill original** (por exemplo, Fortune +0.1 foi mapeado para chance de drop duplo; não para encantar uma ferramenta).
6. **Novos atributos com consumo pelos hooks:** `general.melee_damage`, `general.ranged_damage`, `general.resistance_all` e `general.attack_speed` atuam em dano e modificadores; `general.luck_bonus` em `minecraft:luck`. Velocidade de movimento de jogador agora tem teto de +50% para permitir o caminho inteiro da árvore fornecida; a escala de movimento de mobs preserva o teto antigo de +10%. Duas famílias da árvore importada, cura e salto, continuam intencionalmente **bloqueadas**, pois ainda não têm hooks seguros.
7. **Base matemática de animação esquelética:** `src/crpg_skeletal.rs` — amostragem de quadros por osso, laço de animação, interpolação e blending, com testes unitários escritos. **Não renderiza esqueletos nem implementa GeckoLib/Gecko JSON**. O código não é chamado pelo motor até integração futura com entidades/modelos.

## Situação por arquivo (implementação de TODOS os recursos = NÃO)

| Mod de referência | O que existe no Claude RPG 4.9 | O que falta em relação ao mod original |
|---|---|---|
| GeckoLib | Núcleo matemático de bones/frames (sem uso em runtime) | Renderer, modelos geo, controladores, sync e asset pipeline |
| Fancy VFX | VFX 2D e timeline da 4.8 preservados | Partículas e shaders 3D, trilhas, mixins e assets |
| FTB Quests | Campanha DAG de 18 objetivos, recompensas e perfil (4.8) | Equipes, editor completo, todos os tipos de tarefas |
| Project MMO | XP de profissões, 13 eventos, packs editáveis, marcos | Regras dinâmicas completas de itens/blocos/biomas |
| Better Combat | Combos por arma, aparar e esquiva servidor | Colisores, hitboxes, ataques sincronizados com animação |
| Remnants Skills and Powers | Árvores, atributos, poderes de relíquias prévios | Catálogo integral das skills ativas e seus VFX |
| EntityLibrary | Bestiário/Inspector prévios | Biblioteca universal de entidades e loot parser |
| MobHealthBar | Barras de HP do alvo/mobs prévios | Skins completas e editor de HUD |
| Enhanced Boss Bars (resource ZIP) | Barras com fases e divisórias (4.8) | Assets exatos / variantes nativas das texturas |
| Punchy | Combos, defesa e interpolador de bones | Animator completo e animações de braço/câmera |
| Epic Fight | Fases de chefes, combos, aparar/esquiva | OBBCollider, animations, stance states, entire fight engine |
| Artifacts | Relíquias/slots/procs da base antiga | Todos os artefatos, itens e efeitos especiais |
| Default Skill Trees | **233 nós + posições/conexões** convertidos | Regras de Fortune idênticas e hooks cura/salto |
| Puffish Skills | Compra com dependência/exclusão, árvores próprias | Todas as definições/recompensas, editor e datapacks |
| Remnant Mobs | Mutação/nível e resistência/regeneração integrados | Entidades, traits e drops exclusivos |
| Remnant Bosses | Escala, vida real, fases e HUD | Chefes, IA, arenas, invocação, músicas |
| Immersive Level Up | Eventos e sequências, marcos e notificações | HUD, conquistas e demais detectores do original |

**Algoritmos/ideias são reimplementações originais em Rust** a partir da análise dos JARs e dados enviados, não incluem o Java, código descompilado ou assets licenciados.

## Verificação / riscos

- O JSON de referência foi inspecionado estruturalmente: 233 IDs exclusivos, nenhum pai inexistente, nenhum ciclo no grafo orientado e exclusões simétricas.
- Verificados: arquivos de código referenciados no `lib.rs`, TOML válido, ZIP íntegro, dados das árvores presentes.
- Código inclui testes Rust para a defesa, elites, sequências, layout importado e interpolação, **mas NÃO EXECUTADOS**.
- `cargo` e `rustc` não estão instalados neste ambiente. **Nenhuma compilação Rust foi realizada**, nem existe validação dentro do Starling 0.9.0, portanto não chame de estável nem funcional em produção.
- Recomendado: testar apenas em cópia de um mundo, desativando outras versões com ID `claude_rpg`; se o compilador mostrar erros, coletar o log exato. Os saves devem ser copiados antes de testar.
- O port completo de todos os 17 mods implicaria implementar centenas de classes, renderers, protocolos, animações, entidades e interfaces e **não cabe em uma migração parcial**.
