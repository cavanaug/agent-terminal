---
name: agent-terminal
description: Automates terminal TUI applications (vim, htop, lazygit, dialog) through managed PTY sessions. Use when the user needs to interact with terminal apps, edit files in vim/nano, navigate TUI menus, click terminal buttons/checkboxes, or automate CLI workflows with interactive prompts.
allowed-tools: Bash(agent-terminal:*)
---

# Terminal Automation with agent-terminal

## CRITICAL: Argument Positioning

**All flags (`--name`, `-s`, `--format`, etc.) MUST come BEFORE positional arguments:**

```bash
# CORRECT - flags before command/arguments
agent-terminal spawn --name myapp vim file.txt
agent-terminal key -s myapp Enter
agent-terminal snapshot -s myapp --format text

# WRONG - flags after command (they get passed to the app, not agent-terminal!)
agent-terminal spawn vim file.txt --name myapp   # FAILS: --name goes to vim
agent-terminal key Enter -s myapp                # FAILS: -s goes nowhere useful
```

This is the #1 cause of agent failures. When in doubt: **flags first, then command/args**.

---

## Quick start

```bash
agent-terminal spawn vim file.txt        # Start TUI app in managed session
agent-terminal wait-for "file.txt"       # Wait for app to be ready
agent-terminal snapshot                  # Get screen state with UI elements
agent-terminal key i                     # Enter insert mode
agent-terminal type "Hello, World!"      # Type text
agent-terminal key Escape                # Exit insert mode
agent-terminal kill                      # End session
```

## Core workflow

1. **Spawn**: `agent-terminal spawn <command>` starts the app in a background PTY
2. **Wait**: `agent-terminal wait-for <text>` ensures the app is ready
3. **Snapshot**: `agent-terminal snapshot` returns screen state with detected UI elements
4. **Understand**: Parse `elements[]` to identify buttons, inputs, toggles
5. **Interact**: Use keyboard commands (`key`, `type`) to navigate and interact
6. **Re-snapshot**: Check `content_hash` to detect screen changes

## Commands

### Session management

```bash
agent-terminal spawn <command>           # Start TUI app (e.g., agent-terminal spawn htop)
agent-terminal spawn --name myapp <cmd>  # Start with custom session name (--name before command)
agent-terminal kill                      # Kill default session
agent-terminal kill -s myapp             # Kill specific session
agent-terminal list-sessions             # List all active sessions
agent-terminal daemon                    # Manually start daemon (usually auto-starts)
agent-terminal stop                      # Stop the daemon and all sessions
agent-terminal examples                  # Show end-to-end workflow example
```

### Screen capture

```bash
agent-terminal snapshot                  # Full JSON with text content and elements
agent-terminal snapshot --format compact # JSON without text field
agent-terminal snapshot --format text    # Plain text with cursor indicator
agent-terminal snapshot -s myapp         # Snapshot specific session

# Render modes control style/color fidelity
agent-terminal snapshot --render basic   # Text only
agent-terminal snapshot --render styled  # Adds style_map (bold, italic, underline)
agent-terminal snapshot --render color   # Adds style_map + color_map (default)

# ANSI text output — visually recreates the terminal screen
agent-terminal snapshot --format text --render color   # ANSI-styled text with colors
agent-terminal snapshot --format text --render styled  # ANSI text, text attributes only

# Wait for screen to change (eliminates need for sleep!)
HASH=$(agent-terminal snapshot | jq '.content_hash')
agent-terminal key Enter
agent-terminal snapshot --await-change $HASH           # Block until screen changes
agent-terminal snapshot --await-change $HASH --settle 50  # Wait for 50ms stability
```

### Input

```bash
agent-terminal type "hello"              # Type text at cursor
agent-terminal type -s myapp "text"      # Type in specific session

agent-terminal key Enter                 # Press Enter
agent-terminal key Ctrl+C                # Send interrupt
agent-terminal key Escape                # Send Escape
agent-terminal key Tab                   # Send Tab
agent-terminal key F1                    # Function key
agent-terminal key Alt+F                 # Alt combination
agent-terminal key Up                    # Arrow key
agent-terminal key -s myapp Ctrl+S       # Key in specific session

# Key sequences (space-separated, sent in order)
agent-terminal key "Ctrl+X m"            # Emacs chord: Ctrl+X then m
agent-terminal key "Escape : w q Enter"  # vim :wq sequence
agent-terminal key "a b c" --delay 50    # Send a, b, c with 50ms delay
agent-terminal key -s myapp "Tab Tab Enter"  # Sequence in specific session
```

### Interaction

```bash
agent-terminal click 5 10                # Click at row 5, col 10
agent-terminal click -s myapp 10 20      # Click in specific session
agent-terminal scroll up                 # Scroll up 1 line
agent-terminal scroll down 5             # Scroll down 5 lines
agent-terminal scroll up 10 -s myapp     # Scroll in specific session
```

### Terminal control

```bash
agent-terminal resize 120 40             # Resize terminal to 120 cols x 40 rows
agent-terminal resize 80 24 -s myapp     # Resize specific session

agent-terminal wait-for "Ready"          # Wait for text to appear (30s default)
agent-terminal wait-for "Error" -r       # Wait for regex pattern
agent-terminal wait-for "Done" -t 5000   # Wait with 5s timeout
agent-terminal wait-for "~" -s editor    # Wait in specific session
```

## Global options

| Option | Description |
|--------|-------------|
| `-s, --session <name>` | Target specific session (default: "default") |
| `--format <fmt>` | Snapshot format: full, compact, text |
| `--render <mode>` | Render mode: basic (text only), styled (text attrs), color (full color) |
| `-t, --timeout <ms>` | Timeout for wait-for and await-change (default: 30000) |
| `-r, --regex` | Treat wait-for pattern as regex |
| `--name <name>` | Session name for spawn command |
| `--delay <ms>` | Delay between keys in a sequence (default: 0, max: 10000) |
| `--await-change <hash>` | Block snapshot until content_hash differs |
| `--settle <ms>` | Wait for screen to be stable for this many ms (default: 0) |

### Environment variables

```bash
AGENT_TERMINAL_SESSION="mysession"       # Default session name
AGENT_TERMINAL_SOCKET_DIR="/tmp/agent-terminal" # Override socket directory
RUST_LOG="debug"                  # Enable debug logging
```

## Snapshot Output

The `snapshot` command returns structured JSON with detected UI elements:

```json
{
  "snapshot_id": 42,
  "size": { "cols": 80, "rows": 24 },
  "cursor": { "row": 5, "col": 10, "visible": true },
  "text": "Settings:\n  [x] Notifications  [ ] Dark mode\n  [Save]  [Cancel]",
  "elements": [
    { "kind": "toggle", "row": 1, "col": 2, "width": 3, "text": "[x]", "confidence": 1.0, "checked": true },
    { "kind": "toggle", "row": 1, "col": 20, "width": 3, "text": "[ ]", "confidence": 1.0, "checked": false },
    { "kind": "button", "row": 2, "col": 2, "width": 6, "text": "[Save]", "confidence": 0.8 },
    { "kind": "button", "row": 2, "col": 10, "width": 8, "text": "[Cancel]", "confidence": 0.8 }
  ],
  "content_hash": 12345678901234567890
}
```

With `--render styled` or `--render color`, the Full JSON snapshot also includes `style_map` and/or `color_map`:

```json
{
  "style_map": [{ "r": 0, "c": 0, "l": 9, "s": { "b": true } }],
  "color_map": [{ "r": 0, "c": 0, "l": 14, "s": { "fg": 1 } }]
}
```

**Style keys:** `b` (bold), `i` (italic), `d` (dim), `u` (underline), `v` (inverse)
**Color values:** indexed (0-7), extended (8-255), RGB (`"#rrggbb"`)

Use `--format text` for a plain text view with cursor indicator:

```
--- Terminal 80x24 | Cursor: (5, 10) ---
bash-3.2$ [_]
```

The `[_]` shows cursor position. Use the text content to understand screen state and navigate with keyboard commands.

---

## Element Detection

agent-terminal automatically detects interactive UI elements in terminal applications. Elements provide **read-only context** to help understand UI structure.

### Element Kinds

| Kind | Detection Patterns | Confidence | Fields |
|------|-------------------|------------|--------|
| **toggle** | `[x]`, `[ ]`, `[*]`, `☑`, `☐` | 1.0 | `checked: bool` |
| **button** | Inverse video, `[OK]`, `<Cancel>`, `(Submit)` | 1.0 / 0.8 | `focused: bool` (if true) |
| **input** | Cursor position, `____` underscores | 1.0 / 0.6 | `focused: bool` (if true) |

### Element Fields

| Field | Type | Description |
|-------|------|-------------|
| `kind` | string | Element type: `button`, `input`, or `toggle` |
| `row` | number | Row position (0-based from top) |
| `col` | number | Column position (0-based from left) |
| `width` | number | Width in terminal cells (CJK chars = 2) |
| `text` | string | Text content of the element |
| `confidence` | number | Detection confidence (0.0-1.0) |
| `focused` | bool | Whether element has focus (only present if true) |
| `checked` | bool | Toggle state (only present for toggles) |

### Confidence Levels

| Confidence | Meaning |
|------------|---------|
| **1.0** | High confidence: Cursor position, inverse video, checkbox patterns |
| **0.8** | Medium confidence: Bracket patterns `[OK]`, `<Cancel>` |
| **0.6** | Lower confidence: Underscore input fields `____` |

### Wait for Screen Changes (Recommended)

**Stop guessing sleep durations!** Use `--await-change` to wait for the screen to actually update:

```bash
# Capture baseline hash
HASH=$(agent-terminal snapshot | jq '.content_hash')

# Perform action
agent-terminal key Enter

# Wait for screen to change (blocks until hash differs)
agent-terminal snapshot --await-change $HASH

# Or wait for screen to stabilize (for apps that render progressively)
agent-terminal snapshot --await-change $HASH --settle 100
```

**Flags:**
| Flag | Description |
|------|-------------|
| `--await-change <HASH>` | Block until `content_hash` differs from this value |
| `--settle <MS>` | After change detected, wait for screen to be stable for MS |
| `-t, --timeout <MS>` | Maximum wait time (default: 30000) |

**Why this is better than sleep:**
- `sleep 1` is a guess - too short causes race conditions, too long slows automation
- `--await-change` waits exactly as long as needed - no more, no less
- `--settle` handles apps that render progressively (show partial, then complete)

### Waiting for Streaming AI Responses

When interacting with AI-powered TUIs (like opencode, etc.) that stream responses, you need a longer `--settle` time since the screen keeps updating as tokens arrive:

```bash
# 1. Capture hash before sending prompt
HASH=$(agent-terminal snapshot -s myapp | jq -r '.content_hash')

# 2. Type prompt and submit
agent-terminal type -s myapp "write me a poem about ai agents"
agent-terminal key -s myapp Enter

# 3. Wait for streaming response to complete
#    - Use longer settle (2-3s) since AI apps pause between chunks
#    - Extend timeout for long responses (60s+)
agent-terminal snapshot -s myapp --await-change "$HASH" --settle 3000 -t 60000

# 4. Response may be scrolled - scroll up if needed to see full output
agent-terminal scroll -s myapp up 10
agent-terminal snapshot -s myapp --format text
```

**Key parameters for streaming:**
- `--settle 2000-3000`: AI responses have pauses between chunks; 2-3 seconds ensures streaming is truly done
- `-t 60000`: Extend timeout beyond the 30s default for longer generations
- The settle timer resets on each screen change, so it naturally waits until streaming stops

### Manual Change Detection

For manual polling (not recommended), use `content_hash` directly:

```bash
# Get initial state
SNAP1=$(agent-terminal snapshot)
HASH1=$(echo "$SNAP1" | jq -r '.content_hash')

# Perform action
agent-terminal key Tab

# Check if screen changed
SNAP2=$(agent-terminal snapshot)
HASH2=$(echo "$SNAP2" | jq -r '.content_hash')

if [ "$HASH1" != "$HASH2" ]; then
  echo "Screen changed - re-analyze elements"
fi
```

### Using Elements Effectively

Elements are **read-only context** for understanding the UI. Use **keyboard navigation** for reliable interaction:

```bash
# 1. Get snapshot to understand UI structure
agent-terminal snapshot | jq '.elements'
# Output shows toggles (checked/unchecked) and buttons with positions

# 2. Navigate and interact with keyboard (reliable approach)
agent-terminal key Tab          # Move to next element
agent-terminal key Space        # Toggle checkbox
agent-terminal key Enter        # Activate button

# 3. Verify state changed
agent-terminal snapshot | jq '.elements[] | select(.kind == "toggle")'
```

**Key insight**: Use elements to understand WHAT is on screen, use keyboard to interact with it.

---

## Navigation Approach

agent-terminal uses keyboard-first navigation, just like a human would:

```bash
# 1. Take snapshot to see the screen
agent-terminal snapshot --format text

# 2. Navigate using keyboard
agent-terminal key Tab           # Move to next element
agent-terminal key Enter         # Activate/select
agent-terminal key Escape        # Cancel/back
agent-terminal key Up            # Move up in list/menu
agent-terminal key Space         # Toggle checkbox

# 3. Type text when needed
agent-terminal type "search term"
agent-terminal key Enter

# 4. Click at coordinates for mouse-enabled TUIs
agent-terminal click 5 10        # Click at row 5, col 10
```

**Key insight**: Parse the snapshot text and elements to understand what's on screen, then use keyboard commands to navigate. This works reliably across all TUI applications.

---

## Example: Edit file with vim

```bash
# 1. Spawn vim
agent-terminal spawn --name editor vim /tmp/hello.txt

# 2. Wait for vim to load and capture baseline hash
agent-terminal wait-for -s editor "hello.txt"
HASH=$(agent-terminal snapshot -s editor | jq '.content_hash')

# 3. Enter insert mode
agent-terminal key -s editor i

# 4. Type content
agent-terminal type -s editor "Hello from agent-terminal!"

# 5. Wait for screen to update, then exit (no sleep needed!)
agent-terminal snapshot -s editor --await-change $HASH --settle 50
agent-terminal key -s editor "Escape : w q Enter"

# 6. Verify session ended
agent-terminal list-sessions
```

Alternative using individual keys:
```bash
agent-terminal key -s editor Escape
agent-terminal type -s editor ":wq"
agent-terminal key -s editor Enter
```

## Example: Dialog checklist interaction

```bash
# 1. Spawn dialog checklist (--name before command)
agent-terminal spawn --name opts dialog --checklist "Select features:" 12 50 4 \
    "notifications" "Push notifications" on \
    "darkmode" "Dark mode theme" off \
    "autosave" "Auto-save documents" on \
    "telemetry" "Usage analytics" off

# 2. Wait for dialog to render (use await-change, not sleep!)
agent-terminal snapshot -s opts --settle 200  # Wait for initial render to stabilize

# 3. Get snapshot and examine elements, capture hash
SNAP=$(agent-terminal snapshot -s opts)
echo "$SNAP" | jq '.elements[] | select(.kind == "toggle")'
HASH=$(echo "$SNAP" | jq '.content_hash')

# 4. Navigate to "darkmode" and toggle it
agent-terminal key -s opts Down      # Move to second option
agent-terminal key -s opts Space     # Toggle it on

# 5. Wait for change and verify
agent-terminal snapshot -s opts --await-change $HASH | jq '.elements[] | select(.kind == "toggle") | {text, checked}'

# 6. Confirm selection
agent-terminal key -s opts Enter

# 7. Clean up
agent-terminal kill -s opts
```

## Example: Form filling with elements

```bash
# 1. Spawn a form application
agent-terminal spawn --name form my-form-app

# 2. Get snapshot to understand form structure
agent-terminal snapshot -s form | jq '.elements'
# Shows inputs, toggles, and buttons with positions for click command

# 3. Tab to first input (likely already focused)
agent-terminal type -s form "myusername"

# 4. Tab to password field
agent-terminal key -s form Tab
agent-terminal type -s form "mypassword"

# 5. Tab to remember me and toggle
agent-terminal key -s form Tab
agent-terminal key -s form Space

# 6. Tab to Login and activate
agent-terminal key -s form Tab
agent-terminal key -s form Enter

# 7. Check result
agent-terminal snapshot -s form --format text
```

## Example: Monitor with htop

```bash
# 1. Spawn htop
agent-terminal spawn --name monitor htop

# 2. Wait for display
agent-terminal wait-for -s monitor "CPU"

# 3. Take snapshot to see current state
agent-terminal snapshot -s monitor --format text

# 4. Send commands
agent-terminal key -s monitor F9    # Kill menu
agent-terminal key -s monitor q     # Quit

# 5. Kill session
agent-terminal kill -s monitor
```

## Example: Interact with AI TUI (opencode, etc.)

AI-powered TUIs stream responses, requiring special handling:

```bash
# 1. Spawn the AI app
agent-terminal spawn --name ai opencode

# 2. Wait for the prompt to be ready
agent-terminal wait-for -s ai "Ask anything" -t 15000

# 3. Capture baseline hash
HASH=$(agent-terminal snapshot -s ai | jq -r '.content_hash')

# 4. Type prompt and submit
agent-terminal type -s ai "explain the architecture of this codebase"
agent-terminal key -s ai Enter

# 5. Wait for streaming response to complete
#    - settle=3000: Wait 3s of no changes to ensure streaming is done
#    - timeout=60000: Allow up to 60s for long responses
agent-terminal snapshot -s ai --await-change "$HASH" --settle 3000 -t 60000 --format text

# 6. If response is long and scrolled, scroll up to see full output
agent-terminal scroll -s ai up 20
agent-terminal snapshot -s ai --format text

# 7. Clean up
agent-terminal kill -s ai
```

**Gotchas with AI apps:**
- Use `--settle 2000-3000` because AI responses pause between chunks
- Extend timeout with `-t 60000` for complex prompts
- Long responses may scroll the terminal; use `scroll up` to see the beginning
- The settle timer resets on each screen update, so it waits for true completion

---

## Sessions

Each session is isolated with its own:
- PTY (pseudo-terminal)
- Screen buffer
- Child process

```bash
# Run multiple apps (--name must come before the command)
agent-terminal spawn --name monitoring htop
agent-terminal spawn --name editor vim file.txt

# Target specific session
agent-terminal snapshot -s monitoring
agent-terminal key -s editor Ctrl+S

# List all
agent-terminal list-sessions

# Kill specific
agent-terminal kill -s editor
```

The first session spawned without `--name` is automatically named `default`.

> **Important:** The `--name` flag must come **before** the command. Everything after the command is passed as arguments to that command.

## Daemon Architecture

agent-terminal uses a background daemon for session management:

- **Auto-start**: Daemon starts on first command
- **Auto-stop**: Shuts down after 5 minutes with no sessions
- **Session cleanup**: Sessions removed when process exits (within 500ms)
- **Shared state**: Multiple CLI calls share sessions

You rarely need to manage the daemon manually.

## Error Handling

Errors include actionable suggestions:

```json
{
  "code": "SESSION_NOT_FOUND",
  "message": "Session 'abc123' not found",
  "suggestion": "Run 'agent-terminal list-sessions' to see available sessions"
}
```

```json
{
  "code": "SPAWN_FAILED",
  "message": "Failed to spawn process: command not found",
  "suggestion": "Check that the command exists and is in PATH"
}
```

---

## Common Patterns

### Reliable action + wait (recommended)

```bash
# The pattern: capture hash, act, await change
HASH=$(agent-terminal snapshot | jq '.content_hash')
agent-terminal key Enter
agent-terminal snapshot --await-change $HASH --settle 50

# This replaces fragile patterns like:
# agent-terminal key Enter && sleep 1 && agent-terminal snapshot  # BAD: guessing
```

### Wait then act

```bash
agent-terminal spawn my-app
agent-terminal wait-for "Ready"    # Ensure app is ready
agent-terminal snapshot            # Then snapshot
```

### Check state before action

```bash
agent-terminal snapshot --format text | grep "Error"  # Check for errors
agent-terminal key Enter                               # Then proceed
```

### Check for specific element

```bash
# Check if the first toggle is checked
agent-terminal snapshot | jq '.elements[] | select(.kind == "toggle") | {text, checked}' | head -1

# Find element at specific position
agent-terminal snapshot | jq '.elements[] | select(.row == 5 and .col == 10)'
```

### Retry on timeout

```bash
agent-terminal wait-for "Ready" -t 5000 || {
  agent-terminal snapshot --format text   # Check what's on screen
  # Adjust approach based on actual state
}
```

---

## Deep-dive Documentation

For detailed patterns and edge cases, see:

| Reference | Description |
|-----------|-------------|
| [references/session-management.md](references/session-management.md) | Multi-session patterns, isolation, cleanup |
| [references/key-input.md](references/key-input.md) | Complete key combinations reference |
| [references/element-detection.md](references/element-detection.md) | Detection rules, confidence, patterns |

## Ready-to-use Templates

Executable workflow scripts:

| Template | Description |
|----------|-------------|
| [templates/vim-workflow.sh](templates/vim-workflow.sh) | Edit file with vim, save, exit |
| [templates/dialog-interaction.sh](templates/dialog-interaction.sh) | Handle dialog/whiptail prompts |
| [templates/multi-session.sh](templates/multi-session.sh) | Parallel TUI orchestration |
| [templates/element-detection.sh](templates/element-detection.sh) | Element detection demo |

Usage:
```bash
./templates/vim-workflow.sh /tmp/myfile.txt "File content here"
./templates/dialog-interaction.sh
./templates/multi-session.sh
./templates/element-detection.sh
```
