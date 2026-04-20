use agent_terminal_core::error::{ApiError, ErrorCode};

fn suggestion(err: &ApiError) -> &str {
    err.suggestion
        .as_deref()
        .unwrap_or_else(|| panic!("expected suggestion for {:?}", err.code))
}

fn assert_agent_terminal_hint(label: &str, err: &ApiError, expected_code: ErrorCode, hint: &str) {
    assert_eq!(err.code, expected_code, "{label} changed error code");

    let suggestion = suggestion(err);
    assert!(
        suggestion.contains("agent-terminal"),
        "{label} should mention agent-terminal, got: {suggestion}"
    );
    assert!(
        suggestion.contains(hint),
        "{label} should mention {hint}, got: {suggestion}"
    );
    assert!(
        !suggestion.contains("pilotty"),
        "{label} should not leak pilotty, got: {suggestion}"
    );
}

#[test]
fn session_and_write_error_hints_use_agent_terminal_identity() {
    let cases = [
        (
            "session_not_found",
            ApiError::session_not_found("missing-session"),
            ErrorCode::SessionNotFound,
            "list-sessions",
        ),
        (
            "no_sessions",
            ApiError::no_sessions(),
            ErrorCode::SessionNotFound,
            "spawn <command>",
        ),
        (
            "session_limit_reached",
            ApiError::session_limit_reached(4),
            ErrorCode::CommandFailed,
            "kill",
        ),
        (
            "write_failed",
            ApiError::write_failed("broken pipe"),
            ErrorCode::CommandFailed,
            "list-sessions",
        ),
    ];

    for (label, err, expected_code, hint) in cases {
        assert_agent_terminal_hint(label, &err, expected_code, hint);
    }
}
