# Key Input Reference

Complete reference for key combinations supported by `agent-terminal key`.

## Basic Usage

```bash
agent-terminal key <key>                 # Send single key to default session
agent-terminal key -s myapp <key>        # Send to specific session
agent-terminal key "key1 key2 key3"      # Send key sequence (space-separated)
agent-terminal key "key1 key2" --delay 50  # Sequence with 50ms delay between keys
```

## Named Keys

| Key | Aliases | Description |
|-----|---------|-------------|
| `Enter` | `Return` | Enter/Return key |
| `Tab` | | Tab key |
| `Escape` | `Esc` | Escape key |
| `Space` | | Space bar |
| `Backspace` | | Backspace key |
| `Delete` | `Del` | Delete key |
| `Insert` | `Ins` | Insert key |

## Arrow Keys

| Key | Aliases | Description |
|-----|---------|-------------|
| `Up` | `ArrowUp` | Up arrow |
| `Down` | `ArrowDown` | Down arrow |
| `Left` | `ArrowLeft` | Left arrow |
| `Right` | `ArrowRight` | Right arrow |

## Navigation Keys

| Key | Aliases | Description |
|-----|---------|-------------|
| `Home` | | Home key |
| `End` | | End key |
| `PageUp` | `PgUp` | Page up |
| `PageDown` | `PgDn` | Page down |

## Function Keys

| Key | Description |
|-----|-------------|
| `F1` | Function key 1 |
| `F2` | Function key 2 |
| `F3` | Function key 3 |
| `F4` | Function key 4 |
| `F5` | Function key 5 |
| `F6` | Function key 6 |
| `F7` | Function key 7 |
| `F8` | Function key 8 |
| `F9` | Function key 9 |
| `F10` | Function key 10 |
| `F11` | Function key 11 |
| `F12` | Function key 12 |

## Modifier Combinations

### Ctrl Combinations

| Key | Aliases | Common Use |
|-----|---------|------------|
| `Ctrl+C` | `Control+C` | Interrupt/cancel |
| `Ctrl+D` | | EOF/exit |
| `Ctrl+Z` | | Suspend process |
| `Ctrl+L` | | Clear screen |
| `Ctrl+A` | | Beginning of line (bash, emacs) |
| `Ctrl+E` | | End of line (bash, emacs) |
| `Ctrl+K` | | Kill to end of line |
| `Ctrl+U` | | Kill to beginning of line |
| `Ctrl+W` | | Kill word backward |
| `Ctrl+R` | | Reverse search (bash) |
| `Ctrl+S` | | Save (many apps) |
| `Ctrl+Q` | | Quit (some apps) |
| `Ctrl+X` | | Cut / prefix key |
| `Ctrl+V` | | Paste / literal next |
| `Ctrl+G` | | Cancel (emacs) |
| `Ctrl+O` | | Open (many apps) |
| `Ctrl+N` | | Next / new |
| `Ctrl+P` | | Previous |
| `Ctrl+F` | | Forward / find |
| `Ctrl+B` | | Backward |

### Alt Combinations

| Key | Aliases | Common Use |
|-----|---------|------------|
| `Alt+F` | `Meta+F`, `Option+F` | Forward word / File menu |
| `Alt+B` | | Backward word |
| `Alt+D` | | Delete word forward |
| `Alt+Backspace` | | Delete word backward |
| `Alt+.` | | Last argument (bash) |
| `Alt+Tab` | | (Usually handled by window manager) |

### Shift Combinations

| Key | Description |
|-----|-------------|
| `Shift+Tab` | Reverse tab (previous field) |
| `Shift+Enter` | Shift+Enter (app-specific) |
| `Shift+Up` | Select up (some apps) |
| `Shift+Down` | Select down (some apps) |

### Combined Modifiers

| Key | Description |
|-----|-------------|
| `Ctrl+Alt+C` | Ctrl+Alt+C |
| `Ctrl+Shift+C` | Copy (some terminals) |
| `Ctrl+Shift+V` | Paste (some terminals) |

## Special Characters

| Key | Description |
|-----|-------------|
| `Plus` | Literal `+` character |

## Key Sequences

Send multiple keys in order with a single command. Keys are space-separated:

```bash
# Emacs-style chords
agent-terminal key "Ctrl+X Ctrl+S"       # Save file
agent-terminal key "Ctrl+X Ctrl+C"       # Exit Emacs
agent-terminal key "Ctrl+X m"            # Compose mail

# vim command sequences
agent-terminal key "Escape : w q Enter"  # Save and quit
agent-terminal key "Escape : q ! Enter"  # Quit without saving
agent-terminal key "g g d G"             # Delete entire file

# Navigation sequences
agent-terminal key "Tab Tab Enter"       # Tab twice then Enter
agent-terminal key "Down Down Space"     # Move down twice and select
```

### Inter-key Delay

Use `--delay` for TUIs that need time between keys:

```bash
agent-terminal key "Tab Tab Enter" --delay 100   # 100ms between each key
agent-terminal key "F9 Down Enter" --delay 50    # htop kill menu navigation
```

| Option | Description |
|--------|-------------|
| `--delay <ms>` | Milliseconds between keys (default: 0, max: 10000) |

### When to Use Sequences vs Individual Keys

**Use sequences** for:
- Emacs/vim chords that must be sent together
- Predictable navigation patterns
- Reducing command overhead

**Use individual keys** when:
- You need to check screen state between keys
- Timing is unpredictable
- Different paths based on UI state

## Common TUI Patterns

### Dialog/Whiptail

```bash
agent-terminal key Tab       # Move between buttons
agent-terminal key Enter     # Activate button
agent-terminal key Space     # Toggle checkbox
agent-terminal key Escape    # Cancel dialog
```

### Vim

```bash
agent-terminal key i         # Insert mode (use agent-terminal type for text)
agent-terminal key Escape    # Normal mode
agent-terminal key Ctrl+C    # Also exits insert mode
agent-terminal type ":wq"    # Command (then Enter)
agent-terminal key Enter

# Using sequences for common operations
agent-terminal key "Escape : w q Enter"     # Save and quit
agent-terminal key "Escape : q ! Enter"     # Force quit
agent-terminal key "Escape d d"             # Delete line
agent-terminal key "Escape g g"             # Go to top
```

### Htop

```bash
agent-terminal key F1        # Help
agent-terminal key F2        # Setup
agent-terminal key F5        # Tree view
agent-terminal key F9        # Kill process
agent-terminal key F10       # Quit
agent-terminal key q         # Also quit
```

### Less/More

```bash
agent-terminal key Space     # Page down
agent-terminal key b         # Page up
agent-terminal key q         # Quit
agent-terminal key /         # Search (then type pattern)
agent-terminal key n         # Next match
agent-terminal key N         # Previous match
```

### Nano

```bash
agent-terminal key Ctrl+O    # Save
agent-terminal key Ctrl+X    # Exit
agent-terminal key Ctrl+K    # Cut line
agent-terminal key Ctrl+U    # Paste
agent-terminal key Ctrl+W    # Search

# Using sequences
agent-terminal key "Ctrl+O Enter"    # Save with default filename
agent-terminal key "Ctrl+X n"        # Exit without saving (answer 'n' to save prompt)
```

### Tmux (default prefix)

```bash
agent-terminal key Ctrl+B    # Prefix key
# Then send the command key:
agent-terminal key c         # New window
agent-terminal key n         # Next window
agent-terminal key p         # Previous window
agent-terminal key d         # Detach

# Using sequences for tmux commands
agent-terminal key "Ctrl+B c"    # Prefix + new window
agent-terminal key "Ctrl+B n"    # Prefix + next window
agent-terminal key "Ctrl+B d"    # Prefix + detach
```

### Readline/Bash

```bash
agent-terminal key Ctrl+A    # Beginning of line
agent-terminal key Ctrl+E    # End of line
agent-terminal key Ctrl+U    # Clear line
agent-terminal key Ctrl+R    # Reverse search
agent-terminal key Ctrl+L    # Clear screen
agent-terminal key Up        # Previous history
agent-terminal key Down      # Next history
```

## Case Sensitivity

- Named keys are case-insensitive: `Enter`, `ENTER`, `enter` all work
- Letter keys with Ctrl/Alt are case-insensitive: `Ctrl+c` = `Ctrl+C`
- Plain letters: Use `agent-terminal type` for text, not `agent-terminal key`

## Escaping

The `+` character is the modifier separator. To type a literal `+`:

```bash
agent-terminal key Plus      # Sends the + character
# Or use type for text:
agent-terminal type "2+2"    # Types "2+2"
```

## Troubleshooting

### Key Not Recognized

```bash
# Check if it's a named key or text
agent-terminal key Enter     # Named key
agent-terminal type "hello"  # Text input
```

### Modifier Not Working

Some apps intercept modifiers before the terminal sees them. Try:

```bash
# Check raw terminal behavior
agent-terminal spawn cat
agent-terminal key Ctrl+C    # Should show ^C or exit
```

### Timing Issues

Some TUIs need time to process input:

```bash
agent-terminal key F9        # Opens menu
agent-terminal wait-for "SIGTERM"  # Wait for menu
agent-terminal key Enter     # Then select
```
