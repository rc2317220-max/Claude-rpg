# Claude RPG model workspace

This folder is the only filesystem root exposed to the headless Blockbench MCP server.

Recommended layout:

- `creatures/` — mobs and bosses (.bbmodel)
- `items/` — weapons, relics and artifacts
- `vfx/` — particle-oriented models and helpers
- `exports/` — generated geometry/animation JSON

The MCP server is pinned in `.vscode/mcp.json` to:

`jasonjgardner/blockbench-mcp-plugin@97a8a80d3a37bf95c5e136d93d2c7da399fff753`

Security: keep the server on stdio inside Codespaces. Do not expose it publicly without authentication. The headless server can write files inside this directory.

In VS Code/Codespaces, an MCP-capable coding client can start the `blockbench-headless` server from `.vscode/mcp.json`.
