# OS API Map

Background input (no cursor steal) per OS:

| OS | API | Cursor steal? | Setup |
|---|---|---|---|
| macOS | Accessibility API (`AXUIElement`) | No | Grant in System Settings > Privacy |
| Windows | UI Automation API (`IUIAutomationElement`) | No | None (user-scope) |
| Linux X11 | `XSendEvent` + SubstructureNotifyMask | No | `apt install xdotool` |
| Linux Wayland | compositor-specific | TODO | wlroots-based |

## MCP server mode

```bash
bg-driver-rs mcp-server --port 8080
```

Expose driver to Claude Code / Cursor / Codex via MCP.
