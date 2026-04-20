# Session Management

agent-terminal manages multiple isolated terminal sessions, each running its own TUI application with independent state.

## CRITICAL: Flag Positioning

**All flags MUST come BEFORE positional arguments.** This applies to `--name`, `-s`/`--session`, and all other options:

```bash
# CORRECT
agent-terminal spawn --name myapp vim file.txt
agent-terminal key -s myapp Enter
agent-terminal snapshot -s myapp --format text

# WRONG - flags after positional args get passed to the command, not agent-terminal
agent-terminal spawn vim file.txt --name myapp   # --name goes to vim, session uses "default"
agent-terminal key Enter -s myapp                # -s is ignored, targets wrong session
```

---

## Session Basics

Each session has:
- **PTY**: Pseudo-terminal for the application
- **Screen buffer**: Terminal emulator state
- **Child process**: The running application

## Creating Sessions

### Default Session

The first spawn without `--name` creates the `default` session:

```bash
agent-terminal spawn htop
# Creates session named "default"

agent-terminal snapshot
# Snapshots the "default" session
```

### Named Sessions

Use `--name` for multiple concurrent sessions. **Note:** `--name` must come before the command:

```bash
agent-terminal spawn --name monitoring htop
agent-terminal spawn --name editor vim file.txt
agent-terminal spawn --name git lazygit
```

### Session Naming Rules

- Names are sanitized (alphanumeric, hyphens, underscores)
- Path traversal attempts (`../`) are rejected
- Names must be unique per daemon instance

## Targeting Sessions

Use `-s` or `--session` to target a specific session:

```bash
# Snapshot specific session
agent-terminal snapshot -s monitoring

# Send key to specific session
agent-terminal key -s editor Ctrl+S

# Send key in specific session
agent-terminal key -s git Enter

# Kill specific session
agent-terminal kill -s monitoring
```

Without `-s`, commands target the default session. Set `AGENT_TERMINAL_SESSION` if you want a different default without passing `-s` each time.

## Listing Sessions

```bash
agent-terminal list-sessions
```

Output:
```json
{
  "sessions": [
    { "id": "abc123", "name": "monitoring", "command": "htop" },
    { "id": "def456", "name": "editor", "command": "vim file.txt" },
    { "id": "ghi789", "name": "git", "command": "lazygit" }
  ]
}
```

## Session Lifecycle

### Spawn

```bash
agent-terminal spawn --name myapp my-command arg1 arg2
```

1. Daemon creates PTY
2. Forks child process with command
3. Initializes terminal emulator (default: 80x24)
4. Returns session ID

### Active Use

While a session is active:
- Screen buffer updates on process output
- Cursor position is tracked
- Terminal size can be changed with `resize`

### Process Exit

When the child process exits:
- Session is marked for cleanup
- Cleanup happens within 500ms
- Session is removed from list

### Manual Kill

```bash
agent-terminal kill -s myapp
```

Sends SIGTERM to the child process, then cleans up.

## Multi-Session Patterns

### Parallel Monitoring

Run multiple apps and switch between them:

```bash
# Start apps (--name before command)
agent-terminal spawn --name cpu htop
agent-terminal spawn --name io iotop
agent-terminal spawn --name net nethogs

# Check each
agent-terminal snapshot -s cpu --format text
agent-terminal snapshot -s io --format text
agent-terminal snapshot -s net --format text

# Clean up
agent-terminal kill -s cpu
agent-terminal kill -s io
agent-terminal kill -s net
```

### Editor + Preview

Edit a file while watching output:

```bash
# Start editor (--name before command)
agent-terminal spawn --name editor vim main.py

# Start file watcher
agent-terminal spawn --name preview watch -n1 python main.py

# Edit
agent-terminal key -s editor i
agent-terminal type -s editor "print('hello')"
agent-terminal key -s editor Escape
agent-terminal type -s editor ":w"
agent-terminal key -s editor Enter

# Check preview
agent-terminal snapshot -s preview --format text
```

### Pipeline Workflow

Sequential operations across sessions:

```bash
# Setup (--name before command)
agent-terminal spawn --name worker bash

# Run commands
agent-terminal type -s worker "curl -s https://api.example.com > data.json"
agent-terminal key -s worker Enter
agent-terminal wait-for -s worker "$"  # Wait for prompt

agent-terminal type -s worker "jq '.items[]' data.json"
agent-terminal key -s worker Enter
agent-terminal wait-for -s worker "$"

# Get output
agent-terminal snapshot -s worker --format text
```

## Session Isolation

Sessions are fully isolated:
- Separate PTY file descriptors
- Independent screen buffers
- Independent cursor positions
- No shared state between sessions

This means:
- Killing session A doesn't affect session B
- Each session can have different terminal sizes
- Snapshots from one session don't affect others

## Daemon Lifecycle

The daemon manages all sessions:

### Auto-Start

The daemon starts automatically on the first command:

```bash
agent-terminal spawn vim  # Starts daemon if not running
```

### Auto-Stop

After 5 minutes with no active sessions, the daemon shuts down automatically.

### Manual Control

```bash
agent-terminal daemon     # Manually start daemon
agent-terminal stop       # Stop daemon and all sessions
```

## Runtime paths

The daemon resolves its runtime directory in this priority order, then stores per-session socket and PID files there as `{session}.sock` and `{session}.pid`:

1. `$AGENT_TERMINAL_SOCKET_DIR/{session}.sock`
2. `$XDG_RUNTIME_DIR/agent-terminal/{session}.sock`
3. `~/.agent-terminal/{session}.sock`
4. `/tmp/agent-terminal/{session}.sock`

## Environment Variables

| Variable | Description |
|----------|-------------|
| `AGENT_TERMINAL_SESSION` | Default session name for all commands |
| `AGENT_TERMINAL_SOCKET_DIR` | Override socket directory |

Example:

```bash
export AGENT_TERMINAL_SESSION=editor
agent-terminal snapshot  # Targets "editor" session without -s flag
```

## Error Handling

### Session Not Found

```json
{
  "code": "SESSION_NOT_FOUND",
  "message": "Session 'myapp' not found",
  "suggestion": "Run 'agent-terminal list-sessions' to see available sessions"
}
```

### Session Already Exists

Attempting to spawn with a name that's already in use:

```json
{
  "code": "SESSION_EXISTS",
  "message": "Session 'myapp' already exists",
  "suggestion": "Use a different name or kill the existing session first"
}
```

## Best Practices

1. **Put --name before command**: `agent-terminal spawn --name myapp cmd` (not after)
2. **Use meaningful names**: `--name editor` is better than `--name s1`
3. **Clean up when done**: Kill sessions you're finished with
4. **Don't rely on default**: For multi-session work, always name your sessions
5. **Check session exists**: Use `list-sessions` before targeting
6. **Handle process exit**: Sessions auto-cleanup, but check if your command is still running
