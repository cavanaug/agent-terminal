#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RELEASE_WORKFLOW="$PROJECT_DIR/.github/workflows/release.yml"

phase() {
  printf '\n==> %s\n' "$1"
}

die() {
  echo "FAIL: $1" >&2
  exit 1
}

require_file() {
  local path="$1"
  [ -f "$path" ] || die "missing expected file: $path"
}

require_contains() {
  local path="$1"
  local needle="$2"
  local label="$3"

  if ! rg -n --fixed-strings "$needle" "$path"; then
    die "missing expected ${label}: ${needle}"
  fi
}

require_absent() {
  local path="$1"
  local needle="$2"
  local label="$3"

  if rg -n --fixed-strings "$needle" "$path"; then
    die "stale ${label} found in ${path}: ${needle}"
  fi
}

require_path_absent() {
  local path="$1"

  if [ -e "$path" ]; then
    find "$path" -maxdepth 3 -print 2>/dev/null | sort >&2 || true
    die "stale path still exists: ${path}"
  fi
}

main() {
  local mode="${1:-}"

  case "$mode" in
    ""|--distribution-only)
      ;;
    --help|-h)
      cat <<'EOF'
Usage: bash tests/verify-docs-release-identity.sh [--distribution-only]

Current checks:
  --distribution-only   Verify npm release plumbing is disabled while GitHub
                        tarball + completion release surfaces remain.
  (default)            Same as --distribution-only for now; T02 expands this.
EOF
      exit 0
      ;;
    *)
      die "unknown mode: $mode"
      ;;
  esac

  phase "Check release workflow exists"
  require_file "$RELEASE_WORKFLOW"

  phase "Check GitHub release tarball and completion flow remains"
  require_contains "$RELEASE_WORKFLOW" "name: agent-terminal-completions" "completions artifact upload"
  require_contains "$RELEASE_WORKFLOW" "pattern: agent-terminal-*" "release artifact download pattern"
  require_contains "$RELEASE_WORKFLOW" "agent-terminal-completions.tar.gz" "completion packaging output"
  require_contains "$RELEASE_WORKFLOW" 'name: agent-terminal-${{ matrix.target }}' "tarball upload"

  phase "Check disabled npm workflow tokens stay absent"
  require_absent "$RELEASE_WORKFLOW" "npm_arch" "npm matrix field"
  require_absent "$RELEASE_WORKFLOW" "Upload npm binary" "raw npm upload step"
  require_absent "$RELEASE_WORKFLOW" "EXPECTED_NPM" "npm verification block"
  require_absent "$RELEASE_WORKFLOW" "publish-npm:" "publish job"
  require_absent "$RELEASE_WORKFLOW" "npm publish" "npm publish command"
  require_absent "$RELEASE_WORKFLOW" "version-sync" "version sync script reference"
  require_absent "$RELEASE_WORKFLOW" "path: npm/bin" "npm artifact download path"
  require_absent "$RELEASE_WORKFLOW" "pattern: npm-*" "npm artifact download pattern"

  phase "Check deleted npm package paths stay absent"
  require_path_absent "$PROJECT_DIR/npm"
  require_path_absent "$PROJECT_DIR/scripts/version-sync.sh"

  phase "Distribution identity audit complete"
  echo "PASS: npm distribution is disabled and GitHub release artifacts remain intact."
}

main "$@"
