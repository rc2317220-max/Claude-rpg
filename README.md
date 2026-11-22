# RustPG — Starling 0.9.0

Repositório de trabalho do RustPG 5.1.1-alpha para Starling/Horizon 0.9.0.

## Estrutura

- `project/` — fonte editável do mod 5.0.0 (Rust, JSON, assets, editor embarcado e manifesto).
- `releases/` — snapshots `.hmod`, incluindo a 5.0.0 atual e versões anteriores para rollback.
- `docs/` — relatórios técnicos, auditorias e histórico.
- `patches/` — patches entre versões.
- `tools/` — editor offline para Android.
- `scripts/` — validação e empacotamento local/Codespaces.
- `references/` — manifesto dos JARs/ZIPs externos usados como referência; os binários não são redistribuídos aqui.


## RustPG 5.1.1 — HUD e compilação

- **Nome visível:** RustPG. O identificador técnico `rustpg` continua igual para preservar os saves, registries, traduções e payloads já existentes.
- **Core:** `releases/RustPG-core.hmod` — o gerador remove completamente a tabela Cargo `[features]` e os patches `[[native.*]]`, que são recusados pelo Compile v2.
- **Dev:** `releases/RustPG-dev.hmod` — mantém os 19 mixins para builds integrados do motor. Não deve ser importado no Compile v2.
- **HUD:** HP/stamina 7,7 px na escala 1, XP sem alteração; alvo com barra de HP de 5 px; somente texto de dano/combo quando não houver alvo, sem fundo opaco e sem segunda linha redundante.
- **Testes:** GitHub Actions checa `Cargo.toml`, manifesto, arquivos ZIP e `cargo check` contra o Horizon fixado. A confirmação visual exige rodar Starling no dispositivo.

## Uso no Codespaces

1. Abra este repositório em um Codespace.
2. Trabalhe dentro de `project/`.
3. Execute `./scripts/validate.sh` para checagens estáticas.
4. Execute `./scripts/package_hmod.sh` para gerar `releases/RustPG-dev.hmod`.

## Estado

A 5.0.0 é uma versão Alpha de desenvolvimento. A estrutura e o empacotamento podem ser validados fora do jogo, mas a compilação nativa e o comportamento final precisam ser confirmados no Starling 0.9.0.


## Ajustes em revisão (HUD + Skill Tree)

Veja a [PR #1](https://github.com/rc2317220-max/Claude-rpg/pull/1).
As alterações ainda não estão na branch principal até a revisão ser incorporada.

- HP e stamina: **137,5% da espessura da XP** em escala 1 (9,625 px vs. 7 px), mantendo-as mais curtas (104 px vs. até 216 px).
- XP continua na posição e tamanho originais.
- Detalhes de cada node: rank atual, custo, fórmula aplicada, próximo rank, aumento e requisitos impeditivos.
- Auditoria de nodes importados: ID único, pais existentes, ausência de ciclos, exclusões simétricas.
- CI separa testes do HUD/JSON/skill graphs da **compilação nativa Starling**, que ainda precisa do SDK Horizon.

### No Codespaces

Depois de incorporar a PR, atualize seu terminal com `git pull` e rode
`bash scripts/validate.sh`. Para gerar um arquivo de fontes `.hmod`,
rode `bash scripts/package_hmod.sh`. **Este ZIP não é prova de compilação nativa.**

## RustPG 5.2.1 — revisão a partir de gravação Android

- Inspeção do vídeo de 22/11/2026: HUD de combate com fundo escuro e modelo grande, barra de HP atrasando após golpes, mapa de skill tree com botões/minimapa sobre os nós e zoom excessivamente pequeno.
- Barra de mob: preenchimento primário desenhado a partir da fração autoritativa, não da interpolação atrasada. O indicador de transição não impede a barra de atingir 0.
- Mineração: integridade representada por etapas de quebra 0–9, com estágio inicial 100%, transição limitada a 85 ms e sem extrapolação entre eventos.
- HUD de combate: painel escuro removido, prévia 3D menor e uma única linha de dano/combo.
- Skill tree: controles em faixa própria acima do viewport, mapa sem minimapa e anéis decorativos, zoom útil no intervalo 50–160%, ajuste focado no nó selecionado e zoom da rolagem ancorado no cursor.
- Inspector: largura reduzida em telas amplas para dar mais espaço ao mapa.
- Testes unitários de barras e regiões de interação; validação visual definitiva ainda requer execução no Starling Android.

