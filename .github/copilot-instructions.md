# GitHub Copilot instructions — RustPG

A fonte de regras do projeto é `AI_AGENT_PROTOCOL.md`.

Antes de sugerir mudanças:
1. leia `AI_CONTEXT.txt`;
2. use `SOURCE_INDEX.md` e o mapa do subsistema;
3. preserve as invariantes do protocolo;
4. não presuma APIs do Horizon;
5. trate `bash scripts/validate.sh`, `cargo check` e `cargo test` como gates;
6. não chame mock/HTML de validação visual real; use Horizon offscreen quando disponível.
