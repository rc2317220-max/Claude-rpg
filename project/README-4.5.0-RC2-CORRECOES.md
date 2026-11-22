# RustPG 4.5.0-rc2 — revisão do pacote fornecido

## Alterações implementadas no código-fonte

- **Bestiário por abate confirmado**: remoção de `first_seen` por proximidade do tick do servidor e filtro `kills > 0` na interface. Entradas antigas criadas ao se aproximar permanecem nos dados salvos, mas não desbloqueiam a ficha sem pelo menos um abate registrado.
- **Maior golpe real**: o callback de dano final, após absorção e armadura, enfileira dano legítimo; o tick do servidor identifica o atacante real pela entidade e dimensão, agrega e atualiza `best_damage` do jogador. O painel informa claramente que se trata do maior golpe causado naquela espécie.
- **HUD**: HP e SP de 132×13 para 104×19 (escaladas conforme HUD), com separação vertical adaptada; XP mantém seu tamanho, aspecto e posição anteriores. Ajustada a área reservada do HUD.
- **Atalho**: remover `shortcut_key` (lista de apenas seis letras) e o `input_key` físico redundante. Abrir com `KeyRegistration("open_system")` + `host.consume_click`, configurável pela captura de próxima tecla nas opções de controles do jogo. `/rpg` continua funcionando como alternativa. O atalho da relíquia também fica exclusivamente no mapeamento nativo.
- **Missões principais**: campanha de oito etapas com contadores persistentes separados das diárias: 16 toras, 1 bancada fabricada, 1 picareta de madeira fabricada, 32 pedras extraídas, 1 picareta de pedra fabricada, 24 minérios naturais extraídos, 10 criaturas derrotadas e 8 lingotes de ferro recolhidos. Recompensas de XP e pontos, com eventos recebidos do servidor (quebra/craft/abate/pickup parcial ou total). Eventos não podem pular etapas.
- **Persistência**: a série de missões suporta registro da versão 3 e importa a versão anterior 2 sem apagar a parte legada. A nova campanha começa do início ao migrar.
- **Skill trees**: mostra todos os nós previamente definidos, inclusive os explicitamente indisponíveis; disposição determinística com silhuetas distintas para mineração, espada, desarmado, pesca, agricultura etc., mantendo conexões dos pré-requisitos.
- **VFX**: texto do Inspector aparece progressivamente ao mudar o item/monstro/nó examinado, com rolagem imediata quando solicitada. Informações da espécie no Inspector ao passar mouse/selecionar, sem repetir ficha inteira no cartão direito.
- **Pilhas reais na origem**: `modify_default_components` altera defaults com tamanhos 64→16 e 16→4, tanto no servidor quanto no cliente, preservando os outros tamanhos. Itens duráveis sem componente explícito mantêm sua classificação de item único.

## Limites honestos desta revisão

1. **Evolução individual do limite de pilhas ainda não foi conectada**. O hook de defaults é global, antes de carregar o mundo, e não oferece um `PlayerRef` para decidir quanto um jogador desbloqueou. Para manter o limite inicial e permitir capacidade individual pela skill tree, é necessária uma modificação do núcleo do Starling que consulte um limite por jogador no `Inventory.add`, clique/arraste/shift-click, crafting, hopper/transferência e outras operações de merge. Não fingimos que o nó está funcional nem concedemos stacks que o motor recusará.
2. **Nós marcados `applied = false` ainda não passaram a ter efeitos no mundo**. Eles estão visíveis e desabilitados corretamente. Implementar todos depende de hooks de combate, inventário, projéteis, durabilidade, pescaria, encantamento e outros, possivelmente com forks do motor. A referência dos JARs Java não se transforma automaticamente em código Rust.
3. Não houve **compilação Rust real, teste de integração ou teste dentro do Starling**: o pacote original não inclui `.horizon/sdk/api/crates/horizon-mod-api`, e o ambiente de edição não possui cargo/rustc nem a revisão fixada do motor. A validação foi estática (manifesto, arquivos, sintaxe delimitadora, presença e integração textual de hooks). O primeiro teste do launcher pode expor diferenças de SDK e exige correção com o erro exato.
4. O hook global de stack pode afetar mods que pressupõem 64/16, e pilhas antigas em saves podem precisar de normalização; **faça backup do mundo** antes de usar. Este RC não foi testado num mundo real.

## Procedimento recomendado no celular

1. Faça backup do mundo e da versão anterior `.hmod`.
2. Coloque **somente uma versão** do mod na pasta `horizon-mods/`; não deixe 4.4 e 4.5 instalados simultaneamente.
3. Inicie o Starling/Horizon. Em `Opções > Controles`, encontre a categoria RustPG e pressione a própria opção de tecla `Abrir sistema`, depois a nova tecla escolhida. No Android, confirme que o botão virtual do launcher envia aquela tecla.
4. Execute `/rpg` caso o atalho não funcione. Confirme o mod na lista de mods nativos; se der erro de compilação, salve o log integral.
5. Teste espécie próxima (não pode desbloquear), criatura morta (deve desbloquear), missão da tora, fabricação, coleta, barras, Inspector e remapeamento.

Referências da API do Horizon:
- https://arismoko.github.io/horizon/api/mod_api/trait.Hooks.html
- https://arismoko.github.io/horizon/api/mod_api/struct.ModifyDefaultComponentsEvent.html
- https://arismoko.github.io/horizon/reference/hook-notes.html
