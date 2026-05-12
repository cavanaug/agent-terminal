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
fn top_level_help_prefers_wait_over_wait_for() {
    let stdout = run(&["--help"]);

    assert!(
        stdout.contains("\n  wait           Wait for literal text or a regex to appear on screen [aliases: wait-for]"),
        "top-level help should advertise wait with the compatibility alias\n{stdout}"
    );
    assert!(
        !stdout.contains("\n  wait-for"),
        "top-level help should not list wait-for as its own canonical command\n{stdout}"
    );
}

#[test]
fn wait_help_teaches_two_primitives_contract() {
    let stdout = run(&["wait", "--help"]);

    for expected in [
        "Usage: agent-terminal wait [OPTIONS] <PATTERN>",
        "Simple sync:",
        "Use agent-terminal wait for literal text or regex polling",
        "Advanced terminal-state sync:",
        "snapshot --await-change <content_hash> --settle <ms>",
        "agent-terminal wait 'Ready'",
        "agent-terminal wait -r 'error|warning'",
        "agent-terminal wait -t 5000 'Done'",
        "agent-terminal wait -s editor '~'",
        "agent-terminal wait-for ... remains supported as a compatibility alias",
        "agent-terminal wait-for 'Ready'",
    ] {
        assert!(
            stdout.contains(expected),
            "wait help should include {expected:?}\n{stdout}"
        );
    }
}

#[test]
fn wait_for_alias_help_routes_to_wait_contract() {
    let stdout = run(&["wait-for", "--help"]);

    assert!(
        stdout.contains("Usage: agent-terminal wait [OPTIONS] <PATTERN>"),
        "wait-for alias help should render the canonical wait usage\n{stdout}"
    );
    assert!(
        stdout.contains("agent-terminal wait 'Ready'"),
        "wait-for alias help should inherit preferred wait examples\n{stdout}"
    );
    assert!(
        stdout.contains("snapshot --await-change <content_hash> --settle <ms>"),
        "wait-for alias help should keep the advanced snapshot guidance\n{stdout}"
    );
    assert!(
        !stdout.contains("Usage: agent-terminal wait-for [OPTIONS] <PATTERN>"),
        "wait-for alias help should not invent a separate usage contract\n{stdout}"
    );
}

#[test]
fn examples_command_teaches_wait_then_snapshot_settle() {
    let stdout = run(&["examples"]);

    for expected in [
        "agent-terminal wait -s shell \"agent-terminal> \"",
        "agent-terminal snapshot -s shell --await-change \"$HASH\" --settle 100",
        "agent-terminal wait-for ... still work",
        "For simple text or regex polling, prefer agent-terminal wait.",
        "For advanced terminal-state synchronization, prefer agent-terminal snapshot --await-change ... --settle ... .",
    ] {
        assert!(
            stdout.contains(expected),
            "examples output should include {expected:?}\n{stdout}"
        );
    }

    assert!(
        !stdout.contains("agent-terminal wait-for -s shell"),
        "examples output should not teach wait-for as the primary surface\n{stdout}"
    );
}

#[test]
fn readme_prefers_wait_contract_and_keeps_snapshot_settle_explicit() {
    let readme = readme();

    for expected in [
        "- **Keyboard-first interaction**: Drive TUIs with `press`, `type`, `wait`, `scroll`, and `click` commands.",
        "agent-terminal wait \"Ready\"",
        "agent-terminal wait \"Error\" --regex",
        "agent-terminal wait \"Done\" -t 5000",
        "agent-terminal wait -s shell \"agent-terminal> \"",
        "2. `agent-terminal wait -s shell \"agent-terminal> \"` - Wait for the prompt before sending input.",
        "`agent-terminal wait-for ...` remains available as a compatibility alias for existing scripts.",
        "Use `agent-terminal snapshot --await-change <content_hash> --settle <ms>` when you need to wait for the screen to both change and stabilize.",
    ] {
        assert!(
            readme.contains(expected),
            "README should include {expected:?}\n{readme}"
        );
    }

    for unexpected in [
        "agent-terminal wait-for \"Ready\"",
        "agent-terminal wait-for \"Error\" --regex",
        "agent-terminal wait-for \"Done\" -t 5000",
        "agent-terminal wait-for -s shell \"agent-terminal> \"",
        "4. `agent-terminal wait-for` or `snapshot --await-change` - Synchronize instead of sleeping",
    ] {
        assert!(
            !readme.contains(unexpected),
            "README should not teach wait-for-first wording like {unexpected:?}\n{readme}"
        );
    }
}
