# MCP Server Mode

Expose the bg-driver as an MCP server so other agents (Claude Code, Cursor, Codex) can use it:

```bash
bg-driver-rs mcp-server --port 8080
```

## Client config

```json
{
  "mcpServers": {
    "bg-driver": {
      "transport": "http",
      "url": "http://localhost:8080"
    }
  }
}
```

## Available tools

| Tool | Description |
|---|---|
| `screenshot` | Capture desktop screenshot (background, no cursor steal) |
| `click` | Click at (x, y) via OS Accessibility API |
| `type` | Type a string of keys |
| `scroll` | Scroll by (dx, dy) |
| `find_element` | Find UI element by selector |

## Why MCP

MCP is the emerging standard for agent-to-tool communication. Exposing bg-driver as MCP means any MCP-aware agent can drive the computer — no custom integration needed.
