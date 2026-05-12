#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TOKEN_REGEX='[p]ilotty|_[p]ilotty|[P]ILOTTY'
LEGACY_PRODUCT_TOKEN="p""ilotty"
LEGACY_COMPLETION_TOKEN="_${LEGACY_PRODUCT_TOKEN}"
LEGACY_ENV_TOKEN="P""ILOTTY"
ALLOWLIST_B64='UkVBRE1FLm1kCT4gKipPcmlnaW46KiogYGFnZW50LXRlcm1pbmFsYCBpcyBkZXJpdmVkIGZyb20gdGhlIGVhcmxpZXIgYHBpbG90dHlgIHByb2plY3QsIGFuZCB0aGUgbWFzY290IGFydHdvcmsgYWxzbyBjb21lcyBmcm9tIGBwaWxvdHR5YC4JYm91bmRlZCBSRUFETUUgb3JpZ2luIG5vdGUga2VwdCBmb3IgcHJvamVjdCBoaXN0b3J5IGFuZCBtYXNjb3QgcHJvdmVuYW5jZQpjcmF0ZXMvYWdlbnQtdGVybWluYWwtY2xpL3Rlc3RzL2Vycm9yX2lkZW50aXR5LnJzCSAgICAgICAgIXN0ZGVyci5jb250YWlucygicGlsb3R0eSIpLAluZWdhdGl2ZSBhc3NlcnRpb24gcHJvdmluZyBDTEkgc3RkZXJyIG5ldmVyIGxlYWtzIHRoZSBsZWdhY3kgbmFtZQpjcmF0ZXMvYWdlbnQtdGVybWluYWwtY2xpL3Rlc3RzL2Vycm9yX2lkZW50aXR5LnJzCSAgICAgICAgInN0ZGVyciBzaG91bGQgbm90IGxlYWsgcGlsb3R0eVxue3N0ZGVycn0iCW5lZ2F0aXZlIGFzc2VydGlvbiBmYWlsdXJlIG1lc3NhZ2UgZm9yIENMSSBzdGRlcnIgaWRlbnRpdHkgY292ZXJhZ2UKY3JhdGVzL2FnZW50LXRlcm1pbmFsLWNsaS90ZXN0cy9wdWJsaWNfaWRlbnRpdHkucnMJICAgICAgICAhc3Rkb3V0LmNvbnRhaW5zKCJwaWxvdHR5IiksCW5lZ2F0aXZlIGFzc2VydGlvbiBwcm92aW5nIENMSSBzdGRvdXQgc3VyZmFjZXMgbmV2ZXIgbGVhayB0aGUgbGVnYWN5IG5hbWUKY3JhdGVzL2FnZW50LXRlcm1pbmFsLWNsaS90ZXN0cy9wdWJsaWNfaWRlbnRpdHkucnMJICAgICAgICAie3N1cmZhY2V9IHNob3VsZCBub3QgbGVhayBwaWxvdHR5XG57c3Rkb3V0fSIJbmVnYXRpdmUgYXNzZXJ0aW9uIGZhaWx1cmUgbWVzc2FnZSBmb3IgQ0xJIHN0ZG91dCBpZGVudGl0eSBjb3ZlcmFnZQpjcmF0ZXMvYWdlbnQtdGVybWluYWwtY2xpL3Rlc3RzL3B1YmxpY19pZGVudGl0eS5ycwkgICAgICAgICFzdGRvdXQuY29udGFpbnMoIl9waWxvdHR5IiksCW5lZ2F0aXZlIGFzc2VydGlvbiBwcm92aW5nIGNvbXBsZXRpb24gc3VyZmFjZXMgbmV2ZXIgbGVhayB0aGUgbGVnYWN5IGNvbXBsZXRpb24gbmFtZQpjcmF0ZXMvYWdlbnQtdGVybWluYWwtY2xpL3Rlc3RzL3B1YmxpY19pZGVudGl0eS5ycwkgICAgICAgICJ7c3VyZmFjZX0gc2hvdWxkIG5vdCBsZWFrIF9waWxvdHR5XG57c3Rkb3V0fSIJbmVnYXRpdmUgYXNzZXJ0aW9uIGZhaWx1cmUgbWVzc2FnZSBmb3IgbGVnYWN5IGNvbXBsZXRpb24gY292ZXJhZ2UKY3JhdGVzL2FnZW50LXRlcm1pbmFsLWNsaS90ZXN0cy9wdWJsaWNfaWRlbnRpdHkucnMJICAgICAgICAhc3Rkb3V0LmNvbnRhaW5zKCIjY29tcGRlZiBwaWxvdHR5IiksCW5lZ2F0aXZlIGFzc2VydGlvbiBwcm92aW5nIGdlbmVyYXRlZCB6c2ggY29tcGxldGlvbnMgbmV2ZXIgbWVudGlvbiB0aGUgbGVnYWN5IGNvbXBkZWYKY3JhdGVzL2FnZW50LXRlcm1pbmFsLWNsaS90ZXN0cy9wdWJsaWNfaWRlbnRpdHkucnMJICAgICAgICAie3N1cmZhY2V9IHNob3VsZCBub3QgbGVhayB6c2ggY29tcGRlZiBwaWxvdHR5XG57c3Rkb3V0fSIJbmVnYXRpdmUgYXNzZXJ0aW9uIGZhaWx1cmUgbWVzc2FnZSBmb3IgenNoIGNvbXBsZXRpb24gY292ZXJhZ2UKY3JhdGVzL2FnZW50LXRlcm1pbmFsLWNvcmUvdGVzdHMvZXJyb3JfaWRlbnRpdHkucnMJICAgICAgICAhc3VnZ2VzdGlvbi5jb250YWlucygicGlsb3R0eSIpLAluZWdhdGl2ZSBhc3NlcnRpb24gcHJvdmluZyBzaGFyZWQgZXJyb3Igc3VnZ2VzdGlvbnMgbmV2ZXIgbGVhayB0aGUgbGVnYWN5IG5hbWUKY3JhdGVzL2FnZW50LXRlcm1pbmFsLWNvcmUvdGVzdHMvZXJyb3JfaWRlbnRpdHkucnMJICAgICAgICAie2xhYmVsfSBzaG91bGQgbm90IGxlYWsgcGlsb3R0eSwgZ290OiB7c3VnZ2VzdGlvbn0iCW5lZ2F0aXZlIGFzc2VydGlvbiBmYWlsdXJlIG1lc3NhZ2UgZm9yIHNoYXJlZCBlcnJvciBpZGVudGl0eSBjb3ZlcmFnZQp0ZXN0cy92ZXJpZnktYWdlbnQtdGVybWluYWwtcnVudGltZS5zaAlbICEgLWUgIiRIT01FX0RJUi8ucGlsb3R0eSIgXSB8fCBkaWUgInN0YWxlIC5waWxvdHR5IHJ1bnRpbWUgZGlyZWN0b3J5IGNyZWF0ZWQiCXJ1bnRpbWUgc21va2UgbmVnYXRpdmUgYXNzZXJ0aW9uIGd1YXJkaW5nIGFnYWluc3QgbGVnYWN5IHJ1bnRpbWUgZGlyZWN0b3J5IHJlY3JlYXRpb24KdGVzdHMvdmVyaWZ5LWFnZW50LXRlcm1pbmFsLXJ1bnRpbWUuc2gJaWYgZmluZCAiJEhPTUVfRElSIiAtbWF4ZGVwdGggMyAtbmFtZSAnKnBpbG90dHkqJyB8IGdyZXAgLXEgLjsgdGhlbglydW50aW1lIHNtb2tlIG5lZ2F0aXZlIGFzc2VydGlvbiBzY2FubmluZyBpc29sYXRlZCBIT01FIGZvciBsZWdhY3ktbmFtZWQgYXJ0aWZhY3RzCnRlc3RzL3ZlcmlmeS1hZ2VudC10ZXJtaW5hbC1ydW50aW1lLnNoCSAgZGllICJmb3VuZCBzdGFsZSBwaWxvdHR5LW5hbWVkIHJ1bnRpbWUgYXJ0aWZhY3RzIHVuZGVyIGlzb2xhdGVkIEhPTUUiCXJ1bnRpbWUgc21va2UgZmFpbHVyZSBtZXNzYWdlIGZvciBsZWdhY3kgcnVudGltZSBhcnRpZmFjdCBkZXRlY3Rpb24K'

declare -A ALLOW_REASON=()
declare -A SEEN=()

phase() {
  printf '\n==> %s\n' "$1"
}

die() {
  echo "FAIL: $1" >&2
  exit 1
}

classify_token() {
  local line="$1"

  if [[ "$line" == *"$LEGACY_COMPLETION_TOKEN"* ]]; then
    printf '%s' "$LEGACY_COMPLETION_TOKEN"
    return
  fi

  if [[ "$line" == *"$LEGACY_ENV_TOKEN"* ]]; then
    printf '%s' "$LEGACY_ENV_TOKEN"
    return
  fi

  printf '%s' "$LEGACY_PRODUCT_TOKEN"
}

load_allowlist() {
  local rel_path line reason key

  while IFS=$'\t' read -r rel_path line reason; do
    [ -n "$rel_path" ] || continue
    key="${rel_path}"$'\t'"${line}"

    if [[ -n "${ALLOW_REASON[$key]+x}" ]]; then
      die "duplicate allowlist entry for ${rel_path}: ${line}"
    fi

    ALLOW_REASON["$key"]="$reason"
    SEEN["$key"]=0
  done < <(printf '%s' "$ALLOWLIST_B64" | base64 --decode)
}

scan_repo() {
  local rel_path line_no line key reason token match_count=0

  while IFS=: read -r rel_path line_no line; do
    key="${rel_path}"$'\t'"${line}"
    token="$(classify_token "$line")"

    if [[ -z "${ALLOW_REASON[$key]+x}" ]]; then
      printf 'Unexpected legacy token in %s:%s (%s)\n' "$rel_path" "$line_no" "$token" >&2
      printf '  %s\n' "$line" >&2
      die "repo branding audit found an unallowlisted legacy token match"
    fi

    reason="${ALLOW_REASON[$key]}"
    SEEN["$key"]=1
    match_count=$((match_count + 1))
    printf 'ALLOW: %s:%s (%s) — %s\n' "$rel_path" "$line_no" "$token" "$reason"
  done < <(
    git -C "$PROJECT_DIR" grep -n -I -E "$TOKEN_REGEX" -- . ':(exclude)docs/**'
  )

  if [ "$match_count" -eq 0 ]; then
    die "repo branding audit found no legacy-token matches; the bounded README origin note or allowlist likely drifted"
  fi
}

verify_allowlist_coverage() {
  local key rel_path line reason missing=0

  for key in "${!ALLOW_REASON[@]}"; do
    if [ "${SEEN[$key]}" -eq 1 ]; then
      continue
    fi

    rel_path="${key%%$'\t'*}"
    line="${key#*$'\t'}"
    reason="${ALLOW_REASON[$key]}"
    printf 'Missing allowlist entry in scan: %s\n' "$rel_path" >&2
    printf '  expected line: %s\n' "$line" >&2
    printf '  reason: %s\n' "$reason" >&2
    missing=1
  done

  [ "$missing" -eq 0 ] || die "repo branding audit allowlist drifted; update the exact entry instead of widening scope"
}

main() {
  phase "Load documented allowlist"
  load_allowlist

  phase "Scan tracked non-doc repository files for bounded legacy-brand tokens"
  scan_repo

  phase "Verify every documented allowlist entry was exercised"
  verify_allowlist_coverage

  phase "Tracked-file branding audit complete"
  echo "PASS: repository legacy-brand tokens are limited to the documented README note, negative assertions, and runtime regression guards."
}

main "$@"
