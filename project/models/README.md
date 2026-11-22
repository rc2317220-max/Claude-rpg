# Criaturas — Blockbench no GitHub (sem computador)

O projeto usa **Blockbench MCP headless** fixado no commit
`97a8a80d3a37bf95c5e136d93d2c7da399fff753`. Os modelos são arquivos
`.bbmodel` editáveis; não são imagens de demonstração.

## Sentinela das Cinzas

- Arquivo: `creatures/ashen_sentinel.bbmodel`
- Textura: `textures/ashen_sentinel.png` (64 × 64, PNG)
- Animações: `animation.ashen_sentinel.idle`, `.walk`, `.slam`
- Dados consumidos pelo renderizador nativo:
  `../src/crpg_generated_models.rs`
- Sprite que o renderizador solicita:
  `claude_rpg:entity/ashen_sentinel` da pasta
  `../assets/claude_rpg/textures/entity/ashen_sentinel.png`

O `.bbmodel` é a fonte de verdade. Sempre que um modelo for processado
pelo workflow, o script `scripts/build_creature_model.py` regenera
ossos, cubos, caminhos de parentesco e keyframes Rust, além do PNG do pack.
O `scripts/validate.sh` verifica que os arquivos gerados não foram alterados
manualmente e continuam iguais à fonte.

O renderer nativo utiliza o **idle** e o **walk** para os ossos em tempo real.
O `slam` já existe e passou no validador de animações, mas **a ativação
sincronizada por eventos de ataque ainda é uma etapa pendente**.

## Como abrir no celular Android

1. Acesse o arquivo `.bbmodel` no GitHub e escolha **Raw / Download raw file**.
2. Abra `https://web.blockbench.net/` no Chrome e escolha
   **File > Open Model**; selecione o arquivo baixado.
3. Edite a criatura e exporte/salve o `.bbmodel` normalmente.

## Como modificar pela nuvem

O Codespaces usa `.vscode/mcp.json` para disponibilizar o servidor
`blockbench-headless` a clientes MCP compatíveis. Não é preciso abrir
o Blockbench Desktop ou expor a porta 3000.

Para execuções automatizadas, envie um arquivo JSON em `jobs/` com:

```json
{
  "steps": [
    {
      "tool": "bbmodel_info",
      "args": {"file": "creatures/ashen_sentinel.bbmodel"}
    },
    {
      "tool": "bbmodel_validate",
      "args": {"file": "creatures/ashen_sentinel.bbmodel"}
    }
  ]
}
```

O workflow `blockbench-model-jobs.yml` executa as operações em ordem,
guarda os relatórios em `reports/`, atualiza os modelos e regenera o
código da Sentinela. O MCP local tem como raiz apenas `project/models/`.
Portas MCP públicas sem autenticação **não** são permitidas neste fluxo.

## Limitações honestas

Passar na validação e no `cargo check` não prova execução no Android/Starling.
O renderer atual usa a API nativa de cubos do Horizon, não o runtime GeckoLib
diretamente. Keyframes de ataque e efeitos precisam do sinal autoritativo
do servidor para aparecer no instante correto. O Caçador da Fenda ainda usa
o antigo renderer procedural até receber seu próprio modelo validado.
