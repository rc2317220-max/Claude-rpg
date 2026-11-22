# Claude RPG — Starling 0.9.0

Repositório de trabalho do Claude RPG 5.0.0 para Starling/Horizon 0.9.0.

## Estrutura

- `project/` — fonte editável do mod 5.0.0 (Rust, JSON, assets, editor embarcado e manifesto).
- `releases/` — snapshots `.hmod`, incluindo a 5.0.0 atual e versões anteriores para rollback.
- `docs/` — relatórios técnicos, auditorias e histórico.
- `patches/` — patches entre versões.
- `tools/` — editor offline para Android.
- `scripts/` — validação e empacotamento local/Codespaces.
- `references/` — manifesto dos JARs/ZIPs externos usados como referência; os binários não são redistribuídos aqui.

## Uso no Codespaces

1. Abra este repositório em um Codespace.
2. Trabalhe dentro de `project/`.
3. Execute `./scripts/validate.sh` para checagens estáticas.
4. Execute `./scripts/package_hmod.sh` para gerar `releases/Claude-RPG-dev.hmod`.

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
