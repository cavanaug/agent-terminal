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

require_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"

  if ! printf '%s' "$haystack" | grep -Fq "$needle"; then
    printf '%s\n' "$haystack" >&2
    die "missing expected ${label}: ${needle}"
  fi
}

wait_for_absent() {
  local path="$1"
  local label="$2"
  local attempts=0
  local max_attempts=50

  while [ -e "$path" ]; do
    if [ "$attempts" -ge "$max_attempts" ]; then
      find "$HOME_DIR" -maxdepth 3 -print 2>/dev/null | sort >&2 || true
      die "timed out waiting for ${label} to disappear: ${path}"
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
      RUST_LOG=error \
      "$BIN" "$@"
}

snapshot_hash() {
  local session="$1"
  local json
  json="$(run_agent snapshot -s "$session")"
  printf '%s' "$json" \
    | grep -oE '"content_hash"[[:space:]]*:[[:space:]]*[0-9]+' \
    | head -n1 \
    | grep -oE '[0-9]+'
}

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/agent-terminal-preferred-key.XXXXXX")"
HOME_DIR="$TMP_ROOT/home"
RUNTIME_DIR="$HOME_DIR/.agent-terminal"
SOCKET_PATH="$RUNTIME_DIR/default.sock"
PID_PATH="$RUNTIME_DIR/default.pid"
SESSION_NAME="preferred-key-$$"
UNIQUE_SUFFIX="$$-$(date +%s)"
BASHRC="$TMP_ROOT/bashrc"
mkdir -p "$HOME_DIR"

cat > "$BASHRC" <<'EOF'
export PS1='PROMPT> '
export PROMPT_COMMAND=
export HISTFILE="$HOME/.bash_history"
set -o history
bind 'set editing-mode emacs'
EOF

cleanup() {
  local exit_code=$?
  trap - EXIT

  run_agent kill -s "$SESSION_NAME" >/dev/null 2>&1 || true
  run_agent stop >/dev/null 2>&1 || true

  if [ "$exit_code" -eq 0 ]; then
    rm -rf "$TMP_ROOT"
  else
    echo "Preserving failed preferred-key temp home: $TMP_ROOT" >&2
  fi

  exit "$exit_code"
}
trap cleanup EXIT

phase "help checks"
if [ ! -x "$BIN" ]; then
  (cd "$PROJECT_DIR" && cargo build --release)
fi

top_help="$(run_agent --help)"
require_contains "$top_help" "  press" "top-level press command"
if printf '%s' "$top_help" | grep -Fq "  key            Send a key, key combination, or key sequence"; then
  printf '%s\n' "$top_help" >&2
  die "top-level help still lists key as the canonical command"
fi

press_help="$(run_agent press --help)"
for needle in \
  "Usage: agent-terminal press [OPTIONS] <KEY>" \
  "Control+<key>" \
  "Meta+<key>" \
  "Option+<key>" \
  "ArrowUp" \
  "agent-terminal press Control+C" \
  "agent-terminal press Meta+F" \
  "agent-terminal press Option+F" \
  "agent-terminal press ArrowUp"; do
  require_contains "$press_help" "$needle" "press help"
done

key_help="$(run_agent key --help)"
require_contains "$key_help" "Usage: agent-terminal press [OPTIONS] <KEY>" "key alias canonical usage"
require_contains "$key_help" "agent-terminal press Control+C" "key alias preferred example"

phase "shell-history recall"
spawn_output="$(run_agent spawn --name "$SESSION_NAME" -- bash --noprofile --rcfile "$BASHRC" -i)"
require_contains "$spawn_output" '"type": "session_created"' "session_created response"

prompt_ready="$(run_agent wait-for -s "$SESSION_NAME" -t 10000 'PROMPT> ')"
require_contains "$prompt_ready" '"found": true' "initial prompt wait"

recall_token="recall-${UNIQUE_SUFFIX}"
run_agent type -s "$SESSION_NAME" "echo $recall_token" >/dev/null
run_agent press -s "$SESSION_NAME" Enter >/dev/null
recall_output="$(run_agent wait-for -s "$SESSION_NAME" -t 10000 "$recall_token")"
require_contains "$recall_output" '"found": true' "recall command output"

post_command_hash="$(snapshot_hash "$SESSION_NAME")"
run_agent press -s "$SESSION_NAME" ArrowUp >/dev/null
recalled_screen="$(run_agent snapshot -s "$SESSION_NAME" --await-change "$post_command_hash" --settle 100 --format text)"
require_contains "$recalled_screen" "PROMPT> echo $recall_token" "recalled history line"

recalled_hash="$(snapshot_hash "$SESSION_NAME")"
run_agent press -s "$SESSION_NAME" Enter >/dev/null
replayed_screen="$(run_agent snapshot -s "$SESSION_NAME" --await-change "$recalled_hash" --settle 100 --format text)"
require_contains "$replayed_screen" "$recall_token" "replayed history output"

phase "interrupt delivery"
run_agent type -s "$SESSION_NAME" 'sleep 60' >/dev/null
run_agent press -s "$SESSION_NAME" Enter >/dev/null
sleep 0.2
sleep_hash="$(snapshot_hash "$SESSION_NAME")"
run_agent press -s "$SESSION_NAME" Control+C >/dev/null
interrupted_screen="$(run_agent snapshot -s "$SESSION_NAME" --await-change "$sleep_hash" --settle 100 --format text)"
require_contains "$interrupted_screen" 'PROMPT> ' "prompt after interrupt"

after_interrupt_token="after-interrupt-${UNIQUE_SUFFIX}"
run_agent type -s "$SESSION_NAME" "echo $after_interrupt_token" >/dev/null
run_agent press -s "$SESSION_NAME" Enter >/dev/null
after_interrupt_output="$(run_agent wait-for -s "$SESSION_NAME" -t 10000 "$after_interrupt_token")"
require_contains "$after_interrupt_output" '"found": true' "post-interrupt echo"

phase "cleanup"
kill_output="$(run_agent kill -s "$SESSION_NAME")"
require_contains "$kill_output" '"type": "ok"' "kill response"

stop_output="$(run_agent stop)"
require_contains "$stop_output" 'Daemon shutting down' "stop confirmation"
wait_for_absent "$SOCKET_PATH" "daemon socket"
wait_for_absent "$PID_PATH" "daemon pid file"
if find "$HOME_DIR" -maxdepth 3 -name '*pilotty*' | grep -q .; then
  find "$HOME_DIR" -maxdepth 3 -print | sort >&2 || true
  die "found stale pilotty-named artifacts under isolated HOME"
fi

echo 'PASS: preferred key contract verified.'
