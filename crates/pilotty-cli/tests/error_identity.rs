use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn agent_terminal() -> &'static str {
    env!("CARGO_BIN_EXE_agent-terminal")
}

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    format!("{}-{nanos}", std::process::id())
}

struct RuntimeSandbox {
    root: PathBuf,
    home: PathBuf,
    session_name: String,
}

impl RuntimeSandbox {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("agent-terminal-error-{}", unique_suffix()));
        let home = root.join("home");
        fs::create_dir_all(&home).expect("failed to create isolated HOME");

        Self {
            root,
            home,
            session_name: format!("stderr-{}", unique_suffix()),
        }
    }

    fn runtime_dir(&self) -> PathBuf {
        self.home.join(".agent-terminal")
    }

    fn socket_path(&self) -> PathBuf {
        self.runtime_dir().join("default.sock")
    }

    fn pid_path(&self) -> PathBuf {
        self.runtime_dir().join("default.pid")
    }

    fn command(&self) -> Command {
        let mut command = Command::new(agent_terminal());
        command
            .env_remove("AGENT_TERMINAL_SESSION")
            .env_remove("AGENT_TERMINAL_SOCKET_DIR")
            .env("HOME", &self.home)
            .env("XDG_RUNTIME_DIR", "")
            .env("RUST_LOG", "error");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command()
            .args(args)
            .output()
            .unwrap_or_else(|error| panic!("failed to run {:?}: {error}", args))
    }

    fn wait_for_absent(&self, path: &Path, label: &str) {
        for _ in 0..50 {
            if !path.exists() {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        panic!("timed out waiting for {label} to disappear: {}", path.display());
    }

    fn cleanup(&self) {
        if self.socket_path().exists() {
            let _ = self.run(&["kill", "-s", &self.session_name]);
            let _ = self.run(&["stop"]);
            self.wait_for_absent(&self.socket_path(), "daemon socket");
            self.wait_for_absent(&self.pid_path(), "daemon pid file");
        }

        let _ = fs::remove_dir_all(&self.root);
    }
}

impl Drop for RuntimeSandbox {
    fn drop(&mut self) {
        self.cleanup();
    }
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be valid UTF-8")
}

fn assert_failure_contains(output: &Output, expected_code: &str, expected_hint: &str) {
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = stderr(output);
    assert!(
        stderr.contains(expected_code),
        "stderr should include {expected_code}\n{stderr}"
    );
    assert!(
        stderr.contains(expected_hint),
        "stderr should include {expected_hint}\n{stderr}"
    );
    assert!(
        stderr.contains("agent-terminal"),
        "stderr should mention agent-terminal\n{stderr}"
    );
    assert!(
        !stderr.contains("pilotty"),
        "stderr should not leak pilotty\n{stderr}"
    );
}

#[test]
fn missing_session_stderr_uses_agent_terminal_identity() {
    let sandbox = RuntimeSandbox::new();
    let missing_session = format!("missing-{}", unique_suffix());

    let output = sandbox.run(&["snapshot", "-s", &missing_session, "--format", "text"]);

    assert_failure_contains(
        &output,
        "[SESSION_NOT_FOUND]",
        "Run 'agent-terminal list-sessions' to see available sessions",
    );
}

#[test]
fn invalid_key_stderr_uses_agent_terminal_identity() {
    let sandbox = RuntimeSandbox::new();

    let spawn = sandbox.run(&[
        "spawn",
        "--name",
        &sandbox.session_name,
        "bash",
        "-lc",
        "echo READY; sleep 60",
    ]);
    assert!(
        spawn.status.success(),
        "spawn failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&spawn.stdout),
        String::from_utf8_lossy(&spawn.stderr)
    );
    let spawn_stdout = String::from_utf8(spawn.stdout).expect("spawn stdout should be UTF-8");
    assert!(
        spawn_stdout.contains("session_created"),
        "spawn should create a session\n{spawn_stdout}"
    );

    let wait = sandbox.run(&["wait-for", "-s", &sandbox.session_name, "-t", "10000", "READY"]);
    assert!(
        wait.status.success(),
        "wait-for failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&wait.stdout),
        String::from_utf8_lossy(&wait.stderr)
    );
    let wait_stdout = String::from_utf8(wait.stdout).expect("wait-for stdout should be UTF-8");
    assert!(
        wait_stdout.contains("\"found\": true"),
        "wait-for should confirm READY\n{wait_stdout}"
    );

    let invalid_key = sandbox.run(&["key", "-s", &sandbox.session_name, "DefinitelyNotAKey"]);

    assert_failure_contains(
        &invalid_key,
        "[INVALID_INPUT]",
        "Run 'agent-terminal key --help' for examples.",
    );
}
