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
