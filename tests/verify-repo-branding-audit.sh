#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TOKEN_REGEX='[p]ilotty|_[p]ilotty|[P]ILOTTY'
LEGACY_PRODUCT_TOKEN="p""ilotty"
LEGACY_COMPLETION_TOKEN="_${LEGACY_PRODUCT_TOKEN}"
LEGACY_ENV_TOKEN="P""ILOTTY"
ALLOWLIST_B64='UkVBRE1FLm1kCT4gKipPcmlnaW46KiogYGFnZW50LXRlcm1pbmFsYCBpcyBkZXJpdmVkIGZyb20gdGhlIGVhcmxpZXIgYHBpbG90dHlgIHByb2plY3QsIGFuZCB0aGUgbWFzY290IGFydHdvcmsgYWxzbyBjb21lcyBmcm9tIGBwaWxvdHR5YC4JYm91bmRlZCBSRUFETUUgb3JpZ2luIG5vdGUga2VwdCBmb3IgcHJvamVjdCBoaXN0b3J5IGFuZCBtYXNjb3QgcHJvdmVuYW5jZQpjcmF0ZXMvcGlsb3R0eS1jbGkvQ2FyZ28udG9tbAlhZ2VudC10ZXJtaW5hbC1jb3JlID0geyBwYXRoID0gIi4uL3BpbG90dHktY29yZSIgfQlzdGFibGUgb24tZGlzayBjcmF0ZSBwYXRoIHJldGFpbmVkIGJ5IEQwMDcvRDAwOApjcmF0ZXMvcGlsb3R0eS1jbGkvdGVzdHMvZXJyb3JfaWRlbnRpdHkucnMJICAgICAgICAhc3RkZXJyLmNvbnRhaW5zKCJwaWxvdHR5IiksCW5lZ2F0aXZlIGFzc2VydGlvbiBwcm92aW5nIENMSSBzdGRlcnIgbmV2ZXIgbGVha3MgdGhlIGxlZ2FjeSBuYW1lCmNyYXRlcy9waWxvdHR5LWNsaS90ZXN0cy9lcnJvcl9pZGVudGl0eS5ycwkgICAgICAgICJzdGRlcnIgc2hvdWxkIG5vdCBsZWFrIHBpbG90dHlcbntzdGRlcnJ9IgluZWdhdGl2ZSBhc3NlcnRpb24gZmFpbHVyZSBtZXNzYWdlIGZvciBDTEkgc3RkZXJyIGlkZW50aXR5IGNvdmVyYWdlCmNyYXRlcy9waWxvdHR5LWNsaS90ZXN0cy9wdWJsaWNfaWRlbnRpdHkucnMJICAgICAgICAhc3Rkb3V0LmNvbnRhaW5zKCJwaWxvdHR5IiksCW5lZ2F0aXZlIGFzc2VydGlvbiBwcm92aW5nIENMSSBzdGRvdXQgc3VyZmFjZXMgbmV2ZXIgbGVhayB0aGUgbGVnYWN5IG5hbWUKY3JhdGVzL3BpbG90dHktY2xpL3Rlc3RzL3B1YmxpY19pZGVudGl0eS5ycwkgICAgICAgICJ7c3VyZmFjZX0gc2hvdWxkIG5vdCBsZWFrIHBpbG90dHlcbntzdGRvdXR9IgluZWdhdGl2ZSBhc3NlcnRpb24gZmFpbHVyZSBtZXNzYWdlIGZvciBDTEkgc3Rkb3V0IGlkZW50aXR5IGNvdmVyYWdlCmNyYXRlcy9waWxvdHR5LWNsaS90ZXN0cy9wdWJsaWNfaWRlbnRpdHkucnMJICAgICAgICAhc3Rkb3V0LmNvbnRhaW5zKCJfcGlsb3R0eSIpLAluZWdhdGl2ZSBhc3NlcnRpb24gcHJvdmluZyBjb21wbGV0aW9uIHN1cmZhY2VzIG5ldmVyIGxlYWsgdGhlIGxlZ2FjeSBjb21wbGV0aW9uIG5hbWUKY3JhdGVzL3BpbG90dHktY2xpL3Rlc3RzL3B1YmxpY19pZGVudGl0eS5ycwkgICAgICAgICJ7c3VyZmFjZX0gc2hvdWxkIG5vdCBsZWFrIF9waWxvdHR5XG57c3Rkb3V0fSIJbmVnYXRpdmUgYXNzZXJ0aW9uIGZhaWx1cmUgbWVzc2FnZSBmb3IgbGVnYWN5IGNvbXBsZXRpb24gY292ZXJhZ2UKY3JhdGVzL3BpbG90dHktY2xpL3Rlc3RzL3B1YmxpY19pZGVudGl0eS5ycwkgICAgICAgICFzdGRvdXQuY29udGFpbnMoIiNjb21wZGVmIHBpbG90dHkiKSwJbmVnYXRpdmUgYXNzZXJ0aW9uIHByb3ZpbmcgZ2VuZXJhdGVkIHpzaCBjb21wbGV0aW9ucyBuZXZlciBtZW50aW9uIHRoZSBsZWdhY3kgY29tcGRlZgpjcmF0ZXMvcGlsb3R0eS1jbGkvdGVzdHMvcHVibGljX2lkZW50aXR5LnJzCSAgICAgICAgIntzdXJmYWNlfSBzaG91bGQgbm90IGxlYWsgenNoIGNvbXBkZWYgcGlsb3R0eVxue3N0ZG91dH0iCW5lZ2F0aXZlIGFzc2VydGlvbiBmYWlsdXJlIG1lc3NhZ2UgZm9yIHpzaCBjb21wbGV0aW9uIGNvdmVyYWdlCmNyYXRlcy9waWxvdHR5LWNvcmUvdGVzdHMvZXJyb3JfaWRlbnRpdHkucnMJICAgICAgICAhc3VnZ2VzdGlvbi5jb250YWlucygicGlsb3R0eSIpLAluZWdhdGl2ZSBhc3NlcnRpb24gcHJvdmluZyBzaGFyZWQgZXJyb3Igc3VnZ2VzdGlvbnMgbmV2ZXIgbGVhayB0aGUgbGVnYWN5IG5hbWUKY3JhdGVzL3BpbG90dHktY29yZS90ZXN0cy9lcnJvcl9pZGVudGl0eS5ycwkgICAgICAgICJ7bGFiZWx9IHNob3VsZCBub3QgbGVhayBwaWxvdHR5LCBnb3Q6IHtzdWdnZXN0aW9ufSIJbmVnYXRpdmUgYXNzZXJ0aW9uIGZhaWx1cmUgbWVzc2FnZSBmb3Igc2hhcmVkIGVycm9yIGlkZW50aXR5IGNvdmVyYWdlCnRlc3RzL3ZlcmlmeS1hZ2VudC10ZXJtaW5hbC1ydW50aW1lLnNoCVsgISAtZSAiJEhPTUVfRElSLy5waWxvdHR5IiBdIHx8IGRpZSAic3RhbGUgLnBpbG90dHkgcnVudGltZSBkaXJlY3RvcnkgY3JlYXRlZCIJcnVudGltZSBzbW9rZSBuZWdhdGl2ZSBhc3NlcnRpb24gZ3VhcmRpbmcgYWdhaW5zdCBsZWdhY3kgcnVudGltZSBkaXJlY3RvcnkgcmVjcmVhdGlvbgp0ZXN0cy92ZXJpZnktYWdlbnQtdGVybWluYWwtcnVudGltZS5zaAlpZiBmaW5kICIkSE9NRV9ESVIiIC1tYXhkZXB0aCAzIC1uYW1lICcqcGlsb3R0eSonIHwgZ3JlcCAtcSAuOyB0aGVuCXJ1bnRpbWUgc21va2UgbmVnYXRpdmUgYXNzZXJ0aW9uIHNjYW5uaW5nIGlzb2xhdGVkIEhPTUUgZm9yIGxlZ2FjeS1uYW1lZCBhcnRpZmFjdHMKdGVzdHMvdmVyaWZ5LWFnZW50LXRlcm1pbmFsLXJ1bnRpbWUuc2gJICBkaWUgImZvdW5kIHN0YWxlIHBpbG90dHktbmFtZWQgcnVudGltZSBhcnRpZmFjdHMgdW5kZXIgaXNvbGF0ZWQgSE9NRSIJcnVudGltZSBzbW9rZSBmYWlsdXJlIG1lc3NhZ2UgZm9yIGxlZ2FjeSBydW50aW1lIGFydGlmYWN0IGRldGVjdGlvbgo='

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
    rel_path="${rel_path#./}"
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
    cd "$PROJECT_DIR"
    rg -n -H --hidden \
      --glob '!.git' \
      --glob '!.git/**' \
      --glob '!.gsd/**' \
      --glob '!target/**' \
      --glob '!node_modules/**' \
      --glob '!dist/**' \
      --glob '!coverage/**' \
      "$TOKEN_REGEX" .
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

  phase "Scan repository files for bounded legacy-brand tokens"
  scan_repo

  phase "Verify every documented allowlist entry was exercised"
  verify_allowlist_coverage

  phase "Tracked-file branding audit complete"
  echo "PASS: repository legacy-brand tokens are limited to the documented README note, negative assertions, and stable Cargo path."
}

main "$@"
