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
        "command {:?} failed with status {:?}\nstderr:\n{}",
        args,
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert!(
        !stdout.trim().is_empty(),
        "command {:?} produced empty stdout",
        args
    );
    stdout
}

fn assert_agent_terminal_surface(surface: &str, stdout: &str) {
    assert!(
        stdout.contains("agent-terminal"),
        "{surface} should mention agent-terminal\n{stdout}"
    );
    assert!(
        !stdout.contains("pilotty"),
        "{surface} should not leak pilotty\n{stdout}"
    );
    assert!(
        !stdout.contains("_pilotty"),
        "{surface} should not leak _pilotty\n{stdout}"
    );
    assert!(
        !stdout.contains("#compdef pilotty"),
        "{surface} should not leak zsh compdef pilotty\n{stdout}"
    );
}

#[test]
fn help_and_example_surfaces_use_agent_terminal_identity() {
    for (surface, args) in [
        ("top-level help", &["--help"] as &[&str]),
        ("spawn help", &["spawn", "--help"]),
        ("wait-for help", &["wait-for", "--help"]),
        ("examples command", &["examples"]),
    ] {
        let stdout = run(args);
        assert_agent_terminal_surface(surface, &stdout);
    }
}

#[test]
fn bash_and_zsh_completions_do_not_leak_legacy_identity() {
    for shell in ["bash", "zsh"] {
        let stdout = run(&["completions", shell]);
        assert_agent_terminal_surface(shell, &stdout);
    }
}
