#!/usr/bin/env bash
# Round-trip ANSI fidelity test.
#
# Proves that pilotty's ANSI text output is faithful: a terminal emulator
# (vt100) can parse it back into equivalent screen state.
#
# Pipeline:
#   1. Spawn session A displaying a styled ANSI fixture
#   2. Capture Full JSON snapshot (style_map + color_map) and Text snapshot (ANSI output)
#   3. Strip header line from text output
#   4. Spawn session B displaying the stripped ANSI text
#   5. Capture Full JSON snapshot of session B
#   6. Compare style_map and color_map between A and B
#
# Requirements: bash, jq, cargo (builds pilotty if needed)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FIXTURE="$SCRIPT_DIR/fixtures/roundtrip.ansi"
PILOTTY="$PROJECT_DIR/target/debug/pilotty"

# Temp directory for intermediate files
TMPDIR="$(mktemp -d)"
trap 'cleanup' EXIT

SRC_SESSION="roundtrip-src-$$"
DST_SESSION="roundtrip-dst-$$"

cleanup() {
    # Kill sessions (ignore errors if already dead)
    "$PILOTTY" kill -s "$SRC_SESSION" 2>/dev/null || true
    "$PILOTTY" kill -s "$DST_SESSION" 2>/dev/null || true
    rm -rf "$TMPDIR"
}

die() {
    echo "FAIL: $1" >&2
    exit 1
}

# --- Build ---
echo "Building pilotty..."
(cd "$PROJECT_DIR" && cargo build 2>&1) || die "cargo build failed"

# --- Check fixture exists ---
[ -f "$FIXTURE" ] || die "Missing fixture: $FIXTURE"

# --- Step 1: Spawn session A with the ANSI fixture ---
echo "Spawning source session..."
"$PILOTTY" spawn --name "$SRC_SESSION" --render color -- bash -c "cat '$FIXTURE'; sleep 60" \
    || die "Failed to spawn source session"

# Give the PTY time to render
sleep 0.5

# --- Step 2: Capture snapshots from session A ---
echo "Capturing source snapshots..."
"$PILOTTY" snapshot -s "$SRC_SESSION" --format full --render color > "$TMPDIR/full-A.json" \
    || die "Failed to capture full snapshot A"
"$PILOTTY" snapshot -s "$SRC_SESSION" --format text --render color > "$TMPDIR/text-A.txt" \
    || die "Failed to capture text snapshot A"

# Verify we got style_map data
jq -e '.style_map | length > 0' "$TMPDIR/full-A.json" > /dev/null \
    || die "Source snapshot has no style_map entries"

# --- Step 3: Strip header line, cursor markers, trailing whitespace, trailing blank lines ---
# Header is the first line matching "--- Terminal ... ---"
# Cursor markers [X] → X (preserve the wrapped character to maintain segment lengths)
# Trailing whitespace per line and trailing blank lines are removed
# Final trailing newline is removed to prevent cat from scrolling an N-row PTY
sed '1{/^--- /d}' "$TMPDIR/text-A.txt" \
    | sed 's/\[\(.\)\]/\1/g' \
    | sed 's/[[:space:]]*$//' \
    | sed -e :a -e '/^\n*$/{$d;N;ba}' \
    > "$TMPDIR/stripped-A.txt"

# Remove final trailing newline — cat of N lines into N-row PTY scrolls otherwise
perl -pi -e 'chomp if eof' "$TMPDIR/stripped-A.txt"

echo "Stripped text output: $(wc -l < "$TMPDIR/stripped-A.txt") lines"

# --- Step 4: Spawn session B with the stripped ANSI text ---
echo "Spawning destination session..."
"$PILOTTY" spawn --name "$DST_SESSION" --render color -- bash -c "cat '$TMPDIR/stripped-A.txt'; sleep 60" \
    || die "Failed to spawn destination session"

sleep 0.5

# --- Step 5: Capture full snapshot from session B ---
echo "Capturing destination snapshot..."
"$PILOTTY" snapshot -s "$DST_SESSION" --format full --render color > "$TMPDIR/full-B.json" \
    || die "Failed to capture full snapshot B"

# --- Step 6: Compare style_map and color_map ---
echo "Comparing style maps..."

# Extract and sort style_map entries by (r, c) for stable comparison
jq -S '.style_map // [] | sort_by(.r, .c)' "$TMPDIR/full-A.json" > "$TMPDIR/style-A.json"
jq -S '.style_map // [] | sort_by(.r, .c)' "$TMPDIR/full-B.json" > "$TMPDIR/style-B.json"

# Extract and sort color_map entries
jq -S '.color_map // [] | sort_by(.r, .c)' "$TMPDIR/full-A.json" > "$TMPDIR/color-A.json"
jq -S '.color_map // [] | sort_by(.r, .c)' "$TMPDIR/full-B.json" > "$TMPDIR/color-B.json"

# Compare text content (rows from the .text field, trimmed)
jq -r '.text // ""' "$TMPDIR/full-A.json" | sed 's/[[:space:]]*$//' > "$TMPDIR/text-content-A.txt"
jq -r '.text // ""' "$TMPDIR/full-B.json" | sed 's/[[:space:]]*$//' > "$TMPDIR/text-content-B.txt"

PASS=true

# Compare text content
if ! diff -u "$TMPDIR/text-content-A.txt" "$TMPDIR/text-content-B.txt" > "$TMPDIR/text-diff.txt" 2>&1; then
    echo "TEXT MISMATCH:"
    cat "$TMPDIR/text-diff.txt"
    PASS=false
else
    echo "  Text content: MATCH"
fi

# Compare style_map
if ! diff -u "$TMPDIR/style-A.json" "$TMPDIR/style-B.json" > "$TMPDIR/style-diff.txt" 2>&1; then
    echo "STYLE MAP MISMATCH:"
    cat "$TMPDIR/style-diff.txt"
    PASS=false
else
    STYLE_COUNT=$(jq 'length' "$TMPDIR/style-A.json")
    echo "  Style map: MATCH ($STYLE_COUNT entries)"
fi

# Compare color_map
if ! diff -u "$TMPDIR/color-A.json" "$TMPDIR/color-B.json" > "$TMPDIR/color-diff.txt" 2>&1; then
    echo "COLOR MAP MISMATCH:"
    cat "$TMPDIR/color-diff.txt"
    PASS=false
else
    COLOR_COUNT=$(jq 'length' "$TMPDIR/color-A.json")
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
