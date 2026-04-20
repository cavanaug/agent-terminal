#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RELEASE_WORKFLOW="$PROJECT_DIR/.github/workflows/release.yml"
README="$PROJECT_DIR/README.md"
NEW_HERO_ASSET="$PROJECT_DIR/assets/agent-terminal.png"
LEGACY_PRODUCT_NAME="p""ilotty"
LEGACY_HERO_ASSET="$PROJECT_DIR/assets/${LEGACY_PRODUCT_NAME}.png"
printf -v ORIGIN_NOTE '> **Origin:** `%s` is derived from the earlier `%s` project, and the mascot artwork also comes from `%s`.' 'agent-terminal' "$LEGACY_PRODUCT_NAME" "$LEGACY_PRODUCT_NAME"

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

check_distribution_identity() {
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
}

check_readme_identity() {
  local readme_without_origin
  local legacy_origin_line_count
  local legacy_completion_name
  local legacy_env_prefix
  local legacy_runtime_path
  local legacy_repo_slug
  local legacy_asset_path
  readme_without_origin="$(mktemp "${TMPDIR:-/tmp}/agent-terminal-readme.XXXXXX")"
  legacy_completion_name="_${LEGACY_PRODUCT_NAME}"
  legacy_env_prefix="P""ILOTTY_"
  legacy_runtime_path=".${LEGACY_PRODUCT_NAME}"
  legacy_repo_slug="cavanaug/${LEGACY_PRODUCT_NAME}"
  legacy_asset_path="assets/${LEGACY_PRODUCT_NAME}.png"

  phase "Check README and hero asset exist"
  require_file "$README"
  require_file "$NEW_HERO_ASSET"
  require_path_absent "$LEGACY_HERO_ASSET"

  phase "Check README active identity surfaces"
  require_contains "$README" '<img src="assets/agent-terminal.png"' "renamed hero asset reference"
  require_contains "$README" '<h1 align="center">agent-terminal</h1>' "agent-terminal heading"
  require_contains "$README" 'agent-terminal --help' "help command example"
  require_contains "$README" 'agent-terminal spawn <command>' "spawn example"
  require_contains "$README" 'agent-terminal snapshot --await-change $HASH --settle 100' "await-change example"
  require_contains "$README" 'AGENT_TERMINAL_SESSION' "runtime session env var"
  require_contains "$README" 'AGENT_TERMINAL_SOCKET_DIR' "runtime socket env var"
  require_contains "$README" '~/.agent-terminal/{session}.sock' "home runtime path"
  require_contains "$README" 'npm distribution is intentionally disabled for now.' "no-npm distribution note"

  phase "Check bounded origin note"
  require_contains "$README" "$ORIGIN_NOTE" "bounded origin note"
  legacy_origin_line_count="$(rg -n --fixed-strings "$LEGACY_PRODUCT_NAME" "$README" | wc -l | tr -d ' ')"
  if [ "$legacy_origin_line_count" != "1" ]; then
    rg -n --fixed-strings "$LEGACY_PRODUCT_NAME" "$README" >&2 || true
    die "expected exactly one README line containing '${LEGACY_PRODUCT_NAME}', found ${legacy_origin_line_count}"
  fi

  grep -Fv "$ORIGIN_NOTE" "$README" > "$readme_without_origin"

  phase "Check stale branding stays absent outside the origin note"
  require_absent "$readme_without_origin" "$LEGACY_PRODUCT_NAME" "legacy product naming"
  require_absent "$readme_without_origin" "$legacy_completion_name" "legacy completion naming"
  require_absent "$readme_without_origin" "$legacy_env_prefix" "legacy runtime env vars"
  require_absent "$readme_without_origin" "$legacy_runtime_path" "legacy runtime path"
  require_absent "$readme_without_origin" "$legacy_repo_slug" "legacy repository slug"
  require_absent "$readme_without_origin" "$legacy_asset_path" "legacy hero asset path"
  require_absent "$readme_without_origin" 'npm install -g' "stale npm install guidance"

  rm -f "$readme_without_origin"
}

main() {
  local mode="${1:-}"

  case "$mode" in
    ""|--distribution-only)
      ;;
    --help|-h)
      cat <<'EOF'
Usage: bash tests/verify-docs-release-identity.sh [--distribution-only]

Modes:
  --distribution-only   Verify npm release plumbing is disabled while GitHub
                        tarball + completion release surfaces remain.
  (default)             Run the distribution checks plus README/asset identity
                        checks for the public agent-terminal docs surface.
EOF
      exit 0
      ;;
    *)
      die "unknown mode: $mode"
      ;;
  esac

  check_distribution_identity

  if [ "$mode" = "--distribution-only" ]; then
    phase "Distribution identity audit complete"
    echo "PASS: npm distribution is disabled and GitHub release artifacts remain intact."
    exit 0
  fi

  check_readme_identity

  phase "Full docs/release identity audit complete"
  echo "PASS: docs, hero asset, and release surfaces consistently use the agent-terminal identity."
}

main "$@"
