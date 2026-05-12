//! Socket and PID file path resolution.
//!
//! Priority for the runtime directory:
//! 1. `AGENT_TERMINAL_SOCKET_DIR` (explicit override)
//! 2. `XDG_RUNTIME_DIR/agent-terminal` (Linux standard)
//! 3. `~/.agent-terminal` (home directory fallback)
//! 4. `/tmp/agent-terminal` (last resort)
//!
//! Session support via `AGENT_TERMINAL_SESSION` env var (default: `default`).
//! Each session gets its own socket and PID file under the resolved runtime
//! directory: `{runtime_dir}/{session}.sock` and `{runtime_dir}/{session}.pid`.

use std::env;
use std::path::PathBuf;

const SESSION_ENV_VAR: &str = "AGENT_TERMINAL_SESSION";
const SOCKET_DIR_ENV_VAR: &str = "AGENT_TERMINAL_SOCKET_DIR";
const XDG_RUNTIME_DIR_ENV_VAR: &str = "XDG_RUNTIME_DIR";
const XDG_RUNTIME_SUBDIR: &str = "agent-terminal";
const HOME_RUNTIME_SUBDIR: &str = ".agent-terminal";
const TEMP_RUNTIME_SUBDIR: &str = "agent-terminal";

/// Get current session name from env or default.
pub fn get_session() -> String {
    env::var(SESSION_ENV_VAR).unwrap_or_else(|_| "default".to_string())
}

fn resolve_socket_dir(home_dir: Option<PathBuf>, temp_dir: PathBuf) -> PathBuf {
    // 1. Explicit override (ignore empty)
    if let Ok(dir) = env::var(SOCKET_DIR_ENV_VAR) {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }

    // 2. XDG_RUNTIME_DIR (Linux standard, ignore empty)
    if let Ok(runtime_dir) = env::var(XDG_RUNTIME_DIR_ENV_VAR) {
        if !runtime_dir.is_empty() {
            return PathBuf::from(runtime_dir).join(XDG_RUNTIME_SUBDIR);
        }
    }

    // 3. Home directory fallback
    if let Some(home) = home_dir {
        return home.join(HOME_RUNTIME_SUBDIR);
    }

    // 4. Last resort: temp dir
    temp_dir.join(TEMP_RUNTIME_SUBDIR)
}

/// Get socket directory with priority fallback.
///
/// Priority:
/// 1. `AGENT_TERMINAL_SOCKET_DIR` (explicit override, ignores empty string)
/// 2. `XDG_RUNTIME_DIR/agent-terminal` (Linux standard, ignores empty string)
/// 3. `~/.agent-terminal` (home directory fallback)
/// 4. System temp dir + `agent-terminal` (last resort)
pub fn get_socket_dir() -> PathBuf {
    resolve_socket_dir(dirs::home_dir(), env::temp_dir())
}

/// Validate a session name to prevent path traversal attacks.
///
/// Session names must:
/// - Be non-empty
/// - Contain only alphanumeric characters, hyphens, and underscores
/// - Not start with a hyphen (could be interpreted as option)
///
/// Returns the sanitized name or a safe default if invalid.
pub(crate) fn sanitize_session_name(name: &str) -> String {
    // Check if valid: non-empty, safe chars, doesn't start with hyphen
    let is_valid = !name.is_empty()
        && !name.starts_with('-')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

    if is_valid {
        name.to_string()
    } else {
        // Log warning and use safe fallback
        tracing::warn!(
            "Invalid session name '{}', using 'default'. Names must contain only alphanumeric, hyphen, underscore.",
            name
        );
        "default".to_string()
    }
}

/// Get socket path for a session.
///
/// If no session is provided, uses the current session from `get_session()`.
/// Session names are sanitized to prevent path traversal.
pub fn get_socket_path(session: Option<&str>) -> PathBuf {
    let sess = session.map(String::from).unwrap_or_else(get_session);
    let safe_sess = sanitize_session_name(&sess);
    get_socket_dir().join(format!("{}.sock", safe_sess))
}

/// Get PID file path for a session.
///
/// If no session is provided, uses the current session from `get_session()`.
/// Session names are sanitized to prevent path traversal.
pub fn get_pid_path(session: Option<&str>) -> PathBuf {
    let sess = session.map(String::from).unwrap_or_else(get_session);
    let safe_sess = sanitize_session_name(&sess);
    get_socket_dir().join(format!("{}.pid", safe_sess))
}

/// Ensure socket directory exists with secure permissions (0700 on Unix).
pub fn ensure_socket_dir() -> std::io::Result<()> {
    let dir = get_socket_dir();
    std::fs::create_dir_all(&dir)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Mutex;

    use crate::daemon::paths::{
        get_pid_path, get_session, get_socket_dir, get_socket_path, resolve_socket_dir,
        sanitize_session_name, SESSION_ENV_VAR, SOCKET_DIR_ENV_VAR, XDG_RUNTIME_DIR_ENV_VAR,
    };

    // Mutex to serialize tests that manipulate environment variables.
    // Env var manipulation is inherently non-thread-safe, so tests must run serially.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    // Helper to save and restore env vars during tests.
    // Also holds the mutex guard to ensure serialized access.
    struct EnvGuard {
        vars: Vec<(String, Option<String>)>,
        _lock: std::sync::MutexGuard<'static, ()>,
    }

    impl EnvGuard {
        fn new(var_names: &[&str]) -> Self {
            // Lock first to prevent races. If a prior test panicked while holding the
            // mutex, recover the guard so subsequent runs can still proceed.
            let lock = ENV_MUTEX
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let vars = var_names
                .iter()
                .map(|name| (name.to_string(), std::env::var(name).ok()))
                .collect();
            Self { vars, _lock: lock }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (name, value) in &self.vars {
                // SAFETY: We hold ENV_MUTEX, so no other test thread is modifying env vars.
                unsafe {
                    match value {
                        Some(v) => std::env::set_var(name, v),
                        None => std::env::remove_var(name),
                    }
                }
            }
            // _lock is dropped here, releasing the mutex.
        }
    }

    #[test]
    fn test_get_session_default() {
        let _guard = EnvGuard::new(&[SESSION_ENV_VAR]);
        // SAFETY: We hold ENV_MUTEX via _guard.
        unsafe { std::env::remove_var(SESSION_ENV_VAR) };

        assert_eq!(get_session(), "default");
    }

    #[test]
    fn test_get_session_custom() {
        let _guard = EnvGuard::new(&[SESSION_ENV_VAR]);
        // SAFETY: We hold ENV_MUTEX via _guard.
        unsafe { std::env::set_var(SESSION_ENV_VAR, "my-session") };

        assert_eq!(get_session(), "my-session");
    }

    #[test]
    fn test_get_socket_dir_explicit_override_takes_precedence() {
        let _guard = EnvGuard::new(&[SOCKET_DIR_ENV_VAR, XDG_RUNTIME_DIR_ENV_VAR]);

        // SAFETY: We hold ENV_MUTEX via _guard.
        unsafe {
            std::env::set_var(SOCKET_DIR_ENV_VAR, "/custom/socket/path");
            std::env::set_var(XDG_RUNTIME_DIR_ENV_VAR, "/run/user/1000");
        }

        assert_eq!(get_socket_dir(), PathBuf::from("/custom/socket/path"));
    }

    #[test]
    fn test_get_socket_dir_ignores_empty_override_and_uses_xdg() {
        let _guard = EnvGuard::new(&[SOCKET_DIR_ENV_VAR, XDG_RUNTIME_DIR_ENV_VAR]);

        // SAFETY: We hold ENV_MUTEX via _guard.
        unsafe {
            std::env::set_var(SOCKET_DIR_ENV_VAR, "");
            std::env::set_var(XDG_RUNTIME_DIR_ENV_VAR, "/run/user/1000");
        }

        assert_eq!(
            get_socket_dir(),
            PathBuf::from("/run/user/1000/agent-terminal")
        );
    }

    #[test]
    fn test_get_socket_dir_xdg_runtime() {
        let _guard = EnvGuard::new(&[SOCKET_DIR_ENV_VAR, XDG_RUNTIME_DIR_ENV_VAR]);

        // SAFETY: We hold ENV_MUTEX via _guard.
        unsafe {
            std::env::remove_var(SOCKET_DIR_ENV_VAR);
            std::env::set_var(XDG_RUNTIME_DIR_ENV_VAR, "/run/user/1000");
        }

        assert_eq!(
            get_socket_dir(),
            PathBuf::from("/run/user/1000/agent-terminal")
        );
    }

    #[test]
    fn test_get_socket_dir_ignores_empty_xdg_and_uses_home() {
        let _guard = EnvGuard::new(&[SOCKET_DIR_ENV_VAR, XDG_RUNTIME_DIR_ENV_VAR]);

        // SAFETY: We hold ENV_MUTEX via _guard.
        unsafe {
            std::env::remove_var(SOCKET_DIR_ENV_VAR);
            std::env::set_var(XDG_RUNTIME_DIR_ENV_VAR, "");
        }

        let result = get_socket_dir();
        assert!(result.to_string_lossy().ends_with(".agent-terminal"));
    }

    #[test]
    fn test_get_socket_dir_home_fallback() {
        let _guard = EnvGuard::new(&[SOCKET_DIR_ENV_VAR, XDG_RUNTIME_DIR_ENV_VAR]);

        // SAFETY: We hold ENV_MUTEX via _guard.
        unsafe {
            std::env::remove_var(SOCKET_DIR_ENV_VAR);
            std::env::remove_var(XDG_RUNTIME_DIR_ENV_VAR);
        }

        let result = get_socket_dir();
        assert!(result.to_string_lossy().ends_with(".agent-terminal"));
    }

    #[test]
    fn test_resolve_socket_dir_uses_temp_dir_last_resort() {
        let _guard = EnvGuard::new(&[SOCKET_DIR_ENV_VAR, XDG_RUNTIME_DIR_ENV_VAR]);

        // SAFETY: We hold ENV_MUTEX via _guard.
        unsafe {
            std::env::remove_var(SOCKET_DIR_ENV_VAR);
            std::env::remove_var(XDG_RUNTIME_DIR_ENV_VAR);
        }

        assert_eq!(
            resolve_socket_dir(None, PathBuf::from("/tmp/runtime-fallback")),
            PathBuf::from("/tmp/runtime-fallback/agent-terminal")
        );
    }

    #[test]
    fn test_get_socket_path_default_session() {
        let _guard = EnvGuard::new(&[SOCKET_DIR_ENV_VAR, SESSION_ENV_VAR, XDG_RUNTIME_DIR_ENV_VAR]);

        // SAFETY: We hold ENV_MUTEX via _guard.
        unsafe {
            std::env::set_var(SOCKET_DIR_ENV_VAR, "/tmp/test");
            std::env::remove_var(SESSION_ENV_VAR);
            std::env::remove_var(XDG_RUNTIME_DIR_ENV_VAR);
        }

        assert_eq!(
            get_socket_path(None),
            PathBuf::from("/tmp/test/default.sock")
        );
    }

    #[test]
    fn test_get_socket_path_custom_session() {
        let _guard = EnvGuard::new(&[SOCKET_DIR_ENV_VAR, SESSION_ENV_VAR, XDG_RUNTIME_DIR_ENV_VAR]);

        // SAFETY: We hold ENV_MUTEX via _guard.
        unsafe {
            std::env::set_var(SOCKET_DIR_ENV_VAR, "/tmp/test");
            std::env::remove_var(SESSION_ENV_VAR);
            std::env::remove_var(XDG_RUNTIME_DIR_ENV_VAR);
        }

        assert_eq!(
            get_socket_path(Some("my-session")),
            PathBuf::from("/tmp/test/my-session.sock")
        );
    }

    #[test]
    fn test_get_pid_path_custom_session() {
        let _guard = EnvGuard::new(&[SOCKET_DIR_ENV_VAR, SESSION_ENV_VAR, XDG_RUNTIME_DIR_ENV_VAR]);

        // SAFETY: We hold ENV_MUTEX via _guard.
        unsafe {
            std::env::set_var(SOCKET_DIR_ENV_VAR, "/tmp/test");
            std::env::remove_var(SESSION_ENV_VAR);
            std::env::remove_var(XDG_RUNTIME_DIR_ENV_VAR);
        }

        assert_eq!(
            get_pid_path(Some("my-session")),
            PathBuf::from("/tmp/test/my-session.pid")
        );
    }

    #[test]
    fn test_socket_path_sanitizes_empty_env_session() {
        let _guard = EnvGuard::new(&[SOCKET_DIR_ENV_VAR, SESSION_ENV_VAR, XDG_RUNTIME_DIR_ENV_VAR]);

        // SAFETY: We hold ENV_MUTEX via _guard.
        unsafe {
            std::env::set_var(SOCKET_DIR_ENV_VAR, "/tmp/test");
            std::env::set_var(SESSION_ENV_VAR, "");
            std::env::remove_var(XDG_RUNTIME_DIR_ENV_VAR);
        }

        assert_eq!(
            get_socket_path(None),
            PathBuf::from("/tmp/test/default.sock")
        );
    }

    #[test]
    fn test_socket_path_sanitizes_traversal_session_from_env() {
        let _guard = EnvGuard::new(&[SOCKET_DIR_ENV_VAR, SESSION_ENV_VAR, XDG_RUNTIME_DIR_ENV_VAR]);

        // SAFETY: We hold ENV_MUTEX via _guard.
        unsafe {
            std::env::set_var(SOCKET_DIR_ENV_VAR, "/tmp/test");
            std::env::set_var(SESSION_ENV_VAR, "../../../etc/passwd");
            std::env::remove_var(XDG_RUNTIME_DIR_ENV_VAR);
        }

        assert_eq!(
            get_socket_path(None),
            PathBuf::from("/tmp/test/default.sock")
        );
    }

    #[test]
    fn test_pid_path_sanitizes_traversal_session_argument() {
        let _guard = EnvGuard::new(&[SOCKET_DIR_ENV_VAR, SESSION_ENV_VAR, XDG_RUNTIME_DIR_ENV_VAR]);

        // SAFETY: We hold ENV_MUTEX via _guard.
        unsafe {
            std::env::set_var(SOCKET_DIR_ENV_VAR, "/tmp/test");
            std::env::remove_var(SESSION_ENV_VAR);
            std::env::remove_var(XDG_RUNTIME_DIR_ENV_VAR);
        }

        assert_eq!(
            get_pid_path(Some("../../../etc/passwd")),
            PathBuf::from("/tmp/test/default.pid")
        );
    }

    #[test]
    fn test_sanitize_valid_names() {
        // Simple alphanumeric.
        assert_eq!(sanitize_session_name("default"), "default");
        assert_eq!(sanitize_session_name("session1"), "session1");
        assert_eq!(sanitize_session_name("MySession"), "MySession");

        // With hyphens and underscores.
        assert_eq!(sanitize_session_name("my-session"), "my-session");
        assert_eq!(sanitize_session_name("my_session"), "my_session");
        assert_eq!(sanitize_session_name("my-session_123"), "my-session_123");

        // Underscores at start are fine.
        assert_eq!(sanitize_session_name("_private"), "_private");
    }

    #[test]
    fn test_sanitize_path_traversal_attacks() {
        // Classic path traversal.
        assert_eq!(sanitize_session_name("../../../etc/passwd"), "default");
        assert_eq!(sanitize_session_name(".."), "default");
        assert_eq!(sanitize_session_name("foo/../bar"), "default");

        // Sneaky variants.
        assert_eq!(sanitize_session_name("foo/bar"), "default");
        assert_eq!(sanitize_session_name("/etc/passwd"), "default");
        assert_eq!(sanitize_session_name("..\\..\\windows"), "default");
    }

    #[test]
    fn test_sanitize_empty_and_whitespace() {
        assert_eq!(sanitize_session_name(""), "default");
        assert_eq!(sanitize_session_name(" "), "default");
        assert_eq!(sanitize_session_name("  "), "default");
        assert_eq!(sanitize_session_name("\t"), "default");
        assert_eq!(sanitize_session_name("\n"), "default");
    }

    #[test]
    fn test_sanitize_hyphen_at_start() {
        // Hyphens at start could be interpreted as CLI options.
        assert_eq!(sanitize_session_name("-session"), "default");
        assert_eq!(sanitize_session_name("--session"), "default");
        assert_eq!(sanitize_session_name("-"), "default");
    }

    #[test]
    fn test_sanitize_special_characters() {
        assert_eq!(sanitize_session_name("session!"), "default");
        assert_eq!(sanitize_session_name("session@home"), "default");
        assert_eq!(sanitize_session_name("session#1"), "default");
        assert_eq!(sanitize_session_name("session$var"), "default");
        assert_eq!(sanitize_session_name("session%20"), "default");
        assert_eq!(sanitize_session_name("session&more"), "default");
        assert_eq!(sanitize_session_name("session;rm -rf"), "default");
        assert_eq!(sanitize_session_name("session|cat"), "default");
        assert_eq!(sanitize_session_name("session`id`"), "default");
        assert_eq!(sanitize_session_name("$(whoami)"), "default");
    }

    #[test]
    fn test_sanitize_null_bytes() {
        assert_eq!(sanitize_session_name("session\0evil"), "default");
        assert_eq!(sanitize_session_name("\0"), "default");
    }
}
