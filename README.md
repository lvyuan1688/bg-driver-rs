# bg-driver-rs

> Background computer-use driver for macOS, Windows, and Linux — in Rust.
> Inspired by [trycua/cua](https://github.com/trycua/cua) (21k+ stars), rewritten from scratch in pure Rust (no Swift/Go/Python mix) with Windows as a first-class citizen.

## Why

cua is a 21k-star computer-use agent framework, but:
- Multi-language mix (Swift 63% / Go 14% / Python 10%) makes cross-platform debugging painful
- Windows support is documented but code paths are Swift-first
- cua-bench (benchmark suite) is too heavy for casual users

**bg-driver-rs** ships:
- **Pure Rust** — one language, one toolchain, cross-OS compile
- **Windows first-class** — Windows UI Automation API as primary target
- **Lightweight mini-bench** — 10-task quick benchmark vs cua-bench's full RL environment
- **MCP server mode** — expose driver as MCP server for Claude Code / Cursor / Codex

## Architecture

```
bg-driver-rs/
  crates/
    bg-driver/              # Core driver trait + 3 OS implementations
      src/
        trait.rs            # pub trait ComputerDriver
        macos.rs            # macOS Accessibility API
        windows.rs          # Windows UI Automation API (via windows-rs)
        linux.rs            # X11 (Wayland TODO)
    bg-agent/               # Agent SDK for computer-use tasks
      src/
        agent.rs            # see screen → click → verify loop
        screenshot.rs       # background screenshot (no cursor steal)
    bg-sandbox/             # Sandbox SDK
      src/
        sandbox.rs          # create/control isolated sandboxes
        computer_server.rs  # UI interaction driver inside sandbox
    bg-bench/               # Mini-benchmark (10 tasks)
      src/
        tasks.rs            # 10 canonical tasks (open mail, fill form...)
        runner.rs           # run task → measure success/time
  mcp-server/               # MCP server mode (expose driver to other agents)
  examples/
    basic_driver.rs
    mcp_server.rs
    mini_bench.rs
```

### Core trait

```rust
#[async_trait]
pub trait ComputerDriver: Send + Sync {
    async fn screenshot(&self) -> Result<Vec<u8>>;           // PNG, background
    async fn click(&self, x: i32, y: i32) -> Result<()>;     // background click
    async fn type_text(&self, text: &str) -> Result<()>;     // background type
    async fn key_press(&self, key: KeyCode) -> Result<()>;
    async fn scroll(&self, dx: i32, dy: i32) -> Result<()>;
    async fn find_element(&self, by: Selector) -> Result<Element>;
    async fn get_active_window(&self) -> Result<WindowInfo>;
    fn os(&self) -> Os;                                       // Mac/Win/Linux
    fn supports_background(&self) -> bool;                   // no cursor steal?
}
```

### Background input (core differentiator)

cua's selling point is "agents click, type, and verify **without stealing the cursor or focus**." bg-driver-rs implements this per-OS:

| OS | Background Input API | Cursor Steal? |
|---|---|---|
| macOS | Accessibility API (`AXUIElementCreateApplication`) | No |
| Windows | UI Automation API (`IUIAutomationElement`) | No |
| Linux X11 | `XSendEvent` with `SubstructureNotifyMask` | No |
| Linux Wayland | compositor-specific (TODO) | — |

### Agent loop

```
User task ("open Mail and archive all unread")
  ↓
bg-sandbox launches (cloud/local VM, any OS)
  ↓
Loop:
  1. screenshot (background, no cursor steal)
  2. LLM sees screenshot → "click (340, 215)"
  3. bg-driver executes click via OS Accessibility API
  4. wait for UI stable → screenshot
  ↓
Task complete → trajectory saved to ~/.bg-driver-rs/trajectories/
```

### MCP server mode

Expose driver as MCP server so other agents (Claude Code, Cursor, Codex) can use it:

```bash
bg-driver-rs mcp-server --port 8080
```

```json
// Other agent's config
{
  "mcpServers": {
    "bg-driver": {
      "transport": "http",
      "url": "http://localhost:8080"
    }
  }
}
```

### Mini-bench (10 tasks)

```bash
bg-driver-rs bench --os windows --tasks all
```

```
Task 1: Open Notepad and type "hello"           ✓ (2.3s)
Task 2: Open Calculator and compute 5+3         ✓ (3.1s)
Task 3: Open Mail, archive all unread           ✗ (timeout 30s)
...
Result: 8/10 tasks passed, avg 4.2s
```

## Install

```bash
cargo install bg-driver-rs
```

## Quick start

```bash
# macOS: grant Accessibility permission in System Settings > Privacy & Security
# Windows: no special permission needed (UI Automation is user-scope)
# Linux: ensure X11 or wlroots compositor

bg-driver-rs "open notepad and type hello world"
```

## OS-specific setup

### macOS
```bash
# Grant Accessibility permission (first run will prompt)
sudo sqlite3 /Library/Application\ Support/com.apple.TCC/TCC.db \
  "INSERT OR REPLACE INTO access VALUES('kTCCServiceAccessibility','/usr/local/bin/bg-driver-rs',1,1,1,NULL,NULL,NULL,'0',NULL,NULL, '1643659723');"
```

### Windows
```powershell
# No special setup — UI Automation API is user-scope
# Just ensure Windows UI Automation service is running (default: running)
```

### Linux (X11)
```bash
# Ensure xdotool and xsel are installed
sudo apt install xdotool xsel
```

## Roadmap

- [x] ComputerDriver trait + macOS/Windows/Linux implementations
- [x] Background screenshot (no cursor steal)
- [x] Sandbox SDK (cloud/local VM control)
- [x] MCP server mode
- [x] Mini-bench (10 tasks)
- [ ] Wayland background input (compositor-specific)
- [ ] Cross-OS fleet management (cua's "fleets" feature)

## License

MIT — see [LICENSE](LICENSE).

## Acknowledgments

- [trycua/cua](https://github.com/trycua/cua) — original 21k-star computer-use agent framework that inspired this Rust rewrite
- [windows-rs](https://github.com/microsoft/windows-rs) — Rust bindings for Windows API
- [accessibility-sys](https://crates.io/crates/accessibility-sys) — macOS Accessibility API bindings
