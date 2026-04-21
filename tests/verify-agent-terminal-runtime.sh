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

require_not_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"

  if printf '%s' "$haystack" | grep -Fq "$needle"; then
    printf '%s\n' "$haystack" >&2
    die "found unexpected ${label}: ${needle}"
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

assert_no_generic_wait_for() {
  local path="$1"
  local line=""
  local line_number=0
  local found=0

  while IFS= read -r line; do
    line_number=$((line_number + 1))
    [[ "$line" == '#'* ]] && continue

    if [[ "$line" == *wait-for* ]]; then
      if [[ "$line" =~ (^|[^[:alnum:]_])(run_agent|capture_agent|invoke_agent)([^[:alnum:]_]|$) ]]; then
        printf '%s:%s:%s\n' "$path" "$line_number" "$line" >&2
        found=1
      fi
    fi
  done < "$path"

  if [ "$found" -ne 0 ]; then
    die "generic smoke scripts still invoke wait-for; keep compatibility proof isolated to dedicated verifiers"
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
  local hash

  json="$(capture_agent "capture snapshot hash" snapshot -s "$session")"
  hash="$(printf '%s' "$json" \
    | grep -oE '"content_hash"[[:space:]]*:[[:space:]]*[0-9]+' \
    | head -n1 \
    | grep -oE '[0-9]+' || true)"

  if [ -z "$hash" ]; then
    printf '%s\n' "$json" >&2
    die 'snapshot response missing content_hash'
  fi

  printf '%s' "$hash"
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

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/agent-terminal-runtime.XXXXXX")"
HOME_DIR="$TMP_ROOT/home"
RUNTIME_DIR="$HOME_DIR/.agent-terminal"
SOCKET_PATH="$RUNTIME_DIR/default.sock"
PID_PATH="$RUNTIME_DIR/default.pid"
SESSION_NAME="runtime-smoke-$$"
UNIQUE_SUFFIX="$$-$(date +%s)"
LIFECYCLE_TOKEN="runtime-smoke-${UNIQUE_SUFFIX}"
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
    echo "Preserving failed smoke temp home: $TMP_ROOT" >&2
  fi

  exit "$exit_code"
}
trap cleanup EXIT

phase "docs/help contract"
if [ ! -x "$BIN" ]; then
  (cd "$PROJECT_DIR" && cargo build --release)
fi

require_file_contains "$README" 'agent-terminal wait -s shell "agent-terminal> "' 'README preferred wait example'
require_file_contains "$README" 'agent-terminal press -s shell Enter' 'README preferred press example'
require_file_contains "$README" 'agent-terminal snapshot -s shell --await-change "$HASH" --settle 100' 'README settled snapshot example'
require_file_contains "$README" 'Compatibility spellings remain available for existing scripts:' 'README compatibility note'

assert_no_generic_wait_for "$PROJECT_DIR/tests/verify-agent-terminal-runtime.sh"
assert_no_generic_wait_for "$PROJECT_DIR/tests/roundtrip.sh"

top_help="$(capture_agent 'render top-level help' --help)"
require_contains "$top_help" '  press' 'top-level press command'
require_contains "$top_help" '  wait' 'top-level wait command'
require_not_contains "$top_help" '  wait-for' 'top-level wait-for command entry'

examples_output="$(capture_agent 'render examples output' examples)"
require_contains "$examples_output" 'agent-terminal wait -s shell "agent-terminal> "' 'examples preferred wait example'
require_contains "$examples_output" 'agent-terminal press -s shell Enter' 'examples preferred press example'
require_contains "$examples_output" 'agent-terminal snapshot -s shell --await-change "$HASH" --settle 100' 'examples settled snapshot example'
require_contains "$examples_output" 'agent-terminal wait-for ... still work.' 'examples compatibility note'
require_not_contains "$examples_output" 'agent-terminal wait-for -s shell' 'examples legacy wait smoke'

phase "prompt readiness"
spawn_output="$(capture_agent 'spawn interactive bash session' spawn --name "$SESSION_NAME" -- bash --noprofile --rcfile "$BASHRC" -i)"
require_contains "$spawn_output" '"type": "session_created"' 'session_created response'

prompt_ready="$(capture_agent 'wait for deterministic shell prompt' wait -s "$SESSION_NAME" -t 10000 'PROMPT> ')"
require_contains "$prompt_ready" '"found": true' 'preferred wait prompt result'

phase "runtime placement"
[ -d "$RUNTIME_DIR" ] || die "missing runtime directory: $RUNTIME_DIR"
[ -S "$SOCKET_PATH" ] || die "missing runtime socket: $SOCKET_PATH"
[ -f "$PID_PATH" ] || die "missing runtime pid file: $PID_PATH"
[ ! -e "$HOME_DIR/.pilotty" ] || die "stale .pilotty runtime directory created"
if find "$HOME_DIR" -maxdepth 3 -name '*pilotty*' | grep -q .; then
  find "$HOME_DIR" -maxdepth 3 -print | sort >&2 || true
  die "found stale pilotty-named runtime artifacts under isolated HOME"
fi

baseline_screen="$(capture_agent 'capture baseline shell snapshot' snapshot -s "$SESSION_NAME" --format text)"
require_contains "$baseline_screen" 'PROMPT> ' 'baseline prompt snapshot'
baseline_hash="$(snapshot_hash "$SESSION_NAME")"

phase "lifecycle action"
invoke_agent 'type preferred lifecycle command' type -s "$SESSION_NAME" "printf '$LIFECYCLE_TOKEN\\n'"
invoke_agent 'submit preferred lifecycle command' press -s "$SESSION_NAME" Enter

settled_screen="$(capture_agent 'await lifecycle output and settled prompt' snapshot -s "$SESSION_NAME" --await-change "$baseline_hash" --settle 200 -t 10000 --format text)"
require_contains "$settled_screen" "$LIFECYCLE_TOKEN" 'lifecycle output token'
require_contains "$settled_screen" 'PROMPT> ' 'restored prompt after lifecycle action'
final_hash="$(snapshot_hash "$SESSION_NAME")"
require_changed_hash "$baseline_hash" "$final_hash" 'preferred lifecycle action'

phase "daemon stop"
kill_output="$(capture_agent 'kill interactive session before daemon stop' kill -s "$SESSION_NAME")"
require_contains "$kill_output" '"type": "ok"' 'kill response'

stop_output="$(capture_agent 'stop daemon after runtime proof' stop)"
require_contains "$stop_output" 'Daemon shutting down' 'stop confirmation'

phase "cleanup"
wait_for_absent "$SOCKET_PATH" 'daemon socket'
wait_for_absent "$PID_PATH" 'daemon pid file'
find "$HOME_DIR" -maxdepth 3 -print | sort

echo 'PASS: agent-terminal runtime smoke verified.'
