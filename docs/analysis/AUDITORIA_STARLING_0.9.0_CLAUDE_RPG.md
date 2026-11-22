# Auditoria técnica: Starling 0.9.0 + Claude RPG 4.6.0

Data: 22/11/2026. Base: arquivo enviado `starling-0.9.0(2).zip` (12.917 entradas), `.hmod` Claude RPG 4.6.0 da conversa e documentação pública de horizon-mixins. Este é um estudo de código e contratos; **não é prova de compilação ou execução no Android**.

## 1. Resumo executivo

A abordagem anterior confundiu restrição da interface pública com restrição do motor. A base Starling 0.9.0 contém tanto hooks e registradores de alto nível quanto substituições nativas pontuais por `[[native.mixin]]`. O sistema suporta declaração de atributos próprios, ampliação dos limites de atributos existentes, modificadores por entidade, partes específicas de blocos/itens/entidades, anexos persistentes, hooks de combate, inventário, GUI e entrada. A migração correta deve preservar a autoridade do servidor e sincronizar o cliente.

### Revisões e compatibilidade

- `crates/horizon-mod-package/src/mixins_tool.rs` fixa `e59da9094f0c2fdda6c627fe9eff22dec6a906f0` e aplica `tools/mixins/narrow-reference-rule.patch`.
- O script Android `packaging/android/native_build.sh` contém uma outra revisão (`5a64af938ec86b00d53d16a408aabc0038fb3e57`); não se deve confundi-la automaticamente com a fixação da rota moderna do launcher. Confirmar qual caminho o APK 0.9.0 executa na compilação de um pacote.
- O Claude RPG 4.6.0 traz `[native] horizon = "ac874422ed8f892505ff73d67ae486ecfee7a516"` e `mixins = "ddd9700b21997c95d95ac6ee5ecafe3830cf622a"`, com comentário declarando revisões herdadas da 0.8. Seu manifesto não contém `[[native.mixin]]` nem `[[inject]]`; a versão anterior não utiliza a nova capacidade de mixins.
- Release oficial de 0.9.0 (20/11/2026): mods nativos passam pelo sistema novo do Horizon; o antigo formato `.hnmod` do Starling não roda na 0.9.0. Content mods `.hmod` e pacotes nativos modernos são categorias diferentes de operação, ainda que o manifesto atual use `.hmod`.

## 2. Mecanismos reais presentes no código

| Sistema | Evidência no ZIP | Aplicação Claude RPG |
|---|---|---|
| Tipos de atributos próprios | `docs/modding/src/how-to/content-mods.md` (declare_attribute); `crates/horizon-mod-api/src/parts/attributes.rs` | `crit_chance`, `lifesteal`, `mining_fortune`, `fishing_luck`, `sword_mastery`, etc. |
| Atributos além dos limites vanilla | `Registrar::widen_attribute_bounds` na mesma documentação | HP, velocidade, alcance, dano, armadura com faixas próprias de RPG |
| Instalar atributos em entidades | `Registrar::modify_entity_attributes`; `crates/horizon-client/src/game/modding/native_living_refs.rs` | Jogador, zumbis, chefes personalizados; valores sincronizados |
| Modificadores de atributos reais | `native_living_refs.rs` métodos `living_add_attribute_modifier`, `living_replace_attribute_modifier`, `living_remove_attribute_modifier`, `living_set_attribute_base` | Bônus por node, item, buff, equipamento e status temporário |
| Crítico e dano | `crates/horizon-mod-vocab/src/hooks/critical_hit.rs`, `living_damage_pre.rs`, `living_damage_post.rs` | Acerto crítico, mitigação, armadura e roubo de vida no ponto correto |
| Morte com atribuição do atacante | `crates/horizon-mod-vocab/src/hooks/living_death.rs`; `docs/native-mods.md` | Bestiário liberado por matar, e não por aproximação; objetivos de abates |
| Drops e profissões | `crates/horizon-mod-vocab/src/hooks/block_drops.rs`, `item_fished.rs`, `living_drops.rs`, `item_crafted.rs` | Mineração, pesca, agricultura, coleta, saque, crafting |
| Controle de limites do menu | `crates/horizon-mod-vocab/src/hooks/menu_slot_rules.rs` | Limite por slot do menu (observação: NÃO cobre inventário de jogador/criativo, e restringe o máximo ao limite do item) |
| Coleta de itens | `crates/horizon-world/src/inventory_pickup.rs`, `crates/horizon-world/src/player_inventory.rs` | Limites individuais de pilhas no fluxo de coleta; exige adaptação no chamador/host, pois funções genéricas do crate de baixo não são alvos diretos comuns de mixins |
| Transferência por funil | `crates/horizon-mod-vocab/src/hooks/inventory_move_item.rs` | Impedir transferências inconsistentes em containers |
| CLI e mixins nativos | `docs/modding/src/concepts/native-route/2-mixins.md` | `HEAD`, `RETURN`, `OVERWRITE`, `WRAP_OPERATION`, `MODIFY_ARG`, `MODIFY_VARIABLE`, `MODIFY_CONSTANT` etc. |
| Armazenamento de progresso | `docs/modding/src/concepts/parts.md`; anexos `AttachmentType` usados na versão 4.6 | Persistência de nós, atributos e bestiário, com servidor autoritativo |
| Regras configuráveis por dados | `docs/modding/src/how-to/content-mods.md` (declare_datapack_registry) | Definições de progressão, nodes, quests e inimigos em dados versionados (respeitar restrições do esquema flat da API) |
| Teclas e remapeamento | `docs/modding/src/how-to/native-mods/13-runtime-packages-and-identity.md`; `crates/horizon-mod-vocab/src/host/client_input.rs` | `register_key_mapping` + `consume_click` em vez de leitura fixa de R |
| GUI animada | `crates/horizon-mod-vocab/src/hooks/screen_render_post.rs` | HUD, skills e bestiário, tooltips progressivos e efeitos de impacto |

## 3. Mixins: escolher com precisão

- Preferir `Hooks`, `Registrar`, `Parts` e `Attachments` quando já existe contrato público; são mais robustos entre versões.
- Usar `[[inject]]` quando basta início/fim com dados da API pública.
- Usar `[[native.mixin]]` para interceptar função real e alterar um local, uma constante, uma chamada ou uma expressão; `OVERWRITE` exige exclusividade sobre a função.
- `[[native.bridge]]` para campos/métodos privados; `[[native.module]]` para código comum que usa tipos do motor; `[[native.loop]]` em loops publicados.
- Respeitar restrições documentadas: vários alvos genéricos/async/unsafe/inline(always) não são elegíveis; mixins abaixo do crate do game podem exigir `#[horizon_mod_api_macros::replaceable]` ou alvos em wrappers. `horizon-mixins plan` não prova tipos, borrowing ou compilação.
- A ferramenta nativa é pinada por commit do jogo e, neste ZIP, por patch adicional; nunca usar `main` à força sem validar compatibilidade.

## 4. Projeto de implementação (separação por camadas)

1. **`crpg_stats`**: registro de cada atributo, tipagem e restrições de valor; `AttributeModifier` com ID estável por node/origem, lifecycle e remoção ao reset de árvore.
2. **`crpg_progress`**: árvores individuais (`mining`, `sword`, `unarmed`, `fishing`, `farming`, `crafting`, `exploration`), nós e dependências; cada desbloqueio validado no servidor e gravado como attachment.
3. **`crpg_events`**: adaptadores para `critical_hit`, `living_damage_pre/post`, `living_death`, `block_drops`, `item_crafted`, `item_fished`, encantamentos; cada efeito tem um teste.
4. **`crpg_stack`**: função determinística `stack_limit(player, item)` com base 64→16, 16→4 e rank individual de 0–12; conjunto coerente de hooks para pickup, menus, shift-click, crafting, funis e drops; política de overflow ao perder rank; conservação de itens como invariante.
5. **`crpg_quests`**: objetivos tipados por evento, cadeias principais e diárias independentes; nenhuma progressão por simples abertura de GUI.
6. **`crpg_bestiary`**: informações ocultas até morte válida; contador e maior dano vindos de eventos reais e associados ao atacante correto; separação entre criaturas visualizadas e derrotadas.
7. **`crpg_ui`**: controle nativo remapeável; skill trees com layout grafo independente por profissão; HUD de HP/stamina em dimensões diferentes do XP; animações/client VFX que não alteram lógica server-side.

## 5. Matriz de testes necessária antes de chamar a versão de completa

- API e manifesto: validação de pacote nativo no pin do 0.9.0, `cargo horizon-mod check`, `horizon-mixins plan/apply/check` onde cabível, logs sem erro.
- Atributos: aplicar e remover modificador, persistir em save, trocar equipamento, resetar árvore, sincronizar client, recarregar mundo, limites ampliados.
- Combate: ataque direto, flecha com `causing` e `direct` diferentes, dano nulo, bloqueio, absorção, crítico e crítico forçado, abate por mob, por jogador, queda e `/kill`.
- Pilhas: pickup parcial/completo, shift-click, containers, crafting, drop, morte, itens com max16/64/1, componentes diferentes, login/reload, rank reduzido; para qualquer evento, soma antes = soma depois + consumidos legitimamente.
- Input: clique no botão do remapeamento, pressionar tecla física, teclado Android/controles por toque, tela aberta/fechada, repetição de tecla, `consume_click` uma vez por clique.
- Progressão: requisitos entre nós, compra com pontos, reset, categorias isoladas, missões com objetivos parciais e retomada após reinício.
- UI: escalas de Android, textos longos, clipping, hover ou equivalente por toque, animações sem alterar os dados do servidor.

## 6. Referências Java externas úteis para portar **comportamento**, não executar JARs

- Puffish Skills: https://github.com/pufmat/skillsmod — grafo/árvores configuráveis.
- Project MMO: https://github.com/Caltinor/Project-MMO-2.0 — profissões e XP por atividades.
- FTB Quests: https://github.com/FTBTeam/FTB-Quests — missões/etapas/recompensas e progressão persistente.
- Better Combat: https://github.com/ZsoltMolnarrr/BetterCombat — diferentes armas, combos, hitbox e sincronização.
- Apothic Attributes: https://github.com/Shadows-of-Fire/Apothic-Attributes — atributos e extensões de combate.

Os quatro JARs enviados anteriormente (Puffish Attributes, Additional Attributes, More Attributes, Stackable) continuam sendo referências válidas; código compilado pode ser desassemblado, mas bibliotecas/algoritmos precisam ser reconstruídos com contratos Rust compatíveis. Licenças de código e assets devem ser respeitadas.

## 7. Critério de conclusão

Uma feature só entra na coluna **implementada** quando (a) o hook ou mixin está ligado ao motor, (b) compila na revisão 0.9.0 instalada, (c) passa teste que demonstra o efeito e (d) não introduz regressão de inventário/sincronização. O que for apenas interface, matemática ou API ainda não ligada deve permanecer marcado como protótipo.
