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

require_regex_match() {
  local haystack="$1"
  local pattern="$2"
  local label="$3"

  if ! printf '%s\n' "$haystack" | grep -Eq "$pattern"; then
    printf '%s\n' "$haystack" >&2
    die "missing expected ${label} matching regex: ${pattern}"
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

require_changed_hash() {
  local before_hash="$1"
  local after_hash="$2"
  local label="$3"

  if [ -z "$before_hash" ] || [ -z "$after_hash" ]; then
    die "missing snapshot hash while checking ${label}"
  fi

  if [ "$before_hash" = "$after_hash" ]; then
    die "expected ${label} to change content_hash, but both were ${before_hash}"
  fi
}

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/agent-terminal-preferred-wait.XXXXXX")"
HOME_DIR="$TMP_ROOT/home"
RUNTIME_DIR="$HOME_DIR/.agent-terminal"
SOCKET_PATH="$RUNTIME_DIR/default.sock"
PID_PATH="$RUNTIME_DIR/default.pid"
SESSION_NAME="preferred-wait-$$"
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
    echo "Preserving failed preferred-wait temp home: $TMP_ROOT" >&2
  fi

  exit "$exit_code"
}
trap cleanup EXIT

phase "docs/help contract"
if [ ! -x "$BIN" ]; then
  (cd "$PROJECT_DIR" && cargo build --release)
fi

require_file_contains "$README" 'agent-terminal wait "Ready"' 'README preferred wait literal example'
require_file_contains "$README" 'agent-terminal wait "Error" --regex' 'README preferred wait regex example'
require_file_contains "$README" '`agent-terminal wait-for ...` remains available as a compatibility alias for existing scripts.' 'README wait-for compatibility note'
require_file_contains "$README" 'Use `agent-terminal snapshot --await-change <content_hash> --settle <ms>` when you need to wait for the screen to both change and stabilize.' 'README advanced snapshot guidance'

wait_help="$(capture_agent 'render wait help' wait --help)"
for needle in \
  'Usage: agent-terminal wait [OPTIONS] <PATTERN>' \
  'Simple sync:' \
  'Use agent-terminal wait for literal text or regex polling' \
  'Advanced terminal-state sync:' \
  'snapshot --await-change <content_hash> --settle <ms>' \
  'agent-terminal wait -r '\''error|warning'\''' \
  'agent-terminal wait-for ... remains supported as a compatibility alias'; do
  require_contains "$wait_help" "$needle" 'wait help'
done

wait_for_help="$(capture_agent 'render wait-for alias help' wait-for --help)"
require_contains "$wait_for_help" 'Usage: agent-terminal wait [OPTIONS] <PATTERN>' 'wait-for alias canonical usage'
require_contains "$wait_for_help" 'agent-terminal wait '\''Ready'\''' 'wait-for alias preferred example'
require_contains "$wait_for_help" 'snapshot --await-change <content_hash> --settle <ms>' 'wait-for alias advanced snapshot guidance'
if printf '%s' "$wait_for_help" | grep -Fq 'Usage: agent-terminal wait-for [OPTIONS] <PATTERN>'; then
  printf '%s\n' "$wait_for_help" >&2
  die 'wait-for alias help rendered a separate usage contract'
fi

phase "simple wait"
spawn_output="$(capture_agent 'spawn deterministic shell session' spawn --name "$SESSION_NAME" -- bash --noprofile --rcfile "$BASHRC" -i)"
require_contains "$spawn_output" '"type": "session_created"' 'session_created response'

prompt_ready="$(capture_agent 'wait for initial prompt with preferred wait' wait -s "$SESSION_NAME" -t 10000 'PROMPT> ')"
require_contains "$prompt_ready" '"found": true' 'preferred wait prompt result'

literal_token="literal-${UNIQUE_SUFFIX}"
regex_token="regex-${UNIQUE_SUFFIX}-42"
invoke_agent 'type literal/regex probe command' type -s "$SESSION_NAME" "printf '$literal_token\\n$regex_token\\n'"
invoke_agent 'submit literal/regex probe command' press -s "$SESSION_NAME" Enter

literal_output="$(capture_agent 'wait for literal token with preferred wait' wait -s "$SESSION_NAME" -t 10000 "$literal_token")"
require_contains "$literal_output" '"found": true' 'preferred wait literal result'
require_contains "$literal_output" "$literal_token" 'preferred wait literal matched text'

regex_output="$(capture_agent 'wait for regex token with preferred wait' wait -s "$SESSION_NAME" -r -t 10000 "regex-${UNIQUE_SUFFIX}-[0-9]+")"
require_contains "$regex_output" '"found": true' 'preferred wait regex result'
require_regex_match "$regex_output" "regex-${UNIQUE_SUFFIX}-[0-9]+" 'preferred wait regex matched text'

phase "snapshot settle"
pre_snapshot="$(capture_agent 'capture pre-change snapshot text' snapshot -s "$SESSION_NAME" --format text)"
require_contains "$pre_snapshot" 'PROMPT> ' 'pre-change prompt'
baseline_hash="$(snapshot_hash "$SESSION_NAME")"

snapshot_start="snapshot-start-${UNIQUE_SUFFIX}"
snapshot_end="snapshot-end-${UNIQUE_SUFFIX}"
invoke_agent 'type staged snapshot command' type -s "$SESSION_NAME" "printf '$snapshot_start\\n'; sleep 0.3; printf '$snapshot_end\\n'"
invoke_agent 'submit staged snapshot command' press -s "$SESSION_NAME" Enter
settled_screen="$(capture_agent 'await snapshot change and settle' snapshot -s "$SESSION_NAME" --await-change "$baseline_hash" --settle 400 -t 10000 --format text)"
require_contains "$settled_screen" "$snapshot_start" 'snapshot change output start token'
require_contains "$settled_screen" "$snapshot_end" 'snapshot settle output end token'
require_contains "$settled_screen" 'PROMPT> ' 'snapshot settle prompt restoration'
final_hash="$(snapshot_hash "$SESSION_NAME")"
require_changed_hash "$baseline_hash" "$final_hash" 'snapshot settle proof'

phase "compatibility alias"
compat_token="compat-${UNIQUE_SUFFIX}"
invoke_agent 'type compatibility alias probe command' type -s "$SESSION_NAME" "echo $compat_token"
invoke_agent 'submit compatibility alias probe command' press -s "$SESSION_NAME" Enter
compat_output="$(capture_agent 'wait with compatibility alias' wait-for -s "$SESSION_NAME" -t 10000 "$compat_token")"
require_contains "$compat_output" '"found": true' 'wait-for compatibility result'
require_contains "$compat_output" "$compat_token" 'wait-for compatibility matched text'

phase "daemon stop"
[ -d "$RUNTIME_DIR" ] || die "missing runtime directory before stop: $RUNTIME_DIR"
[ -S "$SOCKET_PATH" ] || die "missing runtime socket before stop: $SOCKET_PATH"
[ -f "$PID_PATH" ] || die "missing runtime pid file before stop: $PID_PATH"
kill_output="$(capture_agent 'kill interactive session before daemon stop' kill -s "$SESSION_NAME")"
require_contains "$kill_output" '"type": "ok"' 'kill response'

stop_output="$(capture_agent 'stop daemon after wait/snapshot proof' stop)"
require_contains "$stop_output" 'Daemon shutting down' 'stop confirmation'

phase "cleanup"
wait_for_absent "$SOCKET_PATH" 'daemon socket'
wait_for_absent "$PID_PATH" 'daemon pid file'
if find "$HOME_DIR" -maxdepth 3 -name '*pilotty*' | grep -q .; then
  find "$HOME_DIR" -maxdepth 3 -print | sort >&2 || true
  die 'found stale pilotty-named artifacts under isolated HOME'
fi

echo 'PASS: preferred wait contract verified.'
