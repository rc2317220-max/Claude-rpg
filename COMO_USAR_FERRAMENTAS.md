# COMO_USAR_FERRAMENTAS.md — build, IA e teste visual do RustPG

Este é o guia de operação das ferramentas do repositório. Leia este arquivo quando a tarefa for **compilar**, **entregar um .hmod**, **usar Claude/Codex no projeto**, **testar a interface**, ou **rodar o Horizon como se a IA estivesse jogando**.

## Bootstrap universal para qualquer IA

Com terminal e rede, o fluxo padrão agora é:

```bash
python3 tools/ai_doctor.py
bash scripts/ai_bootstrap.sh
source .ai-tools/env.sh
python3 tools/ai_doctor.py
```

O bootstrap faz checkout do Horizon no commit fixado, compila um `horizon-ctl` local e grava tudo em `.ai-tools/` (ignorado pelo Git). Em Codespaces isso roda automaticamente pelo devcontainer.

A fonte neutra de instruções é `AI_AGENT_PROTOCOL.md`. `CLAUDE.md`, `AGENTS.md`, `GEMINI.md` e as instruções do Copilot apontam para ela.

## Visão rápida

| Quero fazer... | Use |
| --- | --- |
| Entender o projeto | `CLAUDE.md` + `AI_CONTEXT.txt` |
| Encontrar um arquivo sem navegar por pastas | `SOURCE_INDEX.md` |
| Trabalhar na UI/HUD | `UI_MAP.md` |
| Trabalhar no combate | `COMBAT_MAP.md` |
| Trabalhar nas Fendas/dungeons | `RIFTS_MAP.md` |
| Trabalhar nas criaturas | `CREATURES_MAP.md` |
| Compilar/testar no GitHub | `.github/workflows/build-on-demand.yml` |
| Compilar em terminal/Codespace | `scripts/rustpg_build.sh` |
| Conectar uma IA via MCP ao Horizon | `.mcp.json` + `tools/horizon_mcp_entrypoint.sh` |
| Rodar um teste visual do jogo real | `tools/horizon_visual_driver.py` |
| Entender o teste visual | `VISUAL_TESTING.md` |
| Preparar um Codespace | `.devcontainer/devcontainer.json` |

---

# 1. Só quero o RustPG.hmod pronto

O jeito mais simples é usar o GitHub Actions.

1. Abra a aba **Actions** do repositório.
2. Escolha **RustPG build on demand**.
3. Toque em **Run workflow**.
4. Escolha a branch usada no desenvolvimento.
5. Deixe `publish_latest = false` quando quiser apenas testar.
6. Use `publish_latest = true` somente quando esse build deve virar a versão oficial de `releases/RustPG.hmod`.
7. Aguarde todos os passos passarem.
8. Abra a execução concluída e baixe o artifact `RustPG-hmod-...`.

Esse workflow:

- executa `scripts/validate.sh`;
- gera os arquivos das criaturas;
- valida o pacote;
- faz checkout do Horizon fixado;
- executa `cargo check`;
- executa `cargo test`;
- gera o `.hmod`;
- entrega o arquivo como artifact.

Se `publish_latest=true`, depois dos testes ele publica o pacote validado em:

```
releases/RustPG.hmod
```

Nunca publique um pacote que falhou em `cargo check` ou nos testes.

## Pela linha de comando com GitHub CLI

Em um Codespace ou terminal autenticado:

```bash
gh workflow run build-on-demand.yml \
  --ref fix/remnants-skill-graph-5-1 \
  -f publish_latest=false
```

Veja as execuções:

```bash
gh run list --workflow build-on-demand.yml
```

Acompanhe uma execução:

```bash
gh run watch RUN_ID
```

Baixe todos os artifacts da execução:

```bash
mkdir -p build-artifacts
gh run download RUN_ID --dir build-artifacts
```

---

# 2. Compilar dentro de um Codespace

O repositório possui:

```
.devcontainer/devcontainer.json
```

Ele prepara um ambiente com Rust, Python, GitHub CLI e ferramentas Mesa/Vulkan úteis para testes gráficos em nuvem.

## Abrindo

No GitHub:

1. **Code**.
2. **Codespaces**.
3. **Create codespace** na branch atual.

Quando o container terminar, valide o projeto:

```bash
bash scripts/validate.sh
```

Para usar o helper do RustPG:

```bash
bash scripts/rustpg_build.sh
```

Esse comando gera o pacote em:

```
releases/RustPG.hmod
```

## Com um checkout do Horizon disponível

Se o Horizon estiver, por exemplo, em `/workspaces/horizon`:

```bash
HORIZON_SOURCE=/workspaces/horizon bash scripts/rustpg_build.sh
```

Nesse modo o script também executa `cargo check` e `cargo test` contra aquele checkout.

---

# 3. Como uma IA entende o repositório

Uma IA deve começar por:

```
CLAUDE.md
AI_CONTEXT.txt
```

Se ela não conseguir abrir páginas `/tree/` do GitHub, não precisa pedir caminhos manualmente. Use:

```
SOURCE_INDEX.md
```

Esse arquivo contém links diretos para os módulos Rust.

Depois escolha o mapa certo:

- `UI_MAP.md`: HUD, GUI, Bestiário, Skill Tree e widgets.
- `COMBAT_MAP.md`: dano, combos, stamina, parry, dodge e hitboxes.
- `RIFTS_MAP.md`: portais, Ressonância, salas, waves e bosses.
- `CREATURES_MAP.md`: rigs, texturas, renderers, HP e scaling.
- `REPO_MAP.md`: visão geral da arquitetura.

Os mapas são regenerados por:

```bash
python3 scripts/generate_ai_context.py
```

---

# 4. Conectando qualquer cliente MCP ao Horizon

O repositório contém:

```
.mcp.json
```

Ele chama `tools/horizon_mcp_entrypoint.sh`, que seleciona automaticamente:

1. `horizon-launcher mcp`, quando o launcher completo existe;
2. `horizon-ctl mcp` no PATH;
3. `.ai-tools/bin/horizon-ctl mcp`, compilado pelo bootstrap.

Assim o mesmo projeto funciona com Claude Code, Codex, Gemini ou outro cliente que suporte MCP stdio.

## Fluxo

1. Rode `python3 tools/ai_doctor.py`.
2. Se não houver Horizon CLI, rode `bash scripts/ai_bootstrap.sh`.
3. Reinicie/reabra o cliente MCP depois do bootstrap.
4. Confirme que o servidor `horizon` foi reconhecido.
5. Quando um jogo Horizon estiver sob Agent Control, as ferramentas de jogo ficam disponíveis pelo MCP.

A IA pode então executar ações equivalentes às de um jogador, como:

- abrir/criar mundo;
- pressionar teclas;
- enviar toque/mouse;
- mover o jogador;
- olhar para entidades;
- atacar;
- executar comandos;
- ler entidades próximas;
- inspecionar widgets da tela;
- tirar screenshots.

Isso é muito mais útil que uma prévia HTML.

---

# 5. “Emular” o RustPG — o que significa aqui

Existem três níveis diferentes.

## Nível A — testes sem renderer

É o CI normal:

```bash
bash scripts/validate.sh
cargo check --manifest-path project/Cargo.toml --lib
cargo test --manifest-path project/Cargo.toml --lib
```

Isso prova estrutura, compilação e lógica testável.

**Não mostra pixels.**

## Nível B — Horizon headless

O Horizon possui modo `--headless`.

Esse modo:

- roda o jogo;
- roda ticks;
- não abre janela;
- não possui renderer;
- não usa GPU para desenhar a cena.

É útil para lógica e automação, mas **não pode produzir uma screenshot real da UI**, porque não há renderer.

Portanto, não use headless para validar HUD.

## Nível C — Horizon offscreen

Este é o modo que queremos para validação visual.

`--offscreen`:

- executa o Horizon real;
- mantém o renderer real;
- não precisa mostrar uma janela na tela;
- permite screenshot;
- permite Agent Control;
- permite uma IA interagir com o jogo.

Isso é o mais próximo de “a IA jogar o mod e enxergar exatamente o resultado”.

Não é uma imitação do jogo: **é o próprio Horizon renderizando o RustPG**.

---

# 6. Preparação necessária para o teste visual real

Uma máquina, Codespace ou runner privado precisa ter:

1. Horizon Launcher / `horizon-ctl`;
2. os recursos legítimos do Minecraft preparados para aquela instalação;
3. uma instalação/instância Horizon compatível;
4. um build nativo do Horizon contendo o RustPG.

Não coloque no GitHub:

- arquivos oficiais do Minecraft;
- tokens Microsoft;
- cookies;
- sessões de login;
- senhas;
- arquivos de autenticação.

## Compilando o RustPG dentro do Horizon

Com uma instalação normal do Horizon preparada:

```bash
horizon-launcher native-build project \
  --instance rustpg-visual \
  --trust-native-code
```

O Horizon gera/seleciona um cliente nativo que contém o código Rust do mod.

O executável derivado precisa permanecer junto do pacote nativo correspondente em:

```
horizon-native-mods/rustpg/
```

---

# 7. Teste visual automático

O script é:

```
tools/horizon_visual_driver.py
```

Ele conversa com `horizon-ctl`.

## Caso 1 — já existe um Horizon em execução com Agent Control

```bash
python3 tools/horizon_visual_driver.py \
  --attach \
  --output visual-artifacts
```

## Caso 2 — iniciar um executável Horizon nativo que contém RustPG

```bash
python3 tools/horizon_visual_driver.py \
  --game /caminho/para/horizon-client \
  --game-dir .visual-game \
  --output visual-artifacts
```

O script usa o modo offscreen.

## O que ele tenta fazer

A sequência padrão é:

1. entrar em um mundo;
2. esperar chunks/carregamento;
3. tirar screenshot do mundo;
4. pressionar **V** usando a entrada real do Horizon;
5. capturar a interface RustPG;
6. salvar a lista de widgets que o Horizon expõe;
7. voltar para o mundo;
8. invocar uma criatura RustPG;
9. olhar para ela;
10. capturar o Target HUD;
11. atacar;
12. capturar o HUD depois do dano;
13. salvar logs, eventos, mensagens e estado.

Arquivos esperados:

```
visual-artifacts/
  00-world.png
  01-rustpg-menu.png
  01-rustpg-ui-widgets.json
  02-target-hud.png
  02-entities.json
  03-after-attack.png
  visual-report.json
```

Nem todo arquivo aparecerá se o estado necessário não puder ser criado; `visual-report.json` explica o que ocorreu.

---

# 8. Como a IA usa screenshots para corrigir a UI

Um bom ciclo automático é:

```
editar código
     ↓
compilar
     ↓
abrir Horizon offscreen
     ↓
pressionar V / criar cena
     ↓
screenshot
     ↓
analisar layout
     ↓
editar novamente
```

Para UI, compare principalmente:

- posição;
- tamanho;
- padding;
- alinhamento;
- clipping;
- textos cortados;
- barra de HP;
- transparência;
- miniatura do mob;
- Bestiário;
- Skill Tree;
- touch targets;
- resolução/aspect ratio.

Faça capturas em mais de uma resolução quando possível, principalmente algo próximo de celular vertical e horizontal.

---

# 9. O que o teste em nuvem NÃO substitui

Mesmo com Horizon offscreen real, ainda há diferenças que podem aparecer no Android:

- driver gráfico do Galaxy/POCO;
- resolução e DPI reais;
- proporção da tela;
- teclado/controles sobrepostos pelo launcher;
- comportamento específico de toque;
- desempenho térmico;
- GPU do aparelho.

Portanto:

**CI + offscreen = validação técnica e visual muito forte.**

**Android real = validação final do dispositivo.**

Não diga que algo foi “100% testado no Android” quando só passou no runner Linux.

---

# 10. Testando diferentes telas do RustPG

O Horizon Agent Control aceita teclado, mouse, touch e inspeção de widgets.

A IA deve preferir:

- tecla real para atalhos, como `V`;
- `ui` para saber quais widgets existem;
- clique por ID quando o widget tem ID;
- touch/click por coordenadas somente quando necessário;
- screenshot depois de cada mudança de estado importante.

Para cenas de combate:

- spawn de criatura RustPG;
- leitura de `entities`;
- `look-at-entity`;
- `attack`;
- screenshot.

Para Fendas:

- acumular/ajustar o estado necessário em um mundo de teste;
- executar os comandos normais do RustPG;
- atravessar a Fenda;
- capturar sala, mobs, boss bar e saída.

Nunca altere a lógica de produção apenas para facilitar a screenshot. Se for necessário um modo de teste, ele deve ser claramente isolado como tooling/dev.

---

# 11. Qual ferramenta uma IA deve escolher

## “Corrija este erro de compilação”

Use:

```
scripts/rustpg_build.sh
cargo check
cargo test
```

## “Me mande o .hmod”

Use:

```
RustPG build on demand
```

Depois entregue o artifact ou publique `releases/RustPG.hmod`.

## “Melhore a Skill Tree”

Leia:

```
CLAUDE.md
UI_MAP.md
SOURCE_INDEX.md
```

Depois edite, compile e rode o visual driver offscreen.

## “Veja se o HUD realmente ficou bom”

Não responda apenas olhando o Rust.

Use:

```
tools/horizon_visual_driver.py
```

e analise a screenshot do **renderer real**.

## “Teste lógica sem precisar de pixels”

Pode usar headless/CI.

## “Teste visual”

Use offscreen.

---

# 12. Diagnóstico

## `horizon-launcher: command not found`

O Horizon Launcher não está instalado ou não está no `PATH`.

## Claude Code não mostra Horizon em `/mcp`

Verifique:

```bash
horizon-launcher mcp
```

e confira se o Claude Code abriu a raiz do repositório onde está `.mcp.json`.

## Screenshot falha em headless

Isso é esperado. Headless não tem renderer.

Troque para offscreen.

## `horizon_visual_driver.py` não acha jogo

Use `--game /caminho/exato/do/horizon-client` ou `--attach` em um Horizon que já esteja com Agent Control ativo.

## O mod compila, mas o jogo não inicia

Veja:

- logs do Horizon;
- compatibilidade da versão;
- pacote em `horizon-native-mods/rustpg/`;
- recursos do Minecraft preparados;
- se o cliente nativo foi realmente construído com a revisão atual do RustPG.

## O build on demand passou, mas não mudou `releases/RustPG.hmod`

Isso ocorre quando `publish_latest=false`.

É intencional.

---

# 13. Regra para agentes de IA

Quando uma IA tiver acesso ao terminal/Horizon, ela não deve dizer “não consigo testar visualmente” antes de verificar:

1. se `horizon-launcher` existe;
2. se o MCP do Horizon está conectado;
3. se há cliente preparado;
4. se pode usar `--offscreen`;
5. se `tools/horizon_visual_driver.py` pode ser executado.

Se essas condições não existirem, ela deve informar **qual dependência está faltando**, em vez de fingir que executou o jogo.

Quando existirem, ela deve usar o Horizon real e guardar screenshots/evidências da execução.

---

# 14. Arquivos que trabalham juntos

```
CLAUDE.md
   ↓
COMO_USAR_FERRAMENTAS.md
   ├── build-on-demand.yml ──→ .hmod
   ├── rustpg_build.sh ──────→ build local
   ├── .mcp.json ────────────→ Horizon MCP
   ├── horizon_visual_driver.py
   │       ↓
   │   Horizon offscreen
   │       ↓
   │   PNG + JSON + logs
   └── VISUAL_TESTING.md
```

Para código e arquitetura:

```
AI_CONTEXT.txt
SOURCE_INDEX.md
REPO_MAP.md
UI_MAP.md
COMBAT_MAP.md
RIFTS_MAP.md
CREATURES_MAP.md
```

Esse conjunto existe para que Claude, Codex ou outra IA consiga trabalhar no RustPG sem depender de alguém indicar arquivos manualmente.
