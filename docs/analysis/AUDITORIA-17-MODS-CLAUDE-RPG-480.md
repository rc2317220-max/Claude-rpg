# Claude RPG 4.8.0 — Auditoria de 17 arquivos de referência e integração Rust

**Base analisada:** `starling-0.9.0(2).zip` e o pacote `Claude-RPG-4.7.0-STARLING-0.9.0-MIGRACAO-ALPHA.hmod`, além dos 17 arquivos `.jar`/`.zip` enviados nesta etapa. Análise de classes e métodos compilados com `zipfile` e `javap -p`; não é decompilação completa de todos os métodos. **Os 17 arquivos foram inspecionados, mas seus recursos não foram todos implementados**; o status abaixo distingue cada caso.

## Trabalhos implementados nesta versão (fonte Rust)

1. **Campanha de missões com DAG (inspirada na estrutura de FTB Quests):** `src/crpg_campaign.rs`, 18 objetivos em ramos dependentes, contadores independentes, metas, XP e pontos. `Profile.campaign` usa a mesma serialização de perfil que a recompensa, reduzindo o risco de duas entregas para o mesmo objetivo. O registro usa apenas eventos nativos de quebra real de bloco, criação real, fundição, pesca, morte validada, domar, negociar e encantar. O contador nunca aceita progresso arbitrário enviado pelo cliente. A árvore não permite avançar um pré-requisito e seu filho com um único evento. Interface **Quests** e `/rpg campaign` mostram o progresso; a antiga missão principal de oito etapas e as diárias foram preservadas. Há testes Rust escritos para dependências, recompensa única e serialização; não executados neste ambiente.
2. **Combos autoritativos por arma (referência Better Combat / Epic Fight):** `src/crpg_combo.rs`, estado individual por atacante/dimensão, janela de 30 ticks, reset ao mudar de alvo, máximo de cinco acertos, descarte de eventos repetidos no mesmo tick e expiração. O código é chamado de `living_damage_pre`, com verificação de `attack_entity` recente, dano positivo e fontes que não são projétil/magia/DoT. Node `weapon.<arma>.combo_damage` existe para cada arma como atributo real e fica disponível apenas quando conectado ao cálculo; ganho por sequência limitado em 20%. O primeiro golpe não recebe bônus. **Não** implementa hitboxes, parry, dash, animações de braços ou ataques por fases.
3. **Barras de chefes (referências Enhanced Boss Bars, Remnant Bosses e Mob Health Bar):** `src/crpg_boss_ui.rs`, fases visuais 1–3 baseadas na fração de HP do alvo já sincronizada, divisórias, cor e brilho próprios. Renderização integrada ao cartão de alvo existente, sem textura ou nomes/HP inventados. Não substitui a barra vanilla de chefes em todas as telas.
4. **Animação por keyframes (referências GeckoLib / Punchy / Fancy VFX):** `src/crpg_timeline.rs`, interpolação linear/smoothstep/out-cubic e validação de keys. Ligada aos brilhos dos chefes e aos efeitos de acertos que já são projetados do mundo para a interface. É animação 2D original; **não é** o runtime de ossos/armature do GeckoLib nem partículas volumétricas do Fancy VFX.
5. **Migração segura dos dados:** `crpg_profile.rs` acrescenta somente um campo opcional `campaign=...` aceito pelo codec atual. Os dados legados de levels, atributos, poderes, equipamentos, bestiário, missões e skills permanecem. `horizon-mod.toml` preserva `id=claude_rpg` e os pins usados na 4.7.0 para Starling 0.9.0. A instalação em mundo existente **ainda exige backup**, pois não há teste runtime.

## Inventário técnico — 17 arquivos analisados

| Arquivo | Evidência direta das classes/arquivos | Incorporação no Claude RPG 4.8 | O que NÃO foi portado |
|---|---|---|---|
| `geckolib-forge-1.20.1-4.8.5.jar` | `AnimationController`, `RawAnimation`, `BoneAnimationQueue`; 262 classes | Interpolação por keyframe em timeline original | Skeleton, armature, carregamento `.geo.json`, sync de animações de entidades |
| `fancy-vfx-1.2.6.jar` | `AnimatedParticle`, `WorldRenderHandler`, partículas e mixins de projéteis; 31 classes | Brilhos dirigidos por dano real e timeline de VFX 2D | Shaders, sprites de partículas do mod, trilhas 3D e efeitos globais do mundo |
| `ftb-quests-neoforge-2101.1.39.jar` | `quest/Quest`, `quest/task/Task`, `Chapter`, dependências e sistema de recompensas; 501 classes | Campanha nova com 18 tarefas, requisitos e persistência | Editor visual de quests, tarefas arbitrárias, sincronização de equipes, redes completas |
| `pmmo-26.1.2-2.8.36.jar` | `SkillData`, `EventTriggerRegistry`, `MobAttributeHandler`, classes de XP/perks; 355 classes | Usa o motor de profissão e antiabuso já existente, ligado às missões extras | Regras configuráveis para todo item, bloco/bioma/dimensão, equipes e dezenas de perks |
| `bettercombat-fabric-3.2.2+26.3.jar` | `api/ComboState`, `AttackHand`, `WeaponAttributes`, classes de animação e partículas; 165 classes | Combo servidor por arma, node exclusivo | Golpes varridos, combos multi-hit por animação, raycast/collision e animações sincronizadas |
| `remnants-skills-and-powers-neoforge-26.3-2.0.2.jar` | `AbilityRunner.cast/tickPlayer`, `PowerProfile`, `SkillTree`, `AbilityCatalog`; 89 classes | Usa a infraestrutura de relíquias e árvores preexistente | Magias de fogo/terra/água, targeting avançado e efeitos específicos do mod |
| `entitylibrary-neoforge-1.21.1-0.4.0.jar` | `LootTableParser`, `SummonCommandBuilder`, `CompatibleScreen`; 24 classes | Inspector/bestiário nativo já existente | Interface de biblioteca completa e parser universal de loot de mods Java |
| `mobhealthbar-fabric-26.4-2.3.4.jar` | `HpBarChooser`, widgets de seleção/configuração; 34 classes | HUD do alvo com vida real e fases, integrado ao HUD existente | Todas as variantes de skins e editor de posição do próprio mod |
| `[1.6] Enhanced Boss Bars(2).zip` | Resource pack, `assets/minecraft/font/default.json`, `textures/gui/*.png`; **não contém bytecode** | Estilo original de barra com divisórias e fase | Texturas originais, fontes e mapeamentos de boss bars para mods externos |
| `punchy-3.2b-forge-1.20.1.jar` | `AnimationStateMachine`, `AnimationIntentCoordinator`, controle de estados e menus; 1229 classes | Timeline e transições visuais simplificadas | Transformação de braços/itens, animações Bedrock, controladores de câmera |
| `epic-fight-21.17.3.1-mc1.21.1-neoforge.jar` | `AttackAnimation` (phases), `AnimationPlayer`, `OBBCollider`; 1530 classes | Janela real de combo; estilo phased boss HUD | Sistema de luta completo, stamina de habilidades, colisores OBB, stun, poses |
| `artifacts-neoforge-15.1.3.jar` | `EquipmentAbility`, `AbilityTicker`, `DynamicAttributeModifier`, `PostDamageEffect`; 395 classes | Usa sistema preexistente de seis famílias de relíquia e modificadores nativos | Todos os artefatos/efeitos/itens do mod, efeitos de status por slot |
| `default_skill_trees-1.1-1.20.4.zip` | Datapack com `categories/{mining,combat}/{definitions,skills,connections,experience}.json` | Referência para dependências e ramos; nós nativos já permanecem separados | Conversor completo de configuração Puffish ou cópia direta das árvores |
| `puffish_skills-0.19.2-26.1-fabric.jar` | `SkillConnection`, `SkillDefinitionConfig`, `AttributeReward`, `ServerEventReceiver`; 333 classes | Mantém árvores/atributos próprios e novas skills de combo | Toda sintaxe de datapack/importador e todos os tipos de reward |
| `remnant_mobs-neoforge-26.3-3.2.0.jar` | `MobLeveler.firstTick`, `MobLeveler.roll`, `TraitEffects`; 121 classes | Mantém mob scaling persistente e detecção de elites em quests | Entidades próprias, AI, traits e drops exatos do mod |
| `remnant_bosses-neoforge-26.3-3.1.1.jar` | `BossHunt`, `BossHunts`, `RemnantBossBars`, `RealmEvents`; 253 classes | Missões de caça de chefes + barras de fases | Quatro chefes próprios, arenas, animações, música, boss AI e invocação |
| `immersivelevelup-fabric-1.20.1-0.2.0.jar` | `Detectors`, `Combat`, `TitleHud`, memória de streaks; 50 classes | Aproveita notificações/XP existentes e VFX de interface | Tutoriais, streaks e cenários completos do mod |

Observação: **JAR não equivale a código-fonte**. `javap` recupera assinaturas de métodos e bytecode (com `-c`), mas nomes/classes auxiliares, dependências e mixins exigem inspeção adicional para recriação completa. Os dados de outros mods não foram incorporados literalmente ao pacote.

## Mapeamento com o Starling 0.9.0

- `Hooks::player_destroyed_block` → toras, pedra, minérios naturais e plantações adultas.
- `Hooks::item_crafted`, `item_smelted`, `item_fished` → fabricação, fundição e pesca confirmadas.
- `Hooks::living_drops` → abate atribuído ao jogador, elite/chefe/lendário por tiers persistentes.
- `Hooks::entity_tame`, `player_trade`, `player_enchant_item` → tarefas especializadas.
- `Hooks::attack_entity` + `living_damage_pre` → combos ligados ao evento de ataque reconhecido pelo servidor.
- `Hooks::gui_render_post` → barras de chefes e efeitos de acertos existentes; nenhum recurso depende de mod Java rodando.
- `Profile` (attachment persistente e sincronizado) → progresso de quests e recompensa no mesmo registro de dados.
- Atributos novos via `crpg_starling090::register` e `crpg_nodes` → combo separado para cada arma, em vez de bônus de todo o conjunto.

## Validação realizada e pendências reais

- **Realizado:** inspeção de todas as entradas ZIP/JAR, classes principais via `javap`, comparação com os hooks do fonte 0.9.0, verificações de TOML, referências de módulos, DAG de 18 missões, consistência do pacote e CRC do ZIP.
- **Testes adicionados, mas NÃO executados:** testes Rust unitários para missão, salvamento e retomada, combo, fase de chefes e interpolação keyframe.
- **Não realizado:** compilação `cargo`/`rustc` (ferramentas não instaladas), compilação em Starling Android, execução multijogador, verificação da API remota ou teste de todos os mods equivalentes.
- **Continuam pendentes:** inventário de pilhas por jogador em todas as transações, regras completas de cada node do catálogo, animações 3D de esqueletos, partículas customizadas, novos mobs/bosses e suas IAs, combate com hitboxes/parry/dodge, editor Puffish/FTB completo e itens/artefatos novos. Essas áreas precisam de código adicional e testes no motor; não foram marcadas como concluídas.

**Recomendação de instalação:** teste a 4.8.0 numa cópia do mundo e retire a outra versão Claude RPG com o mesmo ID. No menu do Starling 0.9.0, importe o `.hmod` e execute **Compile**. Se houver erro, o log do compilador é necessário para corrigir a incompatibilidade exata.
