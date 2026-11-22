# AI_START_HERE.md

Universal entry point for Claude, ChatGPT/Codex, Gemini, Copilot, local agents and read-only web AIs.

## If you have a terminal

Run:

```bash
python3 tools/ai_doctor.py
bash scripts/ai_bootstrap.sh
source .ai-tools/env.sh
python3 tools/ai_doctor.py
```

Then read **AI_AGENT_PROTOCOL.md**. It is the canonical cross-agent operating contract.

## If you do not have a terminal

Read these raw files:

- https://raw.githubusercontent.com/rc2317220-max/Claude-rpg/main/AI_AGENT_PROTOCOL.md
- https://raw.githubusercontent.com/rc2317220-max/Claude-rpg/main/AI_CONTEXT.txt
- https://raw.githubusercontent.com/rc2317220-max/Claude-rpg/main/SOURCE_INDEX.md
- https://raw.githubusercontent.com/rc2317220-max/Claude-rpg/main/COMO_USAR_FERRAMENTAS.md
- https://raw.githubusercontent.com/rc2317220-max/Claude-rpg/main/VISUAL_TESTING.md

For UI/HUD/Bestiary/Skill Tree:

https://raw.githubusercontent.com/rc2317220-max/Claude-rpg/main/AI_UI_SOURCE_BUNDLE.txt

The bundle now includes a `Source-SHA256`. A terminal agent can run
`python3 tools/ai_doctor.py` to verify that the concatenated source still matches it.

## Product-specific instruction files

- Claude: `CLAUDE.md`
- OpenAI/Codex: `AGENTS.md`
- Gemini CLI: `GEMINI.md`
- GitHub Copilot: `.github/copilot-instructions.md`

They all defer to `AI_AGENT_PROTOCOL.md`; there is one source of truth.

## GitHub-only agents

If the agent can write to GitHub but has no local shell, use GitHub Actions:

- `.github/workflows/validate.yml`
- `.github/workflows/build-on-demand.yml`
- `.github/workflows/ai-tooling.yml`

Do not claim a build passed until the workflow result was actually read.

## Horizon

`.mcp.json` uses `tools/horizon_mcp_entrypoint.sh`.

It prefers:
1. `horizon-launcher mcp`;
2. `horizon-ctl mcp` from PATH;
3. the pinned local `.ai-tools/bin/horizon-ctl` built by `scripts/ai_bootstrap.sh`.

Real screenshot testing still requires a legally prepared Horizon game client/resources.
No Minecraft assets, Microsoft credentials, cookies or sessions belong in this repository.

## Canonical HMOD

https://raw.githubusercontent.com/rc2317220-max/Claude-rpg/main/releases/RustPG.hmod
