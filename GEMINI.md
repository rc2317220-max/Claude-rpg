# GEMINI.md

Use **AI_AGENT_PROTOCOL.md** como instrução principal deste repositório.

Primeiro diagnóstico:

```bash
python3 tools/ai_doctor.py
```

Preparação automática quando houver terminal/rede:

```bash
bash scripts/ai_bootstrap.sh
source .ai-tools/env.sh
```

Não duplique regras aqui; a arquitetura, os gates de build e o fluxo Horizon/MCP estão centralizados em `AI_AGENT_PROTOCOL.md`.
