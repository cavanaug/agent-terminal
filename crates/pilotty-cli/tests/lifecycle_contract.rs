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

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_file(relative: &str) -> String {
    let path = repo_root().join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

#[test]
fn examples_command_teaches_one_canonical_shell_lifecycle() {
    let stdout = run(&["examples"]);

    for expected in [
        "End-to-end example: Run one deterministic shell session",
        "agent-terminal spawn --name shell env PS1='agent-terminal> ' bash --noprofile --norc -i",
        "agent-terminal wait -s shell \"agent-terminal> \"",
        "HASH=$(agent-terminal snapshot -s shell --format json | jq -r '.content_hash')",
        "agent-terminal type -s shell \"printf 'hello from agent-terminal\\n'\"",
        "agent-terminal press -s shell Enter",
        "agent-terminal snapshot -s shell --await-change \"$HASH\" --settle 100",
        "agent-terminal snapshot -s shell --format json",
        "agent-terminal kill -s shell",
        "agent-terminal stop",
    ] {
        assert!(
            stdout.contains(expected),
            "examples output should include {expected:?}\n{stdout}"
        );
    }

    for unexpected in [
        "agent-terminal spawn --name editor vi /tmp/hello.txt",
        "agent-terminal type -s editor \":wq\"",
        "agent-terminal press -s editor Escape",
    ] {
        assert!(
            !stdout.contains(unexpected),
            "examples output should no longer teach the legacy editor flow {unexpected:?}\n{stdout}"
        );
    }
}

#[test]
fn readme_links_the_same_shell_lifecycle_to_tracked_m009_handoff() {
    let readme = read_file("README.md");

    for expected in [
        "agent-terminal spawn --name shell env PS1='agent-terminal> ' bash --noprofile --norc -i",
        "agent-terminal wait -s shell \"agent-terminal> \"",
        "agent-terminal type -s shell \"printf 'hello from agent-terminal\\n'\"",
        "agent-terminal snapshot -s shell --await-change \"$HASH\" --settle 100",
        "agent-terminal kill -s shell",
        "agent-terminal stop",
        "M009-HANDOFF.md",
    ] {
        assert!(
            readme.contains(expected),
            "README should include {expected:?}\n{readme}"
        );
    }

    assert!(
        !readme.contains("agent-terminal spawn --name editor vi /tmp/hello.txt"),
        "README should not keep the old editor-first lifecycle as the canonical flow\n{readme}"
    );
}

#[test]
fn m009_handoff_records_deferred_grammar_and_distribution_follow_up() {
    let handoff = read_file("M009-HANDOFF.md");

    for expected in [
        "# M009 Handoff",
        "Deferred grammar work",
        "Deferred external distribution / origin-note follow-up",
        "R024 remains intentionally deferred under D018.",
        "Homebrew and related install docs",
        "keep the current runtime and protocol semantics unchanged in M008",
        "Commands::Key",
        "Commands::WaitFor",
    ] {
        assert!(
            handoff.contains(expected),
            "M009 handoff should include {expected:?}\n{handoff}"
        );
    }
}
