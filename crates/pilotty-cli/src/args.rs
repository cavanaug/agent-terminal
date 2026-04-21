//! CLI argument parsing with clap derive macros.

use clap::{Parser, Subcommand, ValueEnum};

const SESSION_HELP: &str = "Target session by name or ID [default: default]";

/// Terminal automation for AI agents.
///
/// Spawn TUI applications in managed PTY sessions and interact with them
/// programmatically. Designed for AI agent consumption with structured
/// JSON output and stable element references.
#[derive(Debug, Parser)]
#[command(name = "agent-terminal", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Spawn a new TUI application in a managed PTY session
    #[command(after_help = "\
Examples:
  agent-terminal spawn htop                    # Simple command
  agent-terminal spawn vim file.txt            # Command with arguments
  agent-terminal spawn --name editor vim       # Named session for easy reference
  agent-terminal spawn --cwd /tmp bash         # Start bash in /tmp directory
  agent-terminal spawn bash -c 'echo hello'    # Shell command with args")]
    Spawn(SpawnArgs),

    /// Kill a session and its child process
    Kill(KillArgs),

    /// Get a snapshot of the terminal screen
    #[command(after_help = "\
Examples:
  agent-terminal snapshot                      # Snapshot default session (full JSON)
  agent-terminal snapshot --format compact     # JSON without text field
  agent-terminal snapshot --format text        # Plain text with cursor indicator
  agent-terminal snapshot -s editor            # Snapshot a specific session

Wait for change:
  HASH=$(agent-terminal snapshot | jq -r '.content_hash')
  agent-terminal press Enter
  agent-terminal snapshot --await-change $HASH           # Block until screen changes
  agent-terminal snapshot --await-change $HASH --settle 100  # Wait for 100ms stability")]
    Snapshot(SnapshotArgs),

    /// Type text at the current cursor position
    #[command(
        name = "type",
        after_help = "\
Examples:
  agent-terminal type 'Hello, world!'          # Type literal text
  agent-terminal type \"line1\\nline2\"          # Type with newline (shell escaping)
  agent-terminal type -s editor ':wq'          # Type in a specific session"
    )]
    Type(TypeArgs),

    /// Send a key, key combination, or key sequence
    #[command(name = "press", visible_alias = "key", after_long_help = "\
Supported Keys:
  Navigation:  Enter, Tab, Escape, Backspace, Space, Delete, Insert
  Arrows:      ArrowUp, ArrowDown, ArrowLeft, ArrowRight, Home, End, PageUp, PageDown
  Function:    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12
  Modifiers:   Control+<key>, Meta+<key>, Option+<key>, Shift+<key>
  Preferred:   Use agent-terminal press with Control+<key>, Meta+<key>, Option+<key>, and Arrow...
  Compatibility: key ..., Ctrl+<key>, Alt+<key>, and short arrows like Up still work

Key Sequences:
  Space-separated keys are sent in order. Useful for chords like Emacs Control+X m.

Examples:
  agent-terminal press Enter                   # Press enter
  agent-terminal press Control+C               # Send interrupt signal
  agent-terminal press Meta+F                  # Meta+F (often opens File menu)
  agent-terminal press Option+F                # Option+F on macOS terminals
  agent-terminal press ArrowUp                 # Arrow key using preferred spelling
  agent-terminal press \"Control+X m\"          # Emacs chord: Control+X then m
  agent-terminal press \"Escape : w q Enter\"    # vim :wq sequence
  agent-terminal press -s editor Escape        # Send Escape to specific session
  agent-terminal press \"a b c\" --delay 50      # Send a, b, c with 50ms delay between")]
    Key(KeyArgs),

    /// Click at a specific row and column coordinate
    #[command(after_help = "\
Click at a specific position in the terminal using 0-indexed coordinates.
Use 'agent-terminal snapshot' to see cursor position and terminal dimensions.

Examples:
  agent-terminal click 10 5                    # Click at row 10, column 5
  agent-terminal click -s editor 5 20          # Click in a specific session")]
    Click(ClickArgs),

    /// Scroll the terminal up or down
    Scroll(ScrollArgs),

    /// List all active sessions
    ListSessions,

    /// Resize the terminal
    Resize(ResizeArgs),

    /// Wait for text to appear on screen
    #[command(after_help = "\
Examples:
  agent-terminal wait-for 'Ready'              # Wait for literal text
  agent-terminal wait-for -r 'error|warning'   # Wait for regex pattern
  agent-terminal wait-for -t 5000 'Done'       # Wait up to 5 seconds
  agent-terminal wait-for -s editor '~'        # Wait in specific session")]
    WaitFor(WaitForArgs),

    /// Show an end-to-end usage example
    Examples,

    /// Generate shell completions
    #[command(after_help = "\
Examples:
  agent-terminal completions bash > ~/.local/share/bash-completion/completions/agent-terminal
  agent-terminal completions zsh > ~/.zfunc/_agent-terminal
  agent-terminal completions fish > ~/.config/fish/completions/agent-terminal.fish")]
    Completions(CompletionsArgs),

    /// Start the daemon process (usually auto-started)
    Daemon,

    /// Stop the daemon process
    Stop,
}

#[derive(Debug, clap::Args)]
pub struct SpawnArgs {
    /// Command and arguments to run (e.g., vim, htop, bash)
    #[arg(
        required = true,
        num_args = 1..,
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub command: Vec<String>,

    /// Give this session a human-readable name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Working directory for the spawned process [default: current directory]
    #[arg(long, value_name = "DIR")]
    pub cwd: Option<String>,

    /// Terminal type advertised to the spawned process [default: 256color]
    ///
    /// Controls what terminal capabilities the application sees.
    /// Sets both TERM and COLORTERM environment variables appropriately.
    #[arg(long, value_enum, default_value_t = XtermVariant::C256color)]
    pub xterm: XtermVariant,

    /// Terminal size as COLSxROWS (e.g. 120x60) [default: 80x24]
    #[arg(long, value_name = "COLSxROWS", value_parser = parse_geometry)]
    pub geometry: Option<(u16, u16)>,
}

/// Parse a geometry string like "120x60" into (cols, rows).
fn parse_geometry(s: &str) -> Result<(u16, u16), String> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() != 2 {
        return Err(format!(
            "expected COLSxROWS format (e.g. 120x60), got '{s}'"
        ));
    }
    let cols: u16 = parts[0]
        .parse()
        .map_err(|_| format!("invalid columns: '{}'", parts[0]))?;
    let rows: u16 = parts[1]
        .parse()
        .map_err(|_| format!("invalid rows: '{}'", parts[1]))?;
    if cols == 0 || rows == 0 {
        return Err("columns and rows must be greater than 0".to_string());
    }
    if cols > 500 {
        return Err(format!("columns too large: {cols} (max 500)"));
    }
    if rows > 500 {
        return Err(format!("rows too large: {rows} (max 500)"));
    }
    Ok((cols, rows))
}

/// Xterm terminal type variants controlling TERM and COLORTERM env vars.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum XtermVariant {
    /// No color (TERM=xterm-mono)
    Mono,
    /// 8 colors (TERM=xterm)
    Basic,
    /// 16 colors (TERM=xterm-16color)
    #[value(name = "16color")]
    C16color,
    /// 256 indexed colors (TERM=xterm-256color)
    #[value(name = "256color")]
    C256color,
    /// 24-bit truecolor (TERM=xterm-direct, COLORTERM=truecolor)
    Direct,
}

impl XtermVariant {
    /// TERM environment variable value for this variant.
    pub fn term_value(self) -> &'static str {
        match self {
            Self::Mono => "xterm-mono",
            Self::Basic => "xterm",
            Self::C16color => "xterm-16color",
            Self::C256color => "xterm-256color",
            Self::Direct => "xterm-direct",
        }
    }

    /// COLORTERM environment variable value, if applicable.
    pub fn colorterm_value(self) -> Option<&'static str> {
        match self {
            Self::Direct => Some("truecolor"),
            _ => None,
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct KillArgs {
    /// Target session by name or ID [default: default]
    #[arg(short, long, help = SESSION_HELP)]
    pub session: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct SnapshotArgs {
    /// Output format
    #[arg(short, long, value_enum, default_value_t = SnapshotFormat::Full)]
    pub format: SnapshotFormat,

    #[arg(short, long, help = SESSION_HELP)]
    pub session: Option<String>,

    /// Render mode for this snapshot: basic (text only), styled (text attributes), color (full color) [default: color]
    #[arg(long = "render", value_enum, default_value_t = CliRenderMode::Color)]
    pub render_mode: CliRenderMode,

    /// Block until content_hash differs from this value
    #[arg(long, value_name = "HASH")]
    pub await_change: Option<u64>,

    /// Wait for screen to stabilize for this many ms before returning
    #[arg(long, default_value_t = 0, value_name = "MS")]
    pub settle: u64,

    /// Total timeout in milliseconds for await-change and settle combined (default: 30s)
    #[arg(short, long, default_value_t = 30000)]
    pub timeout: u64,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SnapshotFormat {
    /// Full JSON with all metadata
    Full,
    /// Compact format with inline refs
    Compact,
    /// Plain text only
    Text,
}

/// Render mode for CLI (maps to protocol RenderMode).
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliRenderMode {
    /// No style data — text only
    Basic,
    /// Text attributes (bold, italic, dim, underline, inverse) via style_map
    Styled,
    /// Full style + color data via style_map + color_map
    Color,
}

#[derive(Debug, clap::Args)]
pub struct TypeArgs {
    /// Text to type
    pub text: String,

    #[arg(short, long, help = SESSION_HELP)]
    pub session: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct KeyArgs {
    /// Key, combo, or sequence to send (e.g., Enter, Control+C, "Control+X m")
    pub key: String,

    /// Delay between keys in a sequence (milliseconds, max 10000)
    #[arg(long, default_value_t = 0)]
    pub delay: u32,

    #[arg(short, long, help = SESSION_HELP)]
    pub session: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct ClickArgs {
    /// Row coordinate (0-indexed)
    pub row: u16,

    /// Column coordinate (0-indexed)
    pub col: u16,

    #[arg(short, long, help = SESSION_HELP)]
    pub session: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct ScrollArgs {
    /// Direction to scroll
    #[arg(value_enum)]
    pub direction: ScrollDirection,

    /// Number of lines to scroll
    #[arg(default_value_t = 1)]
    pub amount: u32,

    #[arg(short, long, help = SESSION_HELP)]
    pub session: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ScrollDirection {
    Up,
    Down,
}

#[derive(Debug, clap::Args)]
pub struct ResizeArgs {
    /// Number of columns
    pub cols: u16,

    /// Number of rows
    pub rows: u16,

    #[arg(short, long, help = SESSION_HELP)]
    pub session: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct WaitForArgs {
    /// Text or regex pattern to wait for
    pub pattern: String,

    /// Timeout in milliseconds
    #[arg(short, long, default_value_t = 30000)]
    pub timeout: u64,

    /// Treat pattern as regex
    #[arg(short, long)]
    pub regex: bool,

    #[arg(short, long, help = SESSION_HELP)]
    pub session: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: clap_complete::Shell,
}

/// End-to-end example text for the `examples` command.
pub const EXAMPLES_TEXT: &str = r#"End-to-end example: Create a file with vi

This example spawns vi, writes text to a file, saves, and exits.

# 1. Spawn vi to create a new file
agent-terminal spawn --name editor vi /tmp/hello.txt

# 2. Wait for vi to start
agent-terminal wait-for -s editor "hello.txt"

# 3. Press 'i' to enter insert mode
agent-terminal press -s editor i

# 4. Type some text
agent-terminal type -s editor "Hello from agent-terminal!"

# 5. Press Escape to return to normal mode
agent-terminal press -s editor Escape

# 6. Save and quit with :wq
agent-terminal type -s editor ":wq"
agent-terminal press -s editor Enter

# 7. Verify the session ended (vi exited)
agent-terminal list-sessions

# The file /tmp/hello.txt now contains "Hello from agent-terminal!"

Compatibility spellings: agent-terminal key ..., Ctrl+..., Alt+..., and short arrows like Up still work.
For new docs and scripts, prefer agent-terminal press with Control+..., Meta+..., Option+..., and Arrow... spellings.
"#;

#[cfg(test)]
mod tests {
    use super::{Cli, CliRenderMode, Commands};
    use clap::Parser;

    #[test]
    fn test_spawn_parses_hyphenated_args() {
        let cli = Cli::parse_from(["agent-terminal", "spawn", "bash", "-c", "echo hello"]);

        match cli.command {
            Commands::Spawn(args) => {
                assert_eq!(args.command, vec!["bash", "-c", "echo hello"]);
            }
            _ => panic!("Expected spawn command"),
        }
    }

    #[test]
    fn test_snapshot_defaults_to_color_render_mode() {
        let cli = Cli::parse_from(["agent-terminal", "snapshot"]);

        match cli.command {
            Commands::Snapshot(args) => {
                assert!(matches!(args.render_mode, CliRenderMode::Color));
            }
            _ => panic!("Expected snapshot command"),
        }
    }

    #[test]
    fn press_alias_parses_preferred_press_command() {
        let cli = Cli::try_parse_from(["agent-terminal", "press", "Control+C"])
            .expect("press should parse as the preferred key verb");

        match cli.command {
            Commands::Key(args) => {
                assert_eq!(args.key, "Control+C");
                assert_eq!(args.delay, 0);
                assert_eq!(args.session, None);
            }
            _ => panic!("Expected key command"),
        }
    }

    #[test]
    fn press_alias_keeps_key_compatibility_alias() {
        let cli = Cli::try_parse_from(["agent-terminal", "key", "Ctrl+C"])
            .expect("key alias should remain supported");

        match cli.command {
            Commands::Key(args) => {
                assert_eq!(args.key, "Ctrl+C");
                assert_eq!(args.delay, 0);
                assert_eq!(args.session, None);
            }
            _ => panic!("Expected key command"),
        }
    }

    #[test]
    fn press_alias_supports_session_and_delay_on_preferred_command() {
        let cli = Cli::try_parse_from([
            "agent-terminal",
            "press",
            "-s",
            "editor",
            "--delay",
            "50",
            "ArrowUp",
        ])
        .expect("press should accept session targeting and delay options");

        match cli.command {
            Commands::Key(args) => {
                assert_eq!(args.key, "ArrowUp");
                assert_eq!(args.delay, 50);
                assert_eq!(args.session.as_deref(), Some("editor"));
            }
            _ => panic!("Expected key command"),
        }
    }
}
