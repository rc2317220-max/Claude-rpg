# AI_HANDOFF_PROMPT.md

Cole este texto em outra IA quando quiser entregar o RustPG para ela:

> Trabalhe diretamente no repositório https://github.com/rc2317220-max/Claude-rpg .
> Não presuma versão, arquitetura ou limitações. Leia primeiro
> `AI_START_HERE.md` e `AI_AGENT_PROTOCOL.md`.
> Se tiver terminal, execute `python3 tools/ai_doctor.py`, depois
> `bash scripts/ai_bootstrap.sh` e `source .ai-tools/env.sh`.
> Se tiver escrita no GitHub mas não tiver terminal, use
> `.agent/run.sh` + `.github/workflows/agent-terminal.yml` como terminal remoto.
> Se só tiver leitura, use `AI_CONTEXT.txt`, `SOURCE_INDEX.md` e,
> para UI, `AI_UI_SOURCE_BUNDLE.txt`.
> Antes de afirmar que algo está pronto, execute ou confirme em Actions:
> `bash scripts/validate.sh`, `cargo check` e `cargo test`.
> Para validação visual, verifique primeiro `python3 tools/ai_doctor.py`;
> quando houver cliente Horizon preparado, use o renderer real offscreen com
> `tools/horizon_visual_driver.py`. Não invente compilação, testes ou screenshots.
