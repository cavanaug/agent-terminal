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
Output format (--format):
  ansi  ANSI-escaped text with color/style — renders in terminal (default)
  json  Structured JSON with 'rows' array — designed for LLM/agent consumption

Render features (--render):
  Comma-separated list of: text, style, color (default: text,style,color)
  text   Plain text content (always included)
  style  Text attributes: bold, italic, dim, underline, inverse
  color  Foreground and background colors

Token budget examples for LLM/agent use:
  --render text              Plain text only (minimum tokens)
  --render text,color        Text + colors, no bold/italic info
  --render text,style,color  Full data (default)

Examples:
  agent-terminal snapshot                              # ANSI output (default)
  agent-terminal snapshot --format json               # JSON rows for agent consumption
  agent-terminal snapshot --format json --render text # Minimal tokens: text only
  agent-terminal snapshot -s editor                   # Snapshot a specific session

Wait for change:
  HASH=$(agent-terminal snapshot --format json | jq -r '.content_hash')
  agent-terminal press Enter
  agent-terminal snapshot --await-change $HASH           # Block until screen changes
  agent-terminal snapshot --await-change $HASH --settle 100  # Wait for 100ms stability

JQ examples (--format json):
  agent-terminal snapshot --format json | jq '.rows[] | select(.r == 5)'
  agent-terminal snapshot --format json | jq '.rows[] | select(.t | contains(\"error\"))'
  agent-terminal snapshot --format json | jq '[.rows[].t]'  # All row text as array")]
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
    #[command(
        name = "press",
        visible_alias = "key",
        after_long_help = "\
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
  agent-terminal press \"a b c\" --delay 50      # Send a, b, c with 50ms delay between"
    )]
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

    /// Wait for literal text or a regex to appear on screen
    #[command(
        name = "wait",
        visible_alias = "wait-for",
        after_long_help = "\
Simple sync:
  Use agent-terminal wait for literal text or regex polling when you just need to
  know whether terminal output has appeared.

Advanced terminal-state sync:
  Use agent-terminal snapshot --await-change <content_hash> --settle <ms> when you
  need to wait for the screen to both change and stabilize before continuing.

Examples:
  agent-terminal wait 'Ready'                  # Wait for literal text
  agent-terminal wait -r 'error|warning'       # Wait for regex pattern
  agent-terminal wait -t 5000 'Done'           # Wait up to 5 seconds
  agent-terminal wait -s editor '~'            # Wait in specific session

Compatibility:
  agent-terminal wait-for ... remains supported as a compatibility alias
  agent-terminal wait-for 'Ready'             # Compatibility alias for wait"
    )]
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
    /// Output format: 'ansi' for terminal display, 'json' for LLM/agent consumption
    #[arg(short, long, value_enum, default_value_t = SnapshotFormat::Ansi)]
    pub format: SnapshotFormat,

    #[arg(short, long, help = SESSION_HELP)]
    pub session: Option<String>,

    /// Features to include in output (comma-separated: text, style, color)
    ///
    /// Controls what data is included — use a subset to reduce token usage for LLM agents.
    /// Examples: 'text' (plain text only), 'text,color' (text + colors), 'text,style,color' (all)
    #[arg(long = "render", default_value = "text,style,color", value_name = "FEATURES")]
    pub render: String,

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
    /// ANSI-escaped text output — renders with color/style in a terminal (default)
    Ansi,
    /// Structured JSON with a 'rows' array — designed for LLM/agent consumption
    Json,
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
pub const EXAMPLES_TEXT: &str = r#"End-to-end example: Run one deterministic shell session

This example starts an interactive shell with an explicit prompt, runs one command,
waits for the visible terminal change to settle, reads the result, and cleans up.

# 1. Spawn a named shell session with a deterministic prompt
agent-terminal spawn --name shell env PS1='agent-terminal> ' bash --noprofile --norc -i

# 2. Wait for the shell prompt before sending input
agent-terminal wait -s shell "agent-terminal> "

# 3. Capture a baseline hash before triggering visible output
HASH=$(agent-terminal snapshot -s shell --format json | jq -r '.content_hash')

# 4. Type a command, then press Enter to run it
agent-terminal type -s shell "printf 'hello from agent-terminal\n'"
agent-terminal press -s shell Enter

# 5. Wait for the screen to change and settle, then read terminal state
agent-terminal snapshot -s shell --await-change "$HASH" --settle 100
agent-terminal snapshot -s shell --format json            # Full JSON for agent
agent-terminal snapshot -s shell                          # ANSI for human viewing

# 6. Minimal-token snapshot (text only, no color/style) for LLM agents
agent-terminal snapshot -s shell --format json --render text

# 7. Clean up the session and stop the daemon when you are done
agent-terminal kill -s shell
agent-terminal stop

Compatibility spellings: agent-terminal key ..., Ctrl+..., Alt+..., short arrows like Up, and agent-terminal wait-for ... still work.
For new docs and scripts, prefer agent-terminal press with Control+..., Meta+..., Option+..., and Arrow... spellings.
For simple text or regex polling, prefer agent-terminal wait.
For advanced terminal-state synchronization, prefer agent-terminal snapshot --await-change ... --settle ... .
"#;

#[cfg(test)]
mod tests {
    use super::{Cli, Commands, EXAMPLES_TEXT};
    use clap::{CommandFactory, Parser};

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
    fn test_snapshot_defaults_to_ansi_format() {
        let cli = Cli::parse_from(["agent-terminal", "snapshot"]);

        match cli.command {
            Commands::Snapshot(args) => {
                assert!(matches!(args.format, super::SnapshotFormat::Ansi));
            }
            _ => panic!("Expected snapshot command"),
        }
    }

    #[test]
    fn test_snapshot_json_format() {
        let cli = Cli::parse_from(["agent-terminal", "snapshot", "--format", "json"]);

        match cli.command {
            Commands::Snapshot(args) => {
                assert!(matches!(args.format, super::SnapshotFormat::Json));
            }
            _ => panic!("Expected snapshot command"),
        }
    }

    #[test]
    fn test_snapshot_default_render_is_full() {
        let cli = Cli::parse_from(["agent-terminal", "snapshot"]);

        match cli.command {
            Commands::Snapshot(args) => {
                assert_eq!(args.render, "text,style,color");
            }
            _ => panic!("Expected snapshot command"),
        }
    }

    #[test]
    fn test_snapshot_render_text_only() {
        let cli = Cli::parse_from(["agent-terminal", "snapshot", "--render", "text"]);

        match cli.command {
            Commands::Snapshot(args) => {
                assert_eq!(args.render, "text");
            }
            _ => panic!("Expected snapshot command"),
        }
    }

    #[test]
    fn test_snapshot_render_text_and_color() {
        let cli = Cli::parse_from(["agent-terminal", "snapshot", "--render", "text,color"]);

        match cli.command {
            Commands::Snapshot(args) => {
                assert_eq!(args.render, "text,color");
            }
            _ => panic!("Expected snapshot command"),
        }
    }

    #[test]
    fn wait_alias_parses_preferred_wait_command() {
        let cli = Cli::try_parse_from(["agent-terminal", "wait", "Ready"])
            .expect("wait should parse as the preferred simple sync verb");

        match cli.command {
            Commands::WaitFor(args) => {
                assert_eq!(args.pattern, "Ready");
                assert_eq!(args.timeout, 30000);
                assert!(!args.regex);
                assert_eq!(args.session, None);
            }
            _ => panic!("Expected wait-for command mapping"),
        }
    }

    #[test]
    fn wait_alias_keeps_wait_for_compatibility_alias() {
        let cli = Cli::try_parse_from(["agent-terminal", "wait-for", "Ready"])
            .expect("wait-for alias should remain supported");

        match cli.command {
            Commands::WaitFor(args) => {
                assert_eq!(args.pattern, "Ready");
                assert_eq!(args.timeout, 30000);
                assert!(!args.regex);
                assert_eq!(args.session, None);
            }
            _ => panic!("Expected wait-for command mapping"),
        }
    }

    #[test]
    fn wait_alias_supports_session_timeout_and_regex_on_preferred_command() {
        let cli = Cli::try_parse_from([
            "agent-terminal",
            "wait",
            "-s",
            "editor",
            "--timeout",
            "5000",
            "--regex",
            "error|warning",
        ])
        .expect("wait should accept session targeting, timeout, and regex options");

        match cli.command {
            Commands::WaitFor(args) => {
                assert_eq!(args.pattern, "error|warning");
                assert_eq!(args.timeout, 5000);
                assert!(args.regex);
                assert_eq!(args.session.as_deref(), Some("editor"));
            }
            _ => panic!("Expected wait-for command mapping"),
        }
    }

    #[test]
    fn wait_help_teaches_simple_wait_and_advanced_snapshot_settle() {
        let mut command = Cli::command();
        let wait = command
            .find_subcommand_mut("wait")
            .expect("wait should be the canonical subcommand");
        let mut help = Vec::new();
        wait.write_long_help(&mut help)
            .expect("wait help should render");
        let help = String::from_utf8(help).expect("wait help should be valid UTF-8");

        assert!(help.contains("agent-terminal wait 'Ready'"));
        assert!(help.contains("agent-terminal wait-for 'Ready'"));
        assert!(help.contains("snapshot --await-change"));
        assert!(help.contains("--settle"));
    }

    #[test]
    fn wait_examples_teach_wait_first_and_snapshot_settle_for_advanced_sync() {
        assert!(EXAMPLES_TEXT.contains("agent-terminal wait -s shell \"agent-terminal> \""));
        assert!(
            EXAMPLES_TEXT.contains("For simple text or regex polling, prefer agent-terminal wait")
        );
        assert!(EXAMPLES_TEXT.contains("snapshot --await-change"));
        assert!(EXAMPLES_TEXT.contains("--settle"));
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
