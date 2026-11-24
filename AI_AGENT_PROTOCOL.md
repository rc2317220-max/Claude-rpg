# AI_AGENT_PROTOCOL.md — protocolo universal de agentes do RustPG

Este arquivo é a fonte neutra para qualquer agente de IA. Arquivos específicos de produto (CLAUDE.md, AGENTS.md, GEMINI.md e Copilot) devem apontar para cá, não inventar um fluxo paralelo.

## 0. Primeiros 60 segundos

Com terminal:

```bash
python3 tools/ai_doctor.py
bash scripts/ai_bootstrap.sh
source .ai-tools/env.sh
python3 tools/ai_doctor.py
```

Sem terminal, leia nesta ordem:

1. `AI_START_HERE.md`
2. `AI_CONTEXT.txt`
3. `SOURCE_INDEX.md`
4. o mapa do subsistema relevante
5. `AI_UI_SOURCE_BUNDLE.txt` para UI/HUD

## 1. Níveis de capacidade

### Nível R — somente leitura
Pode analisar fontes e produzir patches exatos. Não deve afirmar que compilou, testou ou executou Horizon.

### Nível G — GitHub com escrita, sem shell
Pode editar branches/commits e usar os workflows existentes como executor remoto. Para comandos arbitrários de diagnóstico/build, edite `.agent/run.sh` na sua branch; o push dispara `.github/workflows/agent-terminal.yml`, executa em Ubuntu e publica o log como artifact. O runner tem `contents: read`: a IA continua responsável por fazer alterações via sua integração GitHub. Deve ler os resultados reais do Actions antes de afirmar sucesso.

### Nível T — terminal/rede
Use `scripts/ai_bootstrap.sh`. Depois compile e teste localmente ou use Actions.

### Nível V — Horizon real preparado
Além do Nível T, existe um cliente Horizon compatível e recursos legítimos do Minecraft. Use `tools/horizon_visual_driver.py` em `--offscreen` para screenshots.

## 2. Fonte de verdade

- branch oficial: `main`
- versão atual: leia `project/Cargo.toml` e `project/horizon-mod.toml`
- pacote oficial: `releases/RustPG.hmod`
- espelho humano: `versão atual/RustPG.hmod`
- Horizon pin: `ff303502a4f7982551a0e0d3622d9268efe9e829`

Não trabalhe em uma versão citada por memória ou por bundle antigo sem antes executar `tools/ai_doctor.py` ou conferir o manifesto atual.

## 3. Descoberta de código

Leia `AI_CONTEXT.txt` e depois:

- UI/HUD/Bestiário/Skill Tree: `UI_MAP.md`
- combate: `COMBAT_MAP.md`
- Fendas/dungeons: `RIFTS_MAP.md`
- criaturas/render/modelos: `CREATURES_MAP.md`
- arquitetura geral: `REPO_MAP.md`
- links diretos: `SOURCE_INDEX.md`

Se a IA não consegue navegar pela árvore GitHub, `AI_UI_SOURCE_BUNDLE.txt` contém a fonte UI concatenada. O bundle possui um digest de conteúdo; o doctor detecta quando ele não corresponde às fontes atuais.

## 4. Regras técnicas

- somente API pública estável do Horizon;
- não reintroduzir hnmod, whole-game Compile, mixins privados ou patches ocultos;
- confirmar assinaturas novas no SDK fixado;
- combate permanece autoritativo no servidor;
- Skill Tree permanece paginada/fixa;
- Bestiário prefere renderer real da entidade;
- assets próprios do RustPG; não copiar assets de mods de referência;
- arquivos gerados devem ser alterados pelo gerador correspondente.

## 5. Build obrigatório

No mínimo:

```bash
bash scripts/validate.sh
```

Com Horizon SDK preparado:

```bash
HORIZON_SOURCE=.ai-tools/horizon bash scripts/rustpg_build.sh
```

Ou diretamente:

```bash
cargo check --manifest-path project/Cargo.toml --lib
cargo test --manifest-path project/Cargo.toml --lib
```

Em GitHub sem shell, use `.github/workflows/build-on-demand.yml`.

## 6. Horizon/MCP

O project-level MCP é `.mcp.json`. Ele chama `tools/horizon_mcp_entrypoint.sh`, que escolhe nesta ordem:

1. `horizon-launcher mcp`, se o launcher estiver instalado;
2. `horizon-ctl mcp` no PATH;
3. `.ai-tools/bin/horizon-ctl mcp`, compilado pelo bootstrap.

Isso torna o projeto compatível com qualquer cliente MCP que aceite servidores stdio, sem amarrar o repositório a um único produto de IA.

## 7. Visual real

**Novo laboratório experimental preferencial para trabalho Sunrise:** leia `SUNRISE_PREVIEW4_LAB.md`. Use `.github/workflows/sunrise-preview4-lab.yml` para testar Preview 4, efeitos, dano e custos reais; `tools/creature_inspector.py` para render 3D fora do jogo. A regra Horizon abaixo permanece para a release estável 0.9.0. A referência completa da Preview 4 está em `references/starling/preview-2026.11.23.4/`. Não confundir capturas offline com screenshots reais nem publicar o pacote LAB-ONLY.

Antes de dizer que screenshots não são possíveis:

```bash
python3 tools/ai_doctor.py
```

Se houver `horizon-ctl` + um cliente preparado:

```bash
python3 tools/horizon_visual_driver.py --game "$HORIZON_GAME" --output visual-artifacts
```

Use offscreen, não headless, para pixels reais.

O repositório nunca deve conter Minecraft jars/assets proprietários, tokens Microsoft, cookies ou sessões. Esses itens pertencem ao ambiente privado do usuário.

## 8. Entrega

Uma IA só chama uma alteração de pronta quando informa evidências verificáveis:

- commit/branch usados;
- `validate.sh`;
- `cargo check`;
- `cargo test`;
- pacote gerado;
- screenshots reais somente se Horizon offscreen foi realmente executado;
- limitações específicas restantes.

Nunca invente execução.
