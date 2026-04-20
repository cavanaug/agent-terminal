#!/bin/bash
# Template: Multi-session orchestration
# Run multiple TUI apps in parallel and interact with each
#
# Usage: ./multi-session.sh
# Demonstrates parallel session management

set -euo pipefail

echo "=== Multi-Session Orchestration Demo ==="

# Session names
SESSION_SHELL="worker-shell"
SESSION_MONITOR="system-monitor"
SESSION_EDITOR="file-editor"

cleanup() {
  echo ""
  echo "Cleaning up sessions..."
  agent-terminal kill -s "$SESSION_SHELL" 2>/dev/null || true
  agent-terminal kill -s "$SESSION_MONITOR" 2>/dev/null || true
  agent-terminal kill -s "$SESSION_EDITOR" 2>/dev/null || true
  echo "Done."
}

trap cleanup EXIT

# --- 1. Start all sessions ---
echo ""
echo "1. Starting sessions..."

# Shell for running commands
agent-terminal spawn --name "$SESSION_SHELL" bash
echo "   Started: $SESSION_SHELL (bash)"

# System monitor (top is more portable than htop)
agent-terminal spawn --name "$SESSION_MONITOR" top
echo "   Started: $SESSION_MONITOR (top)"

# Editor for a temp file
TEMP_FILE="/tmp/agent-terminal-demo-$$.txt"
agent-terminal spawn --name "$SESSION_EDITOR" vi "$TEMP_FILE"
echo "   Started: $SESSION_EDITOR (vi)"

# Wait for all to be ready
agent-terminal wait-for -s "$SESSION_SHELL" '$' -t 5000 || true
agent-terminal wait-for -s "$SESSION_MONITOR" "load" -t 5000 || agent-terminal wait-for -s "$SESSION_MONITOR" "CPU" -t 5000 || true
agent-terminal wait-for -s "$SESSION_EDITOR" "~" -t 5000 || true

echo "   All sessions ready"

# --- 2. List active sessions ---
echo ""
echo "2. Active sessions:"
agent-terminal list-sessions

# --- 3. Interact with shell ---
echo ""
echo "3. Running command in shell session..."

agent-terminal type -s "$SESSION_SHELL" 'echo "Hello from agent-terminal multi-session demo"'
agent-terminal key -s "$SESSION_SHELL" Enter

# Wait for command to complete
sleep 0.5
agent-terminal wait-for -s "$SESSION_SHELL" '$' -t 5000

# Capture output
echo "   Shell output:"
agent-terminal snapshot -s "$SESSION_SHELL" --format text | tail -5

# --- 4. Check monitor ---
echo ""
echo "4. Checking system monitor..."

agent-terminal snapshot -s "$SESSION_MONITOR" --format text | head -10
echo "   (truncated)"

# --- 5. Write to editor ---
echo ""
echo "5. Writing to editor..."

# Enter insert mode
agent-terminal key -s "$SESSION_EDITOR" i

# Type content
agent-terminal type -s "$SESSION_EDITOR" "# Multi-session demo"
agent-terminal key -s "$SESSION_EDITOR" Enter
agent-terminal type -s "$SESSION_EDITOR" "This file was created by agent-terminal"
agent-terminal key -s "$SESSION_EDITOR" Enter
agent-terminal type -s "$SESSION_EDITOR" "Running $(date)"

# Exit insert mode
agent-terminal key -s "$SESSION_EDITOR" Escape

# Save (but don't quit yet)
agent-terminal type -s "$SESSION_EDITOR" ":w"
agent-terminal key -s "$SESSION_EDITOR" Enter

echo "   Content written to $TEMP_FILE"

# --- 6. Run another shell command ---
echo ""
echo "6. Running another shell command..."

agent-terminal type -s "$SESSION_SHELL" "cat $TEMP_FILE"
agent-terminal key -s "$SESSION_SHELL" Enter

sleep 0.5
agent-terminal wait-for -s "$SESSION_SHELL" '$' -t 5000

echo "   File contents from shell:"
agent-terminal snapshot -s "$SESSION_SHELL" --format text | grep -A5 "Multi-session" || true

# --- 7. Close editor ---
echo ""
echo "7. Closing editor..."

agent-terminal type -s "$SESSION_EDITOR" ":q"
agent-terminal key -s "$SESSION_EDITOR" Enter

sleep 0.5

# --- 8. Stop monitor ---
echo ""
echo "8. Stopping monitor..."

agent-terminal key -s "$SESSION_MONITOR" q

sleep 0.5

# --- 9. Final shell command ---
echo ""
echo "9. Final shell command..."

agent-terminal type -s "$SESSION_SHELL" "echo 'Demo complete!'"
agent-terminal key -s "$SESSION_SHELL" Enter

sleep 0.5

# --- Summary ---
echo ""
echo "=== Demo Summary ==="
echo "Demonstrated:"
echo "  - Starting multiple named sessions"
echo "  - Interacting with each independently"
echo "  - Running commands in a shell session"
echo "  - Monitoring system with top"
echo "  - Editing files with vi"
echo "  - Capturing output from sessions"
echo ""
echo "Sessions will be cleaned up on exit."

# Cleanup handled by trap
