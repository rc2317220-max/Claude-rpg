# RustPG 4.6.0-jars-alpha — análise dos JARs e porte parcial para Rust

## Como a análise foi feita

Os quatro JARs fornecidos pelo usuário foram abertos como arquivos ZIP e as classes selecionadas foram desassembladas com `javap -p -c` (bytecode Java). Os métodos abaixo foram realmente encontrados no código compilado. O código Rust em `src/rustpg_java_port.rs` foi escrito a partir da semântica desses métodos, **não** é uma tradução automática de código-fonte original ou uma ponte JVM.

| JAR | Minecraft / mod loader no arquivo | Classes analisadas | Operações observadas |
| --- | --- | --- | --- |
| Puffish Attributes `puffish_attributes-0.8.3-26.3-neoforge` | 26.3, NeoForge | `net.puffish.attributesmod.util.DynamicModificationImpl`; `mixin.LivingEntityMixin`; `mixin.PlayerMixin`; `mixin.AttributeInstanceMixin` | `applyTo(double)`, modificadores com operações `ADD_VALUE`, `ADD_MULTIPLIED_BASE`, `ADD_MULTIPLIED_TOTAL`, atributos dinâmicos em dano, cura, movimento, ferramentas |
| Additional Attributes `additional_attributes-1.21.1-1.2.3` | 1.21.1, NeoForge | `de.cadentem.additional_attributes.registry.AAttributes`; `loot.HarvestLootAmplifier`; `mixin.EnchantmentHelperMixin`; `mixin.FishingHookMixin` | Registro de `FISHING_LURE`, `FISHING_LUCK`, `LOOTING`, `RESPIRATION`, `HARVEST_BONUS`; ajuste de drops de culturas e arredondamento estocástico; substituições de encantamentos |
| More Attributes `more_attributes-1.5.3-all` | 1.20.1, Forge | `com.fu_meng.more_attributes.AttributesRegistry`; `api.AttributeCalculationHelper`; `AttributesHandler`; `ProjectileAttributeHandler`; `LootDropHandler`; `CombatHelper` | 66 campos de atributos, cálculo de chance por `-ln(1-p)`/`1-exp(-x)`, acerto/evasão, penetração, eventos de ataque, sofrimento, projéteis e loot |
| Stackable `Stackable-forge-26.3-2.1.1` | 26.3, Forge | `com.lonkachu.stackable.Config`; mixins `DataComponentCommonFixin`, `ItemStackFixin`, `ItemEntityMixin`, `InventoryStackMixin`, `BundleContentsMixin` | Override de limites/codec, mesclagem de entidades de itens e ocupação de bundles; `InventoryStackMixin.getMaxCountPerStack(int)` converte 99 em `Integer.MAX_VALUE` |

### 1. Puffish Attributes → tipos e modificadores com identidade

`src/rustpg_java_port.rs` introduz `Attribute`, `NamedModifier` e `Operation`; `replace(source, amount, operation)` evita empilhar o mesmo bônus toda vez que o servidor atualiza, e `remove(source)` permite removê-lo ao sair do jogo, trocar equipamento ou perder efeito. A ordem de combinação é distinta das operações vanilla; a rotina `dynamic_apply` representa o padrão de contribuição por origem com sinal positivo ou negativo observado em `DynamicModificationImpl.applyTo`.

A integração direta com `horizon-mod-api` aproveita `LivingEntity::replace_attribute_modifier` nos atributos vanilla de movimento, armadura, tenacidade e velocidade de ataque. Isso não registra automaticamente os 66 atributos do outro mod no registro Starling.

### 2. More Attributes → chances, combate e projéteis

`AttributeCalculationHelper.probabilityToStrength` transforma `p` em `-ln(1-p)`; a volta usa `-expm1(-x)`. Assim, dois bônus independentes de 50% resultam em 75%, não em 100%. Também portamos:

- `getFinalHitProbability`: `clamp(hit_rate*(1-evasion)+final_hit-final_evasion,0,1)`.
- `applyPenetration`: `max(0, reduction_strength*(1-percent_penetration)-flat_penetration)`.
- A base para vida roubada usando dano efetivo à vida, depois de armadura/absorção, restrita à tentativa de ataque corpo a corpo validada.

**Status**: as funções de chance, acerto/evasão e penetração estão no módulo Rust, mas *ainda não são acionadas* pelos eventos adequados. A vida roubada corpo a corpo é lida do nó e enviada à fila de cura no `living_damage_post` — precisa de teste real. Ataques de projéteis ainda exigem snapshot no disparo para não usar a arma errada. `ProjectileAttributeHandler` do Forge depende de interceptações que não existem de forma equivalente no SDK disponível neste ambiente.

### 3. Additional Attributes → drops e arredondamento

`HarvestLootAmplifier.doApply` só modifica drops se existir `BLOCK_STATE` **e** `THIS_ENTITY`, o minerador for `ServerPlayer` e o bloco pertencer a `BlockTags.CROPS`. Os itens precisam estar em `Tags.Items.CROPS`. `AAttributes.getIntValue` preserva a fração probabilisticamente: por exemplo, 2,25 itens significa 25% de chance de virar 3 e 75% de ficar em 2.

`stochastic_count(expected, roll)` foi portado para Rust. **Status:** cálculo independente disponível; o hook atual do RustPG conta eventos de colheita e pesca, mas ainda não reescreve as pilhas reais nos resultados do loot contextualizado. Isso requer adaptar `LootDropsEvent` por tipo de bloco/tag e RNG do servidor; não se deve fabricar drops no cliente nem conceder recompensas pelo mero uso do item.

### 4. Stackable → limite pessoal e invariantes de inventário

O 4.5.0 já tem `rustpg_stack_rules::initial_capacity`: itens de 64 começam em 16, itens de 16 em 4, os de 1 permanecem em 1. O novo `PersonalStacks` calcula uma **capacidade desejada** por jogador baseada em ranks comprados, aumentando 4 por rank para classe 64 e 1 por rank para classe 16 até chegar aos limites originais. Esse é um algoritmo, ainda **não é um hook operacional**.

O código do Stackable torna claro que *alterar apenas `minecraft:max_stack_size` não basta*: precisam ser tratados os limites dos componentes, decodificação/codificação, mesclagem de itens no chão, contêineres, shift-click, drag and drop, crafting, bundles, hoppers e sincronização com o cliente. O Stackable original trabalha com limite global e configuração de exceções, não skill tree individual.

**Contrato para um fork nativo:**

1. Capturar e manter o **limite original** do item (64/16/1) antes de aplicar a redução global ÷4.
2. Em transação de inventário, identificar o proprietário efetivo (jogador / contêiner / sem proprietário). Não usar o jogador que abriu a tela para alterar o limite global de um baú ou hopper.
3. Consultar o perfil sincronizado do servidor para os ranks comprados das duas árvores/linhas de empilhamento.
4. Calcular a capacidade do destino **por item e por proprietário** antes de pickup, merge, split, craft output, trade, drag, shift-click, quick-move, hopper e bundle.
5. Manter validação de soma `antes = depois + remanescente` para impedir perda/duplicação e garantir que o cliente recebe o limite correto; remover modificadores temporários no logout.
6. Se um jogador perder níveis de pilha e possuir pilhas maiores, preservar as quantidades na migração e reparticionar em slots/itens descartados com segurança, sem truncar silenciosamente.

`transfer_amount` recebe o limite **original** do item, não o `max_stack_size` já reduzido; caso contrário, o rank jamais permitiria passar do 16 inicial.

### Alterações efetivas no código do 4.6.0-alpha

- Novo `src/rustpg_java_port.rs`, com modelos matemáticos originais e testes `#[test]`.
- `src/lib.rs`: snapshots de Stats e do último dano sofrido no servidor; integração candidata de bônus de mineração, movimento, armadura, tenacidade, velocidade de ataque corpo a corpo, cura por roubo de vida, regeneração em combate e fora dele.
- `src/rustpg_attributes.rs`: visibilidade declarativa de `mining.speed`, `mining.tool_eff`, `general.move_speed`, `weapon.<tipo>.attack_speed` e `weapon.<tipo>.lifesteal` em classes corpo a corpo.
- `Cargo.toml` / `horizon-mod.toml`: versão `4.6.0-jars-alpha`.

### Limitações rigorosas e validação

**Não foi compilado e não foi testado dentro do Starling.** Este ambiente não tem `rustc`, `cargo` nem a pasta `.horizon/sdk/api/crates/horizon-mod-api` do build exato. A presença de código-fonte no `.hmod` não é prova de que o jogo o executará.

**Não estão completos:** todos os nodes do catálogo; bônus efetivos de loot/colheita/pesca; atributos extras reais do registro nativo; pilhas individualizadas por jogador; comportamento dos projéteis; testes in-game em Android. `mining.tool_eff` neste alpha ainda não diferencia blocos/ferramentas no evento de speed: o multiplicador é aplicado ao break speed recebido. Isso **precisa** ser refinado antes de tratar o nó como eficiência específica de uma ferramenta.

**Testes especificados em Rust (ainda não executados):** operações e origem de modificadores; chance dupla 0.5→0.75; penetração e evasão; arredondamento 2.25; pilhas 64→16→64 e 16→4→16; saneamento de valores inválidos. No Starling, testar principalmente troca de equipamento, PvP e ataques de outras fontes, dimensionamento de pilhas, salvamento/relog e ferramentas modded.

### Entregáveis

- `.hmod` fonte: pacote 4.6.0-alpha para compilação via toolchain Horizon apropriada.
- `src/rustpg_java_port.rs`: biblioteca Rust legível com porta dos algoritmos examinados.
- `ANALISE-JARS-E-PORT-RUST-RUSTPG-RPG.md`: este relatório com vínculos entre classes Java e código Rust.
- `RELATORIO-JARS-BYTECODE-RUST.zip`: trechos de bytecode `javap` das classes inspecionadas e código Rust, para auditoria.
