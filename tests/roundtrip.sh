#!/usr/bin/env bash
# Round-trip ANSI fidelity test.
#
# Proves that agent-terminal's ANSI text output is faithful: a terminal emulator
# (vt100) can parse it back into equivalent screen state.
#
# Pipeline:
#   1. Spawn session A displaying a styled ANSI fixture
#   2. Capture Full JSON snapshot (style_map + color_map) and Text snapshot (ANSI output)
#   3. Strip header line from text output
#   4. Spawn session B displaying the stripped ANSI text
#   5. Capture Full JSON snapshot of session B
#   6. Compare text, style_map, and color_map between A and B
#
# Requirements: bash, jq, cargo
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FIXTURE="$SCRIPT_DIR/fixtures/roundtrip.ansi"
BIN="$PROJECT_DIR/target/debug/agent-terminal"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/agent-terminal-roundtrip.XXXXXX")"
WORK_DIR="$TMP_ROOT/work"
HOME_DIR="$TMP_ROOT/home"
SRC_SESSION="roundtrip-src-$$"
DST_SESSION="roundtrip-dst-$$"
mkdir -p "$WORK_DIR" "$HOME_DIR"

phase() {
    printf '\n==> %s\n' "$1"
}

die() {
    echo "FAIL: $1" >&2
    exit 1
}

run_agent() {
    env -u AGENT_TERMINAL_SESSION \
        -u AGENT_TERMINAL_SOCKET_DIR \
        HOME="$HOME_DIR" \
        XDG_RUNTIME_DIR= \
        "$BIN" "$@"
}

validate_screen_state() {
    local path="$1"
    local label="$2"

    jq -e '
        .type == "screen_state"
        and (.text | type == "array")
        and (.style_map | type == "array")
        and (.color_map | type == "array")
    ' "$path" > /dev/null || die "$label is not a valid screen_state snapshot"
}

cleanup() {
    local exit_code=$?
    trap - EXIT

    run_agent kill -s "$SRC_SESSION" >/dev/null 2>&1 || true
    run_agent kill -s "$DST_SESSION" >/dev/null 2>&1 || true
    run_agent stop >/dev/null 2>&1 || true

    if [ "$exit_code" -eq 0 ]; then
        rm -rf "$TMP_ROOT"
    else
        echo "Preserving failed round-trip temp dir: $TMP_ROOT" >&2
    fi

    exit "$exit_code"
}
trap cleanup EXIT

phase "Build debug binary"
(cd "$PROJECT_DIR" && cargo build) || die "cargo build failed"

phase "Assert renamed debug binary is the proof surface"
[ -x "$BIN" ] || die "Missing debug binary: $BIN"

phase "Check tracked ANSI fixture"
[ -f "$FIXTURE" ] || die "Missing fixture: $FIXTURE"

phase "Spawn source session"
spawn_source_output="$(run_agent spawn --name "$SRC_SESSION" -- bash -lc "cat '$FIXTURE'; sleep 60")"
printf '%s\n' "$spawn_source_output"
printf '%s' "$spawn_source_output" | grep -q '"type": "session_created"' \
    || die "Source spawn did not return session_created"

wait_source_output="$(run_agent wait -s "$SRC_SESSION" -t 5000 "Bold text")"
printf '%s\n' "$wait_source_output"
printf '%s' "$wait_source_output" | grep -q '"found": true' \
    || die "Source wait did not observe rendered fixture text"

phase "Capture source snapshots"
run_agent snapshot -s "$SRC_SESSION" --format full --render color > "$WORK_DIR/full-A.json" \
    || die "Failed to capture full snapshot A"
run_agent snapshot -s "$SRC_SESSION" --format text --render color > "$WORK_DIR/text-A.txt" \
    || die "Failed to capture text snapshot A"

validate_screen_state "$WORK_DIR/full-A.json" "Source snapshot"
[ -s "$WORK_DIR/text-A.txt" ] || die "Source text snapshot was empty"
jq -e '.style_map | length > 0' "$WORK_DIR/full-A.json" > /dev/null \
    || die "Source snapshot has no style_map entries"

phase "Prepare stripped ANSI replay"
sed '1{/^--- /d}' "$WORK_DIR/text-A.txt" \
    | sed 's/\[\(.\)\]/\1/g' \
    | sed 's/[[:space:]]*$//' \
    | sed -e :a -e '/^\n*$/{$d;N;ba}' \
    > "$WORK_DIR/stripped-A.txt"

perl -pi -e 'chomp if eof' "$WORK_DIR/stripped-A.txt"

echo "Stripped text output: $(wc -l < "$WORK_DIR/stripped-A.txt") lines"

phase "Spawn destination session"
spawn_destination_output="$(run_agent spawn --name "$DST_SESSION" -- bash -lc "cat '$WORK_DIR/stripped-A.txt'; sleep 60")"
printf '%s\n' "$spawn_destination_output"
printf '%s' "$spawn_destination_output" | grep -q '"type": "session_created"' \
    || die "Destination spawn did not return session_created"

wait_destination_output="$(run_agent wait -s "$DST_SESSION" -t 5000 "Bold text")"
printf '%s\n' "$wait_destination_output"
printf '%s' "$wait_destination_output" | grep -q '"found": true' \
    || die "Destination wait did not observe replayed text"

phase "Capture destination snapshot"
run_agent snapshot -s "$DST_SESSION" --format full --render color > "$WORK_DIR/full-B.json" \
    || die "Failed to capture full snapshot B"
validate_screen_state "$WORK_DIR/full-B.json" "Destination snapshot"

phase "Compare round-trip state"
jq -S '.style_map // [] | sort_by(.r, .c)' "$WORK_DIR/full-A.json" > "$WORK_DIR/style-A.json"
jq -S '.style_map // [] | sort_by(.r, .c)' "$WORK_DIR/full-B.json" > "$WORK_DIR/style-B.json"

jq -S '.color_map // [] | sort_by(.r, .c)' "$WORK_DIR/full-A.json" > "$WORK_DIR/color-A.json"
jq -S '.color_map // [] | sort_by(.r, .c)' "$WORK_DIR/full-B.json" > "$WORK_DIR/color-B.json"

jq -r '.text // [] | sort_by(.r) | .[].t' "$WORK_DIR/full-A.json" | sed 's/[[:space:]]*$//' > "$WORK_DIR/text-content-A.txt"
jq -r '.text // [] | sort_by(.r) | .[].t' "$WORK_DIR/full-B.json" | sed 's/[[:space:]]*$//' > "$WORK_DIR/text-content-B.txt"

PASS=true

if ! diff -u "$WORK_DIR/text-content-A.txt" "$WORK_DIR/text-content-B.txt" > "$WORK_DIR/text-diff.txt" 2>&1; then
    echo "TEXT MISMATCH:"
    cat "$WORK_DIR/text-diff.txt"
    PASS=false
else
    echo "  Text content: MATCH"
fi

if ! diff -u "$WORK_DIR/style-A.json" "$WORK_DIR/style-B.json" > "$WORK_DIR/style-diff.txt" 2>&1; then
    echo "STYLE MAP MISMATCH:"
    cat "$WORK_DIR/style-diff.txt"
    PASS=false
else
    STYLE_COUNT=$(jq 'length' "$WORK_DIR/style-A.json")
    echo "  Style map: MATCH ($STYLE_COUNT entries)"
fi

if ! diff -u "$WORK_DIR/color-A.json" "$WORK_DIR/color-B.json" > "$WORK_DIR/color-diff.txt" 2>&1; then
    echo "COLOR MAP MISMATCH:"
    cat "$WORK_DIR/color-diff.txt"
    PASS=false
else
    COLOR_COUNT=$(jq 'length' "$WORK_DIR/color-A.json")
    echo "  Color map: MATCH ($COLOR_COUNT entries)"
fi

if $PASS; then
    echo ""
    echo "PASS: Round-trip ANSI fidelity verified."
    exit 0
else
    echo ""
    echo "FAIL: Round-trip mismatch detected."
    exit 1
fi
