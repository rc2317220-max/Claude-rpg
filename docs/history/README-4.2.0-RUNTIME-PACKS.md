# RustPG 4.2.0-rc — datapacks de profissões e progresso

## Implementado

O motor de profissões é um registro de regras ABI 1, isolado do mundo e do host.
Os eventos de gameplay são validados no servidor antes de conceder XP. As regras
**não podem** dar XP geral, inventar drops, alterar perfis diretamente ou superar
64 XP de profissão por evento concluído. Pacotes com erros são ignorados e listados
no comando `/rpg debug`; os dados válidos anteriores não são parcialmente editados.

### Plugins embutidos

- `data/rustpg/plugins/professions.rustpg` contém os gatilhos originais.
- `data/rustpg/plugins/extra.rustpg` permite adicionar gatilhos antes de compilar
  o arquivo .hmod sem mexer em código Rust. Precisa recompilar o mod.

### Plugins por datapack (sem recompilar o núcleo)

Adicionar um arquivo `data/<namespace>/rustpg/professions/<nome>.rustpg` a um
datapack do mundo selecionado. O Starling descobre as regras pelo `ServerReloadListener`.
Executar `/reload` para aplicar a nova geração de regras. Cada arquivo contém:

```
id=seumod:profissoes_extras
api=1
trade|trading|6|1
```

IMPORTANTE: esta regra de exemplo só será aceita se não duplicar outra regra
`(evento,profissão)` dos pacotes habilitados. Por padrão o `trade|trading` já vem
no pacote core, então não deve ser repetido. Um exemplo não duplicado seria:

```
id=seumod:treino_agricola
api=1
craft|farming|1|1
```

O exemplo confere 1 XP de Agricultura por criação concluída; ele é uma prova do
mecanismo, não uma regra de equilíbrio recomendada. Pacotes não são código executável.

Formato `evento|profissão|xp_por_unidade|max_unidades`. Eventos aceitos: craft,
fish, trade, tame, brew, enchant, smith, cook, smelt. A profissão deve ser
uma das 25 do jogo. Uma regra não pode conceder mais de 32 XP por unidade,
e o processamento inteiro não concede mais de 64 XP por evento.

### Profissões com novos eventos reais

Comércio: troca concluída com aldeão; Alquimia: frasco coletado; Encantamento:
encantamento aplicado; Ferreiro: item coletado na bancada de ferraria;
Culinária: alimento reconhecido coletado da fornalha; Ferreiro (metal): lingotes
e netherite scrap reconhecidos; Domador: entidade domada com dono confirmado.
Pesca e artesanato usam os mesmos registros do motor para não duplicar XP.

O Starling também possui eventos de reprodução e saque de contêineres, mas
estes não comprovam sempre o jogador responsável. Não há XP atribuído
arbitrariamente nesses casos. Combate corpo a corpo agora associa o item realmente selecionado no servidor
à criatura atacada e só concede XP ao morrer, dentro da janela de 60 ticks.
Espadas, machados, maças, tridentes, lanças vanilla e mãos vazias são
distinguidos. O tipo de projétil permanece a fonte de verdade quando presente;
a distinção entre arco e besta ainda requer rastrear o disparo do projétil.
Itens desconhecidos de outros mods são tratados como desconhecidos, sem
inventar uma disciplina.

### Limites e validação

- O projeto não inclui carregador de DLLs, WASM ou plug-ins nativos externos.
- Para plug-ins de código será necessário definir ABI, isolamento, registro,
  permissões, compatibilidade e assinaturas para a revisão exata do motor.
- Não há Rust/cargo nem Starling executável neste ambiente. A edição atual foi
  inspecionada, mas **não compilada nem testada no Minecraft**.
