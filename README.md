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
- **Two-axis snapshot control**: `--format` (encoding: `ansi` or `json`) × `--render` (feature set: `text`, `style`, `color`).
- **ANSI text output**: `snapshot --format ansi` emits ANSI SGR sequences that recreate terminal appearance.
- **Token-efficient JSON**: `snapshot --format json --render text` strips all style/color spans for minimum LLM token usage.
- **Keyboard-first interaction**: Drive TUIs with `press`, `type`, `wait`, `scroll`, and `click` commands.
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
# Spawn a deterministic interactive shell session
agent-terminal spawn --name shell env PS1='agent-terminal> ' bash --noprofile --norc -i

# Wait for the prompt before sending input
agent-terminal wait -s shell "agent-terminal> "

# Capture a baseline before triggering a visible change
HASH=$(agent-terminal snapshot -s shell --format json | jq -r '.content_hash')

# Run one command
agent-terminal type -s shell "printf 'hello from agent-terminal\n'"
agent-terminal press -s shell Enter

# Wait for the change to settle, then inspect the result
agent-terminal snapshot -s shell --await-change "$HASH" --settle 100
agent-terminal snapshot -s shell --format json --render text

# Clean up
agent-terminal kill -s shell
agent-terminal stop
```

Compatibility spellings remain available for existing scripts: `agent-terminal key ...`, `Ctrl+...`, `Alt+...`, short arrows like `Up`, and `agent-terminal wait-for ...`. New docs and examples use `press` with `Control+...`, `Meta+...`, `Option+...`, and `Arrow...`, plus `wait` for simple text/regex polling, first.

Use `agent-terminal snapshot --await-change <content_hash> --settle <ms>` when you need to wait for the screen to both change and stabilize.

Broader command grammar cleanup is intentionally deferred to [M009-HANDOFF.md](./M009-HANDOFF.md) so M008 can standardize the preferred shell lifecycle without changing runtime or protocol semantics.

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
agent-terminal snapshot                                  # ANSI output (default)
agent-terminal snapshot --format json                    # JSON rows for agent/LLM consumption
agent-terminal snapshot --format json --render text      # Minimal tokens: text only, no spans
agent-terminal snapshot --format json --render text,color  # Text + colors, no bold/italic info
agent-terminal snapshot -s editor                        # Snapshot a specific session

# --format controls encoding
#   ansi  ANSI-escaped output — renders with color/style in a terminal (default)
#   json  Structured JSON with a 'rows' array — for LLM/agent consumption

# --render controls what data appears in the output (comma-separated, default: text,style,color)
#   text   Plain text content (always included)
#   style  Text attributes: bold, italic, dim, underline, inverse
#   color  Foreground and background colors

# ANSI output recreates terminal appearance
agent-terminal snapshot                         # ANSI with full style+color
agent-terminal snapshot --render text           # ANSI plain text only

# Wait for the screen to change before returning
HASH=$(agent-terminal snapshot --format json | jq -r '.content_hash')
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
agent-terminal wait "Ready"
agent-terminal wait "Error" --regex
agent-terminal wait "Done" -t 5000
```

Prefer `agent-terminal wait` for literal text or regex polling. `agent-terminal wait-for ...` remains available as a compatibility alias for existing scripts.

Use `agent-terminal snapshot --await-change <content_hash> --settle <ms>` when you need to wait for the screen to both change and stabilize.

## Snapshot Output

The `snapshot --format json` command returns structured data about the terminal screen. Each row is represented as an entry in the `rows` array; spans only appear when style/color data is present.

```json
{
  "type": "screen_state",
  "snapshot_id": 42,
  "size": { "cols": 80, "rows": 24 },
  "cursor": { "row": 5, "col": 10, "visible": true },
  "rows": [
    {
      "r": 0,
      "t": "ERROR normal blue",
      "spans": [
        { "c": 0, "l": 5, "s": { "fg": 1, "b": true } },
        { "c": 13, "l": 4, "s": { "fg": 4 } }
      ]
    },
    { "r": 1, "t": "dim text", "spans": [{ "c": 0, "l": 8, "s": { "d": true } }] }
  ],
  "elements": [
    { "kind": "toggle", "row": 0, "col": 9, "width": 3, "text": "[x]", "confidence": 1.0, "checked": true },
    { "kind": "button", "row": 1, "col": 9, "width": 4, "text": "[OK]", "confidence": 0.8 }
  ],
  "content_hash": 12345678901234567890
}
```

Row fields: `r` = row index, `t` = text content, `spans` = optional style/color spans.
Span fields: `c` = start column, `l` = length, `s` = style object.
Style fields (`s`): `fg`/`bg` = color index, `b` = bold, `i` = italic, `d` = dim, `u` = underline, `v` = inverse. All fields are optional and omitted when not set.

### Two-axis snapshot control

Snapshot output is controlled by two independent flags:

| Flag | Values | Description |
|------|--------|-------------|
| `--format` | `ansi` (default), `json` | Output encoding |
| `--render` | `text,style,color` (default) | Comma-separated feature set |

**`--render` features:**

| Feature | Description |
|---------|-------------|
| `text` | Plain text content (always included) |
| `style` | Text attributes: bold, italic, dim, underline, inverse |
| `color` | Foreground and background colors |

**Token budget examples for LLM/agent use:**

```bash
agent-terminal snapshot --format json                          # Full JSON (default render)
agent-terminal snapshot --format json --render text            # Minimum tokens: text only
agent-terminal snapshot --format json --render text,color      # Text + colors, no style attrs
agent-terminal snapshot --format json --render text,style,color  # All data (same as default)
```

**jq recipes:**

```bash
# All row text as an array
agent-terminal snapshot --format json | jq '[.rows[].t]'

# Find a specific row by index
agent-terminal snapshot --format json | jq '.rows[] | select(.r == 5)'

# Find rows containing "error"
agent-terminal snapshot --format json | jq '.rows[] | select(.t | contains("error"))'
```

### ANSI output

`--format ansi` (the default) emits ANSI SGR escape sequences that recreate the terminal's visual appearance. Use `--render` to control which attributes are included:

```bash
agent-terminal snapshot                             # ANSI with full style+color
agent-terminal snapshot --render text               # ANSI plain text (no SGR codes)
agent-terminal snapshot | less -R                   # Pipe into a pager
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
# Spawn a named shell session with an explicit prompt
agent-terminal spawn --name shell env PS1='agent-terminal> ' bash --noprofile --norc -i

# Wait for the prompt before sending input
agent-terminal wait -s shell "agent-terminal> "

# Capture a baseline hash, then trigger visible output
HASH=$(agent-terminal snapshot -s shell --format json | jq -r '.content_hash')
agent-terminal type -s shell "printf 'hello from agent-terminal\n'"
agent-terminal press -s shell Enter

# Wait for the output to change and settle, then inspect it
agent-terminal snapshot -s shell --await-change "$HASH" --settle 100
agent-terminal snapshot -s shell --format json --render text

# Clean up the session and daemon explicitly
agent-terminal kill -s shell
agent-terminal stop
```

## Usage with AI Agents

### Just tell the agent to use it

Most coding agents can work directly from the CLI help:

```text
Use agent-terminal to drive one shell session end to end. Start with agent-terminal examples or --help, then follow the lifecycle: spawn a named shell with an explicit prompt, wait for readiness, type a command, press Enter, snapshot --await-change --settle after the visible change, and clean up with kill then stop.
```

### Add an instruction snippet to AGENTS.md / CLAUDE.md

```markdown
## Terminal Automation

Use `agent-terminal` for TUI automation. Run `agent-terminal examples` or `agent-terminal --help` before the first interaction.

Preferred shell lifecycle:
1. `agent-terminal spawn --name shell env PS1='agent-terminal> ' bash --noprofile --norc -i` - Start a deterministic shell session.
2. `agent-terminal wait -s shell "agent-terminal> "` - Wait for the prompt before sending input.
3. `HASH=$(agent-terminal snapshot -s shell --format json | jq -r '.content_hash')` - Capture a baseline before the next visible change.
4. `agent-terminal type -s shell "printf 'hello from agent-terminal\n'"` then `agent-terminal press -s shell Enter` - Trigger one command.
5. `agent-terminal snapshot -s shell --await-change "$HASH" --settle 100` - Wait for the screen to change and stabilize.
6. `agent-terminal snapshot -s shell --format json --render text` - Read the updated terminal state (minimal tokens).
7. `agent-terminal kill -s shell` then `agent-terminal stop` - Clean up explicitly when done.

Compatibility spellings remain available for existing scripts: `agent-terminal key ...`, `Ctrl+...`, `Alt+...`, short arrows like `Up`, and `agent-terminal wait-for ...`.
Deferred broader grammar work lives in [M009-HANDOFF.md](./M009-HANDOFF.md).
```

### Example: AI TUI interaction

For AI-powered terminal apps that stream responses:

```bash
agent-terminal spawn --name ai opencode
agent-terminal wait -s ai "Ask anything" -t 15000
HASH=$(agent-terminal snapshot -s ai --format json | jq -r '.content_hash')
agent-terminal type -s ai "write a haiku about rust"
agent-terminal press -s ai Enter
agent-terminal snapshot -s ai --await-change "$HASH" --settle 3000 -t 60000
agent-terminal scroll -s ai up 10
agent-terminal snapshot -s ai --format json --render text
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
