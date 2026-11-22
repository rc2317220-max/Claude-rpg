# RustPG 4.7.0 — Migração de fonte para Starling 0.9.0

## Base e compatibilidade

- Base de jogo examinada: `starling-0.9.0(2).zip` enviado pelo usuário, tag pública `v0.9.0` / commit `ff303502a4f7982551a0e0d3622d9268efe9e829`.
- `[native] horizon`: atualizado do 0.8 para o commit acima; `mixins`: `e59da9094f0c2fdda6c627fe9eff22dec6a906f0` indicado pelo `mixins_tool.rs` do SDK moderno.
- **Não houve compilação Rust nem teste no Android.** O `.hmod` é fonte para o compilador do launcher; a pinagem *não prova* que a Build v2 remota disponha de um SDK com essa identidade ou aceite esse conjunto completo de hooks.
- Starling 0.9.0 usa dois caminhos nativos: biblioteca compilada por mod (Compile v2) e alterações no próprio jogo (whole-game Compile). `[[native.mixin]]` não foi ativado nesta versão porque a biblioteca por mod não pode aplicar patches à execução do jogo. Essa escolha é deliberada; não anunciar mixins inexistentes.

## Alterações concretas

1. **Manifesto nativo** repinado para commit oficial da versão 0.9.0 e ferramenta de mixins da nova rota; `Cargo.toml` atualizado. Id de mod e attachment `rustpg` preservados para ler perfis antigos.
2. **Registrador `rustpg_starling090.rs`** chama `declare_attribute` uma vez por atributo do catálogo existente, com ID `rustpg:<stat>` (incluindo a keystone `enchanting.transcendence`). São *tipos reais* nativos de atributos, incluindo nós ainda sem efeito, não atributos apenas na interface. Valor-base 0, IDs determinísticos, limites finitos, `syncable` para os nodes com ligação existente.  A declaração de um atributo não torna o seu efeito funcional.
3. **Player attribute map**: `modify_entity_attributes("minecraft:player", ...)` adiciona os atributos a novos jogadores; o Starling 0.9.0 possui a linha do jogador na tabela de atributos. A API registra uma vez no load.
4. **Bounds vanilla** ampliados para vida máxima, armadura, tenacidade e ataque: essa alteração permite que stats altos não sejam truncados pelo teto vanilla, sem tocar em bases de outros mods.
5. **Atributos como modificadores por jogador**: cada bonus conquistado no skill graph / equipamento vai em `LivingRef::replace_attribute_modifier` com ID estável `rustpg:skill_*`. Na remoção/respec, `remove_attribute_modifier` desfaz o bônus. O modificador é transitório, evitamos somas duplicadas e escritas sem alterações.
6. **Cache de instância de jogador** com UUID + dimensão + EntityId: relog/morte/portal faz reaplicação da progressão, sem sobrescrever a base do vanilla. Limpeza periódica e no logout.
7. **Snapshots de dano/mineração** passam a ler valores efetivos do atributo nativo para stats marcadas `applied`, permitindo composição com modificadores de outras fontes. As demais continuam prontas no registro, mas ainda não produzem efeito no jogo.
8. **Telas, HUD, tecla remapeável R, bestiário, quests e assets** foram preservados, assim como os codecs dos perfis existentes; alterações de API são isoladas.

## Problemas ainda abertos

- **Compile 0.9.0**: precisa de log real `Options > Mods > Compile`. Sem compilador `rustc/cargo` e sem SDK publicado em `.horizon/sdk`, não posso afirmar que a versão executa.
- **Skill nodes**: muitos ainda não possuem evento consumindo o atributo nativo. Declaração != hook; nodes com `applied = false` não foram falsamente habilitados.
- **Pilhas individuais por jogador**: o código de capacidade pessoal existe, mas o stack global ÷4 permanece; a lógica transacional de pickup, menus e contêineres exige uma adaptação separada de engine/Mixins que a Compile v2 não executa.
- **Inventário**: não é seguro elevar o `max_stack_size` global sem saber a identidade real do jogador; isso geraria duplicação e inconsistência cliente-servidor.
- **Client input**: o atalho R continua com `register_key_mapping` + `consume_click`. Precisa ser validado com teclado virtual ou layout de toque do Android.

## Como testar

1. Faça backup do mundo e retire a versão 4.6.0 antes da 4.7.0: o ID é o mesmo.
2. Importe o `.hmod` como mod nativo na instância Starling 0.9.0; abra **Opções > Mods > Compile**. Se o compilador mostrar erro de pin/SDK, compartilhe o log — ele identifica o commit de game suportado.
3. Depois de compilar: entre em mundo de teste; rode `/rpg debug`; confirme que `/attribute` do jogador reconhece `rustpg:general.move_speed`; compre um node; confira o valor; use respec e confira a remoção.
4. Valide HP, velocidade, combate, save/load, tecla R, bestiário por abate e itens 64→16 / 16→4. Não teste em mundo principal até passar.
5. O jogo e o mod têm versões próprias: se a Compile v2 pedir outra revisão exata, repine no build que o log indicar; não altere arbitrariamente o commit do mixins.

## Fontes locais do código Starling usados na implementação

- `crates/horizon-mod-vocab/src/parts/{attributes,entity_attributes}.rs`
- `crates/horizon-mod-vocab/src/host/living.rs`
- `crates/horizon-mod-api/src/parts/{attributes,entity_attributes}.rs`
- `docs/modding/src/concepts/native-route/{2-mixins.md,7-how-it-works.md}`
- `crates/horizon-client/src/starling_store.rs` (Compile v1 vs v2)
- `crates/horizon-mod-package/src/mixins_tool.rs`
