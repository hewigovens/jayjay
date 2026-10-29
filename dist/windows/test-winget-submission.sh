#!/usr/bin/env bash
set -euo pipefail

check_script=${1:-"$(dirname "$0")/winget/check-open-pr.sh"}
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
export GITHUB_OUTPUT="$scratch/output"

gh() {
  if [[ "$LOOKUP_RESULT" == error ]]; then
    echo 'GitHub lookup failed' >&2
    return 1
  fi
  printf '%s\n' "$LOOKUP_RESULT"
}
export -f gh

export LOOKUP_RESULT=https://github.com/microsoft/winget-pkgs/pull/464518
bash "$check_script" > "$scratch/log"
grep -qx 'submit=false' "$GITHUB_OUTPUT"
grep -Fq "$LOOKUP_RESULT" "$scratch/log"

export LOOKUP_RESULT=''
: > "$GITHUB_OUTPUT"
bash "$check_script"
grep -qx 'submit=true' "$GITHUB_OUTPUT"

export LOOKUP_RESULT=error
: > "$GITHUB_OUTPUT"
if bash "$check_script" > "$scratch/log" 2>&1; then
  echo 'A failed lookup must stop submission' >&2
  exit 1
fi
test ! -s "$GITHUB_OUTPUT"
echo 'PASS: existing PR skips with URL, no PR permits submission, lookup failure stops submission'
