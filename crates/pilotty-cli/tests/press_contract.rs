use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn agent_terminal() -> &'static str {
    env!("CARGO_BIN_EXE_agent-terminal")
}

fn run(args: &[&str]) -> String {
    let output = Command::new(agent_terminal())
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run {:?}: {error}", args));

    assert!(
        output.status.success(),
        "command {:?} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("stdout should be valid UTF-8")
}

fn readme() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../README.md");
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

#[test]
fn top_level_help_prefers_press_over_key() {
    let stdout = run(&["--help"]);

    assert!(
        stdout.contains("\n  press"),
        "top-level help should advertise press\n{stdout}"
    );
    assert!(
        !stdout.contains("\n  key            Send a key, key combination, or key sequence"),
        "top-level help should not list key as the canonical command\n{stdout}"
    );
}

#[test]
fn press_help_teaches_preferred_notation_first() {
    let stdout = run(&["press", "--help"]);

    for expected in [
        "Usage: agent-terminal press [OPTIONS] <KEY>",
        "Control+<key>",
        "Meta+<key>",
        "Option+<key>",
        "ArrowUp",
        "agent-terminal press Enter",
        "agent-terminal press Control+C",
        "agent-terminal press Meta+F",
        "agent-terminal press Option+F",
        "agent-terminal press \"Control+X m\"",
        "agent-terminal press -s editor Escape",
    ] {
        assert!(
            stdout.contains(expected),
            "press help should include {expected:?}\n{stdout}"
        );
    }
}

#[test]
fn key_alias_help_routes_to_press_contract() {
    let stdout = run(&["key", "--help"]);

    assert!(
        stdout.contains("Usage: agent-terminal press [OPTIONS] <KEY>"),
        "key alias help should render the canonical press usage\n{stdout}"
    );
    assert!(
        stdout.contains("agent-terminal press Control+C"),
        "key alias help should inherit preferred press examples\n{stdout}"
    );
}

#[test]
fn examples_command_prefers_press_examples() {
    let stdout = run(&["examples"]);

    for expected in [
        "agent-terminal press -s editor i",
        "agent-terminal press -s editor Escape",
        "agent-terminal press -s editor Enter",
        "Compatibility spellings: agent-terminal key ...",
        "Ctrl+...",
        "Alt+...",
        "Up",
    ] {
        assert!(
            stdout.contains(expected),
            "examples output should include {expected:?}\n{stdout}"
        );
    }

    assert!(
        !stdout.contains("agent-terminal key -s editor"),
        "examples output should not teach key as the primary surface\n{stdout}"
    );
}

#[test]
fn readme_prefers_press_contract_and_keeps_compatibility_note() {
    let readme = readme();

    for expected in [
        "agent-terminal press Enter",
        "agent-terminal press Control+C",
        "agent-terminal press ArrowUp",
        "Compatibility spellings",
        "`key`",
        "`Ctrl+...`",
        "`Alt+...`",
        "`Up`",
    ] {
        assert!(
            readme.contains(expected),
            "README should include {expected:?}\n{readme}"
        );
    }

    for unexpected in [
        "agent-terminal key Enter",
        "agent-terminal key Ctrl+C",
        "agent-terminal key Alt+F",
        "agent-terminal key \"Ctrl+X m\"",
    ] {
        assert!(
            !readme.contains(unexpected),
            "README should not teach legacy key-first examples like {unexpected:?}\n{readme}"
        );
    }
}
