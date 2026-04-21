<p align="center">
  <img src="assets/agent-terminal.png" alt="agent-terminal - Terminal automation CLI enabling AI agents to control TUI applications" width="400">
</p>

<h1 align="center">agent-terminal</h1>

<p align="center">
  <sub>The terminal equivalent of <a href="https://github.com/vercel-labs/agent-browser">agent-browser</a></sub>
</p>

<p align="center">
  <strong>Terminal automation CLI for AI agents</strong><br>
  <em>Control vim, htop, lazygit, dialog, and other TUI applications programmatically</em>
</p>

<p align="center">
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#commands">Commands</a> •
  <a href="#runtime-and-daemon">Runtime</a> •
  <a href="#usage-with-ai-agents">AI Agents</a>
</p>

---

> [!NOTE]
> **Built with AI, for AI.** This project was built with the support of an AI agent, planned thoroughly with a tight feedback loop, and reviewed at each step. While it has been tested extensively, edge cases may still exist. Use it in production with the level of caution you apply to any automation that can drive terminal software.

agent-terminal lets AI agents interact with terminal applications through a simple command-line interface. It manages pseudo-terminal (PTY) sessions with VT100 terminal emulation, captures screen state, and provides keyboard and mouse input for navigating terminal user interfaces. Think of it as headless terminal automation for AI workflows.

> [!NOTE]
> **Origin:** `agent-terminal` is derived from the earlier `pilotty` project, and the mascot artwork also comes from `pilotty`.

## Features

- **PTY session management**: Spawn and manage terminal applications in background sessions.
- **Terminal emulation**: VT100 emulation for accurate screen capture and state tracking.
- **Render modes**: `basic`, `styled`, and `color` snapshot fidelity.
- **ANSI text output**: `snapshot --format text` with styled/color render modes emits ANSI SGR sequences that recreate terminal appearance.
- **Keyboard-first interaction**: Drive TUIs with `press`, `type`, `wait-for`, `scroll`, and `click` commands.
- **AI-friendly output**: Structured JSON responses with actionable suggestions on errors.
- **Multi-session support**: Run multiple terminal apps simultaneously in isolated sessions.
- **Zero-config daemon**: The daemon auto-starts on first use and auto-stops after 5 minutes idle.

## Why agent-terminal?

[agent-browser](https://github.com/vercel-labs/agent-browser) gives AI agents a browser automation surface. agent-terminal does the same for terminal apps, which makes it useful for TUIs, curses-style dashboards, interactive CLIs, and editor workflows that do not expose a web UI.

## Installation

### GitHub release tarballs (recommended)

Download the `agent-terminal-<target>.tar.gz` asset for your platform from the latest GitHub Release, then extract and install the binary somewhere on your `PATH`:

```bash
tar -xzf agent-terminal-<target>.tar.gz
install -m 0755 agent-terminal ~/.local/bin/agent-terminal
```

The release also includes `agent-terminal-completions.tar.gz` if you prefer to install generated shell completions from the published artifacts.

### Build from source

Requires [Rust](https://rustup.rs) 1.70+.

```bash
git clone <repository-url> agent-terminal
cd agent-terminal
cargo build --release -p agent-terminal-cli
install -m 0755 target/release/agent-terminal ~/.local/bin/agent-terminal
```

### Shell completions

Generate completions from the installed binary:

```bash
agent-terminal completions bash > ~/.local/share/bash-completion/completions/agent-terminal
agent-terminal completions zsh > ~/.zfunc/_agent-terminal
agent-terminal completions fish > ~/.config/fish/completions/agent-terminal.fish
```

### npm distribution

npm distribution is intentionally disabled for now. The supported public install paths are GitHub release tarballs, shell completion artifacts, or building the binary from source.

### Verify installation

```bash
agent-terminal --version
agent-terminal --help
```

## Platform Support

| Platform | Architecture | Status |
|----------|--------------|--------|
| macOS | x64 (Intel) | ✅ |
| macOS | arm64 (Apple Silicon) | ✅ |
| Linux | x64 | ✅ |
| Linux | arm64 | ✅ |
| Windows | - | ❌ Not supported |

Windows is not currently supported because the daemon/runtime model depends on Unix domain sockets and POSIX PTY APIs.

## Quick Start

```bash
# Spawn a TUI application
agent-terminal spawn htop

# Take a snapshot of the terminal
agent-terminal snapshot

# Type text
agent-terminal type "hello world"

# Send keys with the preferred press surface
agent-terminal press Enter
agent-terminal press Control+C

# Click at specific coordinates (row, col)
agent-terminal click 10 5

# List active sessions
agent-terminal list-sessions

# Stop the daemon
agent-terminal stop
```

Compatibility spellings remain available for existing scripts: `agent-terminal key ...`, `Ctrl+...`, `Alt+...`, and short arrows like `Up`. New docs and examples use `press` with `Control+...`, `Meta+...`, `Option+...`, and `Arrow...` first.

## Commands

### Session management

```bash
agent-terminal spawn <command>           # Spawn a TUI app (for example, vim file.txt)
agent-terminal spawn --name myapp <cmd>  # Spawn with a custom session name
agent-terminal spawn --cwd /path cmd     # Spawn in a specific working directory
agent-terminal kill                      # Kill default session
agent-terminal kill -s myapp             # Kill specific session
agent-terminal list-sessions             # List all active sessions
agent-terminal stop                      # Stop the daemon and all sessions
agent-terminal daemon                    # Manually start daemon (usually auto-starts)
agent-terminal examples                  # Show end-to-end workflow example
```

### Screen capture

```bash
agent-terminal snapshot                      # Full JSON with text
agent-terminal snapshot --format compact     # JSON without text field
agent-terminal snapshot --format text        # Plain text with cursor indicator
agent-terminal snapshot -s editor            # Snapshot a specific session

# Render modes control style/color fidelity
agent-terminal snapshot --render basic       # Text only
agent-terminal snapshot --render styled      # Text attributes (bold, italic, underline, dim, inverse)
agent-terminal snapshot --render color       # Full color (default: style_map + color_map)

# ANSI text output recreates terminal appearance
agent-terminal snapshot --format text --render color
agent-terminal snapshot --format text --render styled

# Wait for the screen to change before returning
HASH=$(agent-terminal snapshot | jq -r '.content_hash')
agent-terminal press Enter
agent-terminal snapshot --await-change $HASH
agent-terminal snapshot --await-change $HASH --settle 100
```

### Input and interaction

```bash
agent-terminal type "hello"
agent-terminal press Enter
agent-terminal press Control+C
agent-terminal press Meta+F
agent-terminal press ArrowUp
agent-terminal press F1
agent-terminal press Tab
agent-terminal press Escape
agent-terminal press "Control+X m"
agent-terminal press "Escape : w q Enter"
agent-terminal press "a b c" --delay 50
agent-terminal click 10 5
agent-terminal scroll up
agent-terminal scroll down 5
```

Compatibility spellings: `key`, `Ctrl+...`, `Alt+...`, and short arrows like `Up` still work. Prefer `press` with `Control+...`, `Meta+...`, `Option+...`, and `Arrow...` when writing new examples or prompts.

### Terminal control

```bash
agent-terminal resize 120 40
agent-terminal wait-for "Ready"
agent-terminal wait-for "Error" --regex
agent-terminal wait-for "Done" -t 5000
```

## Snapshot Output

The `snapshot` command returns structured data about the terminal screen:

```json
{
  "snapshot_id": 42,
  "size": { "cols": 80, "rows": 24 },
  "cursor": { "row": 5, "col": 10, "visible": true },
  "text": "Options: [x] Enable  [ ] Debug\nActions: [OK] [Cancel]",
  "elements": [
    { "kind": "toggle", "row": 0, "col": 9, "width": 3, "text": "[x]", "confidence": 1.0, "checked": true },
    { "kind": "toggle", "row": 0, "col": 22, "width": 3, "text": "[ ]", "confidence": 1.0, "checked": false },
    { "kind": "button", "row": 1, "col": 9, "width": 4, "text": "[OK]", "confidence": 0.8 },
    { "kind": "button", "row": 1, "col": 14, "width": 8, "text": "[Cancel]", "confidence": 0.8 }
  ],
  "content_hash": 12345678901234567890
}
```

### Render modes

Sessions always capture full style and color data internally, and `agent-terminal snapshot` defaults to `--render color` unless you override it per request.

| Mode | `--format full` (JSON) | `--format text` |
|------|------------------------|-----------------|
| `basic` | text + elements | Plain text with cursor indicator |
| `styled` | text + elements + `style_map` | ANSI text with bold/italic/underline |
| `color` (default) | text + elements + `style_map` + `color_map` | ANSI text with full color |

```bash
agent-terminal snapshot
agent-terminal snapshot --render styled
agent-terminal snapshot --render basic
```

### ANSI text output

With `--format text` and `--render styled` or `--render color`, the output contains ANSI SGR escape sequences that recreate the terminal's visual appearance:

```bash
agent-terminal spawn ls --color
agent-terminal snapshot --format text --render color
agent-terminal snapshot -s myapp --format text --render color | less -R
```

## Runtime and Daemon

agent-terminal uses a daemon architecture similar to agent-browser:

```
┌─────────────┐     Unix Socket      ┌─────────────────┐
│   CLI       │ ──────────────────▶  │     Daemon      │
│agent-terminal│    JSON-line        │  (auto-started) │
└─────────────┘                      └─────────────────┘
                                              │
                                     ┌────────┴────────┐
                                     ▼                 ▼
                              ┌───────────┐     ┌───────────┐
                              │  Session  │     │  Session  │
                              │  (htop)   │     │  (vim)    │
                              └───────────┘     └───────────┘
```

- **Auto-start**: The daemon starts automatically on the first command.
- **Auto-stop**: The daemon shuts down after 5 minutes with no active sessions.
- **Session cleanup**: Sessions are removed when their child process exits.
- **Shared state**: Multiple CLI invocations can address the same named session.
- **Clean shutdown**: `agent-terminal stop` gracefully terminates all sessions.

### Runtime paths

The daemon socket directory is resolved in this priority order:

1. `$AGENT_TERMINAL_SOCKET_DIR/{session}.sock`
2. `$XDG_RUNTIME_DIR/agent-terminal/{session}.sock`
3. `~/.agent-terminal/{session}.sock`
4. `/tmp/agent-terminal/{session}.sock`

Related public environment variables:

| Variable | Description |
|----------|-------------|
| `AGENT_TERMINAL_SESSION` | Default session name |
| `AGENT_TERMINAL_SOCKET_DIR` | Override socket directory |
| `RUST_LOG` | Logging level (for example `debug`, `info`) |

### Example daemon-backed workflow

```bash
# Spawn a TUI in a named session
agent-terminal spawn --name editor vi /tmp/hello.txt

# Wait for the session to be ready
agent-terminal wait-for -s editor "hello.txt"

# Capture a baseline hash, then make a change
HASH=$(agent-terminal snapshot -s editor | jq -r '.content_hash')
agent-terminal press -s editor i
agent-terminal type -s editor "Hello from agent-terminal!"
agent-terminal press -s editor Escape

# Wait for the screen to settle, then save and quit
agent-terminal snapshot -s editor --await-change "$HASH" --settle 50
agent-terminal type -s editor ":wq"
agent-terminal press -s editor Enter
```

## Usage with AI Agents

### Just tell the agent to use it

Most coding agents can work directly from the CLI help:

```text
Use agent-terminal to interact with vim. Run agent-terminal --help to inspect the available commands first.
```

### Add an instruction snippet to AGENTS.md / CLAUDE.md

```markdown
## Terminal Automation

Use `agent-terminal` for TUI automation. Run `agent-terminal --help` for the full command list.

Core workflow:
1. `agent-terminal spawn <command>` - Start a TUI application
2. `agent-terminal snapshot` - Get screen state and capture `content_hash` when needed
3. `agent-terminal press Tab` / `agent-terminal type "text"` - Navigate and interact
4. `agent-terminal wait-for` or `snapshot --await-change` - Synchronize instead of sleeping
5. `agent-terminal list-sessions` / `agent-terminal kill` - Inspect and clean up sessions
```

### Example: AI TUI interaction

For AI-powered terminal apps that stream responses:

```bash
agent-terminal spawn --name ai opencode
agent-terminal wait-for -s ai "Ask anything" -t 15000
HASH=$(agent-terminal snapshot -s ai | jq -r '.content_hash')
agent-terminal type -s ai "write a haiku about rust"
agent-terminal press -s ai Enter
agent-terminal snapshot -s ai --await-change "$HASH" --settle 3000 -t 60000 --format text
agent-terminal scroll -s ai up 10
agent-terminal snapshot -s ai --format text
agent-terminal kill -s ai
```

## Key Combinations

Preferred contract: use `agent-terminal press` and spell modifiers/arrows as `Control+...`, `Meta+...`, `Option+...`, and `Arrow...` in new docs, prompts, and scripts.

Compatibility spellings: `key`, `Ctrl+...`, `Alt+...`, and short arrows like `Up` still work for existing scripts.

| Surface | Preferred examples | Compatibility notes |
|---------|--------------------|---------------------|
| Named keys | `Enter`, `Tab`, `Escape`, `Space`, `Backspace` | Case insensitive aliases like `Return` = `Enter`, `Esc` = `Escape` still work |
| Arrow keys | `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight` | Short forms like `Up`, `Down`, `Left`, `Right` still parse |
| Navigation | `Home`, `End`, `PageUp`, `PageDown`, `Insert`, `Delete` | Also: `PgUp`, `PgDn`, `Ins`, `Del` |
| Function keys | `F1` - `F12` | Unchanged |
| Modifier combos | `Control+C`, `Meta+F`, `Option+F`, `Shift+A` | `Ctrl+C` and `Alt+F` remain supported |
| Combined modifiers | `Control+Option+C` | Short-form combinations like `Ctrl+Alt+C` still parse |
| Special | `Plus` | Literal `+` character |
| Sequences | `"Control+X m"`, `"Escape : w q Enter"` | Space-separated keys |

## Contributing

Contributions are welcome. Before opening a change, run the project checks that match your scope (for example `cargo fmt`, `cargo clippy --all --all-features`, targeted tests, and any README/runtime audits touched by your changes).

## License

MIT

## Acknowledgments

- Inspired by [agent-browser](https://github.com/vercel-labs/agent-browser) by Vercel Labs
- Built with [vt100](https://crates.io/crates/vt100) for terminal emulation
- Built with [portable-pty](https://crates.io/crates/portable-pty) for PTY management
