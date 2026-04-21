#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN="$PROJECT_DIR/target/release/agent-terminal"
README="$PROJECT_DIR/README.md"
CURRENT_PHASE=""

phase() {
  CURRENT_PHASE="$1"
  printf '\n==> %s\n' "$CURRENT_PHASE"
}

die() {
  echo "FAIL${CURRENT_PHASE:+ [$CURRENT_PHASE]}: $1" >&2
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

require_file_contains() {
  local path="$1"
  local needle="$2"
  local label="$3"

  if ! rg -n --fixed-strings "$needle" "$path" >/dev/null; then
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

capture_agent() {
  local label="$1"
  shift
  local output

  if ! output="$(run_agent "$@" 2>&1)"; then
    printf '%s\n' "$output" >&2
    die "$label"
  fi

  printf '%s' "$output"
}

invoke_agent() {
  local label="$1"
  shift
  local output

  if ! output="$(run_agent "$@" 2>&1)"; then
    printf '%s\n' "$output" >&2
    die "$label"
  fi
}

snapshot_hash() {
  local session="$1"
  local json
  json="$(capture_agent "capture snapshot hash" snapshot -s "$session")"
  printf '%s' "$json" \
    | grep -oE '"content_hash"[[:space:]]*:[[:space:]]*[0-9]+' \
    | head -n1 \
    | grep -oE '[0-9]+'
}

verify_interrupt_flow() {
  local phase_label="$1"
  local command_name="$2"
  local key_value="$3"
  local after_token="$4"
  local sleep_hash
  local interrupted_screen
  local after_output

  phase "$phase_label"
  invoke_agent "type long-running command before ${command_name} ${key_value}" type -s "$SESSION_NAME" 'sleep 60'
  invoke_agent "submit long-running command before ${command_name} ${key_value}" press -s "$SESSION_NAME" Enter
  sleep 0.2
  sleep_hash="$(snapshot_hash "$SESSION_NAME")"
  invoke_agent "send ${command_name} ${key_value}" "$command_name" -s "$SESSION_NAME" "$key_value"
  interrupted_screen="$(capture_agent "await prompt after ${command_name} ${key_value}" snapshot -s "$SESSION_NAME" --await-change "$sleep_hash" --settle 100 -t 10000 --format text)"
  require_contains "$interrupted_screen" 'PROMPT> ' "prompt after ${command_name} ${key_value}"

  invoke_agent "type follow-up command after ${command_name} ${key_value}" type -s "$SESSION_NAME" "echo $after_token"
  invoke_agent "submit follow-up command after ${command_name} ${key_value}" press -s "$SESSION_NAME" Enter
  after_output="$(capture_agent "wait for follow-up output after ${command_name} ${key_value}" wait-for -s "$SESSION_NAME" -t 10000 "$after_token")"
  require_contains "$after_output" '"found": true' "follow-up output after ${command_name} ${key_value}"
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

phase "docs and help checks"
if [ ! -x "$BIN" ]; then
  (cd "$PROJECT_DIR" && cargo build --release)
fi

require_file_contains "$README" 'agent-terminal press Control+C' 'README preferred interrupt example'
require_file_contains "$README" 'agent-terminal press ArrowUp' 'README preferred arrow example'
require_file_contains "$README" 'Compatibility spellings: `key`, `Ctrl+...`, `Alt+...`, and short arrows like `Up` still work.' 'README compatibility note'

top_help="$(capture_agent 'render top-level help' --help)"
require_contains "$top_help" '  press' 'top-level press command'
if printf '%s' "$top_help" | grep -Fq '  key            Send a key, key combination, or key sequence'; then
  printf '%s\n' "$top_help" >&2
  die 'top-level help still lists key as the canonical command'
fi

press_help="$(capture_agent 'render press help' press --help)"
for needle in \
  'Usage: agent-terminal press [OPTIONS] <KEY>' \
  'Control+<key>' \
  'Meta+<key>' \
  'Option+<key>' \
  'ArrowUp' \
  'agent-terminal press Control+C' \
  'agent-terminal press ArrowUp'; do
  require_contains "$press_help" "$needle" 'press help'
done

key_help="$(capture_agent 'render key compatibility help' key --help)"
require_contains "$key_help" 'Usage: agent-terminal press [OPTIONS] <KEY>' 'key alias canonical usage'
require_contains "$key_help" 'agent-terminal press Control+C' 'key alias preferred example'

phase "shell-history recall"
spawn_output="$(capture_agent 'spawn interactive bash session' spawn --name "$SESSION_NAME" -- bash --noprofile --rcfile "$BASHRC" -i)"
require_contains "$spawn_output" '"type": "session_created"' 'session_created response'

prompt_ready="$(capture_agent 'wait for initial shell prompt' wait-for -s "$SESSION_NAME" -t 10000 'PROMPT> ')"
require_contains "$prompt_ready" '"found": true' 'initial prompt wait'

recall_token="recall-${UNIQUE_SUFFIX}"
invoke_agent 'type recall seed command' type -s "$SESSION_NAME" "echo $recall_token"
invoke_agent 'submit recall seed command' press -s "$SESSION_NAME" Enter
recall_output="$(capture_agent 'wait for recall seed output' wait-for -s "$SESSION_NAME" -t 10000 "$recall_token")"
require_contains "$recall_output" '"found": true' 'recall command output'

post_command_hash="$(snapshot_hash "$SESSION_NAME")"
invoke_agent 'send ArrowUp for history recall' press -s "$SESSION_NAME" ArrowUp
recalled_screen="$(capture_agent 'await history recall snapshot change' snapshot -s "$SESSION_NAME" --await-change "$post_command_hash" --settle 100 -t 10000 --format text)"
require_contains "$recalled_screen" "PROMPT> echo $recall_token" 'recalled history line'

recalled_hash="$(snapshot_hash "$SESSION_NAME")"
invoke_agent 'replay recalled command' press -s "$SESSION_NAME" Enter
replayed_screen="$(capture_agent 'await replayed command output' snapshot -s "$SESSION_NAME" --await-change "$recalled_hash" --settle 100 -t 10000 --format text)"
require_contains "$replayed_screen" "$recall_token" 'replayed history output'

verify_interrupt_flow 'preferred interrupt delivery' 'press' 'Control+C' "after-press-${UNIQUE_SUFFIX}"
verify_interrupt_flow 'compatibility interrupt delivery' 'key' 'Ctrl+C' "after-key-${UNIQUE_SUFFIX}"

phase "daemon stop"
kill_output="$(capture_agent 'kill interactive session before daemon stop' kill -s "$SESSION_NAME")"
require_contains "$kill_output" '"type": "ok"' 'kill response'

stop_output="$(capture_agent 'stop daemon after runtime proof' stop)"
require_contains "$stop_output" 'Daemon shutting down' 'stop confirmation'

phase "runtime cleanup"
wait_for_absent "$SOCKET_PATH" 'daemon socket'
wait_for_absent "$PID_PATH" 'daemon pid file'
if find "$HOME_DIR" -maxdepth 3 -name '*pilotty*' | grep -q .; then
  find "$HOME_DIR" -maxdepth 3 -print | sort >&2 || true
  die 'found stale pilotty-named artifacts under isolated HOME'
fi

echo 'PASS: preferred key contract verified.'
