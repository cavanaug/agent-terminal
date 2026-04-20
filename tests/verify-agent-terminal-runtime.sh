#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN="$PROJECT_DIR/target/release/agent-terminal"

phase() {
  printf '\n==> %s\n' "$1"
}

die() {
  echo "FAIL: $1" >&2
  exit 1
}

wait_for_absent() {
  local path="$1"
  local label="$2"
  local attempts=0
  local max_attempts=50

  while [ -e "$path" ]; do
    if [ "$attempts" -ge "$max_attempts" ]; then
      echo "Timed out waiting for $label to disappear: $path" >&2
      find "$HOME_DIR" -maxdepth 3 -print 2>/dev/null | sort >&2 || true
      return 1
    fi
    sleep 0.1
    attempts=$((attempts + 1))
  done
}

run_agent() {
  env -u AGENT_TERMINAL_SESSION \
      -u AGENT_TERMINAL_SOCKET_DIR \
      HOME="$HOME_DIR" \
      XDG_RUNTIME_DIR= \
      "$BIN" "$@"
}

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/agent-terminal-runtime.XXXXXX")"
HOME_DIR="$TMP_ROOT/home"
RUNTIME_DIR="$HOME_DIR/.agent-terminal"
SOCKET_PATH="$RUNTIME_DIR/default.sock"
PID_PATH="$RUNTIME_DIR/default.pid"
SESSION_NAME="runtime-smoke-$$"
mkdir -p "$HOME_DIR"

cleanup() {
  local exit_code=$?
  trap - EXIT

  run_agent kill -s "$SESSION_NAME" >/dev/null 2>&1 || true
  run_agent stop >/dev/null 2>&1 || true

  if [ "$exit_code" -eq 0 ]; then
    rm -rf "$TMP_ROOT"
  else
    echo "Preserving failed smoke temp home: $TMP_ROOT" >&2
  fi

  exit "$exit_code"
}
trap cleanup EXIT

phase "Ensure release binary exists"
if [ ! -x "$BIN" ]; then
  (cd "$PROJECT_DIR" && cargo build --release)
fi

phase "Spawn named session under default daemon runtime"
spawn_output="$(run_agent spawn --name "$SESSION_NAME" -- bash -lc 'echo READY; sleep 60')"
printf '%s\n' "$spawn_output"
printf '%s' "$spawn_output" | grep -q '"type": "session_created"' \
  || die "spawn did not return session_created"

phase "Wait for named session output"
wait_output="$(run_agent wait-for -s "$SESSION_NAME" -t 10000 READY)"
printf '%s\n' "$wait_output"
printf '%s' "$wait_output" | grep -q '"found": true' \
  || die "wait-for did not confirm READY"

phase "Assert ~/.agent-terminal runtime placement"
[ -d "$RUNTIME_DIR" ] || die "missing runtime directory: $RUNTIME_DIR"
[ -S "$SOCKET_PATH" ] || die "missing runtime socket: $SOCKET_PATH"
[ -f "$PID_PATH" ] || die "missing runtime pid file: $PID_PATH"
[ ! -e "$HOME_DIR/.pilotty" ] || die "stale .pilotty runtime directory created"
if find "$HOME_DIR" -maxdepth 3 -name '*pilotty*' | grep -q .; then
  die "found stale pilotty-named runtime artifacts under isolated HOME"
fi
find "$HOME_DIR" -maxdepth 3 -print | sort

phase "Snapshot named session"
snapshot_output="$(run_agent snapshot -s "$SESSION_NAME" --format text)"
printf '%s\n' "$snapshot_output"
printf '%s' "$snapshot_output" | grep -q 'READY' \
  || die "snapshot did not contain READY"

phase "Kill named session"
kill_output="$(run_agent kill -s "$SESSION_NAME")"
printf '%s\n' "$kill_output"
printf '%s' "$kill_output" | grep -q '"type": "ok"' \
  || die "kill did not return ok"

phase "Stop daemon and wait for runtime cleanup"
stop_output="$(run_agent stop)"
printf '%s\n' "$stop_output"
printf '%s' "$stop_output" | grep -q 'Daemon shutting down' \
  || die "stop did not confirm daemon shutdown"
wait_for_absent "$SOCKET_PATH" "socket file" \
  || die "socket file was still present after daemon stop"
wait_for_absent "$PID_PATH" "pid file" \
  || die "pid file was still present after daemon stop"
find "$HOME_DIR" -maxdepth 3 -print | sort

phase "Smoke verification complete"
echo "PASS: agent-terminal runtime smoke verified."
