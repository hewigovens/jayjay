#!/usr/bin/env bash
set -euo pipefail

existing_pr=$(gh pr list --repo microsoft/winget-pkgs --state open \
  --search '"hewig.JayJay" in:title' --limit 1 --json url --jq '.[0].url')
if [[ -n "$existing_pr" ]]; then
  echo "Skipping WinGet submission: $existing_pr is still open."
  echo 'submit=false' >> "$GITHUB_OUTPUT"
else
  echo 'submit=true' >> "$GITHUB_OUTPUT"
fi
