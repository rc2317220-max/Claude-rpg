# CLAUDE.md — RustPG

Leia **AI_AGENT_PROTOCOL.md** primeiro; este arquivo adiciona apenas o entrypoint específico do Claude.

## Projeto atual

- Mod: **RustPG**. Consulte a versão em `project/horizon-mod.toml`.
- Plataforma: **Starling/Horizon 0.9.0**, game 26.3
- Linguagem: Rust
- Pacote instalável canônico: `releases/RustPG.hmod`
- Horizon fixado: `ff303502a4f7982551a0e0d3622d9268efe9e829`
- Fonte principal: `project/`

## Regra principal

Este projeto usa **somente a API pública estável do Horizon**. Não reintroduza `.hnmod`, whole-game Compile, mixins, bridges privados, patches do engine ou código morto que dependa deles.

Não presuma uma API. Antes de usar uma chamada nova do Horizon, confirme a assinatura no SDK fixado e deixe o `cargo check` provar que compila.

## Como obter contexto sem navegar pela árvore

Se a ferramenta não consegue listar pastas, abra primeiro `AI_START_HERE.md`. Para tarefas de GUI/UI/HUD/Bestiário/Skill Tree, use `AI_UI_SOURCE_BUNDLE.txt`, que contém os arquivos relevantes concatenados em um único texto raw.

Se sua ferramenta não consegue abrir páginas `/tree/` do GitHub:

1. leia `AI_CONTEXT.txt`;
2. leia `SOURCE_INDEX.md`;
3. use os links individuais de arquivos apresentados ali;
4. para UI use `UI_MAP.md`;
5. para combate use `COMBAT_MAP.md`;
6. para Fendas use `RIFTS_MAP.md`;
7. para criaturas use `CREATURES_MAP.md`.

Os mapas são gerados de novo por `scripts/generate_ai_context.py`.

## Invariantes atuais

- V abre a interface do RustPG.
- Não existem missões/dailies.
- Não existe a antiga Block Bar falsa.
- Skill Tree é paginada/fixa; não voltar ao canvas infinito.
- Bestiário tenta renderizar a entidade real e suporta tipos de terceiros quando a API expõe o renderer.
- Fendas usam Ressonância, não chave obrigatória.
- Dungeons usam runtime levels privadas `rustpg:rift/<id>`.
- Combate é autoritativo no servidor; cliente nunca informa que um golpe acertou.
- B = parry; G = dodge.
- Criaturas próprias usam rigs/texturas originais do RustPG.
- Assets do mod de Solo Leveling usado como referência não devem ser copiados.

## Antes de declarar uma versão pronta

Execute:

```bash
bash scripts/validate.sh
cargo check --manifest-path project/Cargo.toml --lib
cargo test --manifest-path project/Cargo.toml --lib
```

No CI, o checkout do Horizon fixado substitui temporariamente o path local do SDK. Uma versão só deve substituir `releases/RustPG.hmod` depois de validação estrutural + cargo check + testes.

## Como editar

Prefira módulos pequenos e integração mínima em `project/src/lib.rs`. Não substitua `lib.rs` inteiro para adicionar uma feature. Preserve Bestiário, UI, attachments e hooks existentes.

Arquivos gerados como `rustpg_creatures_generated.rs` devem ser alterados pelo gerador correspondente, não manualmente.

Ao adicionar um subsistema importante, atualize também o gerador de contexto caso ele precise aparecer nos mapas.


## Ferramentas de build e validação visual

Leia também **`COMO_USAR_FERRAMENTAS.md`** para o passo a passo completo de build, Codespaces, MCP, headless/offscreen e screenshots do jogo real.

- **Build manual na nuvem:** `.github/workflows/build-on-demand.yml`. Dispare pelo GitHub Actions; ele valida, compila contra o Horizon fixado, testa e entrega `RustPG.hmod` como artifact. `publish_latest=true` atualiza o `releases/RustPG.hmod`.
- **Build local/Codespace:** `scripts/rustpg_build.sh`. Com `HORIZON_SOURCE=/caminho/horizon`, também executa cargo check/test contra esse checkout.
- **Jogo real como harness visual:** leia `VISUAL_TESTING.md` e use `tools/horizon_visual_driver.py`.
- **MCP do Horizon:** `.mcp.json` configura `horizon-launcher mcp` para Claude Code. Quando um cliente Horizon preparado estiver disponível, use as ferramentas `game_*` para jogar, tocar/clicar, atacar e capturar screenshots do renderer real.
- Nunca chame uma prévia HTML de "pixel-exact". Só screenshots produzidos pelo Horizon real contam como validação visual exata do cliente.
