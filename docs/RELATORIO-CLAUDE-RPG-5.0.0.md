# Claude RPG 5.0.0 — sistemas avançados (ALPHA)

Base: Claude RPG 4.9.0 e Starling 0.9.0. Esta entrega é **código-fonte em um pacote .hmod**, não um mod compilado nem uma reprodução completa dos 17 mods Java. Os designs de mobs, itens e poderes são originais e não reutilizam modelos, texturas, código Java ou animações protegidas dos mods de referência.

## Novas implementações

1. **Renderer 3D procedural com ossos animados.** `src/crpg_native_entities.rs` define um renderer nativo para `claude_rpg:ashen_sentinel` e `claude_rpg:rift_hunter` por meio de `EntityRenderer`, `RegisterEntityRenderersEvent` e `BlockEntityDrawContext` do SDK 0.9.0. Usa `crpg_skeletal` para amostragem de keyframes do pescoço/braços, poses da caminhada e mudanças de escala/brilho na Sentinela. Geometria de cubos em vez de importação de modelos GeckoLib.
2. **Partículas nativas.** `src/crpg_world_vfx.rs` registra dois providers vanilla e emite bursts limitados em `ClientHost::add_particle` após dano real positivo registrado pelo servidor. O pacote **não** inclui os shaders, pós-processamento, materiais ou o editor do Fancy VFX.
3. **Defesa e hitboxes.** Mantém aparar/esquivar da 4.9, combos e custos de stamina; `src/crpg_hitboxes.rs` acrescenta volume AABB, broad-phase de varredura e validação de alcance antes de conceder bônus de combo. **Não** implementa as animações de ataque 3D, todas as hitboxes ou o sistema de estados do Epic Fight.
4. **Poderes ativos.** `src/crpg_powers.rs` implementa 3 poderes originais (`power.fury`, `power.barrier`, `power.siphon`) desbloqueados pela árvore `powers`, com custo/recarga/duração validados no servidor. Teclas configuráveis nas opções: J, K e L. Ímpeto: +25% dano corpo a corpo por 80 ticks; Égide: -28% dano recebido por 100 ticks; Sifão: cura equivalente a 9% do dano corpo a corpo realmente causado, limitada a 4 HP por golpe, por 100 ticks. Durações temporárias não são salvas entre reinicializações.
5. **Itens personalizados.** Quatro itens novos (Espada da Fenda, Amuleto da Sentinela, Insígnia do Caçador e Coração do Abismo) definidos por JSON nativo com modificadores/raridade, receitas próprias e tabelas de saque dos dois mobs. Não é o catálogo inteiro de Artifacts nem do Remnants.
6. **Criaturas.** Os dois tipos de mob têm JSONs de entidade real, modelos 3D procedurais, atributos, drops, e escalonamento pelo Claude RPG. Herda comportamentos básicos do template zombie.
7. **IA de chefes.** `EliteGoals` registra uma meta de salto por `GoalRegistrar`; `server_tick` faz a Sentinela convocar Caçadores da Fenda abaixo de 50% de vida, com limite de 3 lacaios próximos e cooldown de 300 ticks. Não reproduz todas as fases, golpes, animações, habilidades e equipamentos dos chefes originais.
8. **Editor offline Android.** `editor/CLAUDE_RPG_EDITOR_OFFLINE.html` edita JSONs de missões/skill trees com inspeção de IDs, dependências/ciclos e prévia de grafo arrastável. Também cria scripts `.crpgx` no formato aceito pela VM anterior. Os projetos JSON **não são carregados automaticamente pelo mod** e o editor **não é uma tela integrada ao Minecraft**.

## Uso e testes recomendados

Faça backup do mundo e retire outras versões do Claude RPG. Instale a versão 5.0.0 no Starling 0.9.0, abra Opções > Mods > Compile e confira **todos** os erros emitidos pelo compilador remoto. Se compilar, faça em um mundo temporário:

- Confira Opções > Controles: J / K / L devem aparecer e permitir remapeamento. Abra `/rpg` ou o botão de menu, desbloqueie os 3 nós da árvore Poderes e confirme o uso/recarga e o custo de stamina.
- Com cheats no mundo de testes, use `/summon claude_rpg:ashen_sentinel` e `/summon claude_rpg:rift_hunter`. Verifique aparência 3D, movimento, queda de itens e salvamento/carregamento.
- Diminua a vida da Sentinela para abaixo de 50%, aguarde aproximadamente 10 segundos e confira se surgem Caçadores próximos, sem multiplicação sem limite.
- Dê golpes em criatura e observe o VFX nativo e o combo no HUD. Verifique se ações não confirmadas não concedem bônus/recompensas.
- Abra o arquivo `editor/CLAUDE_RPG_EDITOR_OFFLINE.html` pelo navegador Android para testar o projeto de árvores/missões. Exporte arquivos, valide IDs/dependências/ciclos e execute apenas os scripts `.crpgx` pelo mecanismo que já exista na VM.

## Limitações honestas

- O ambiente de criação **não tem `rustc`, `cargo` ou um runtime Starling**, portanto a compilação nativa e o funcionamento no jogo **não foram testados**. A verificação de ZIP, JSON, TOML, JS e referências dos módulos não substitui compilação.
- Os JARs examinados servem como referências de arquitetura, mas a versão não contém a implementação literal/integral dos 17 mods. Não inclui animações GeckoLib importadas, eventos cinematográficos completos do Remnants, todas as classes de inimigos, modelos originais, todos os artefatos ou interface do editor integrada ao cliente.
- O sistema de pilhas por jogador evoluindo com a skill tree continua pendente de interceptação de todos os caminhos de transferência, arraste, crafting, recipientes e rede. Mudar apenas o tamanho-base global não resolve essa regra.
- A compatibilidade exata do compilador nativo v2 com todas essas assinaturas precisa ser comprovada pelo log de uma compilação real no Starling 0.9.0.

## Arquivos de código relevantes

`src/crpg_native_entities.rs`, `src/crpg_world_vfx.rs`, `src/crpg_powers.rs`, `src/crpg_hitboxes.rs`, alterações em `src/lib.rs`, `src/crpg_actions.rs`, `src/crpg_nodes.rs`, `src/crpg_scaling.rs`, `data/claude_rpg/horizon/*`, `data/claude_rpg/recipe/*`, `data/claude_rpg/loot_table/*`, `assets/claude_rpg/lang/*` e `editor/*`.

Os perfis existentes mantêm o codec anterior, sem alteração de versão de save. Use backup mesmo assim, pois o compilador e a execução no motor ainda não foram validados.
