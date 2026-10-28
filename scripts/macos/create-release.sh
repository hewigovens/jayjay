#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

version="$1"
channel="$2"

swiftui_notes="${root}/releases/${version}.html"
gpui_notes="${root}/releases/${version}-gpui.md"
if [ ! -s "$swiftui_notes" ]; then
  echo "Release notes are missing or empty: $swiftui_notes" >&2
  exit 1
fi
announcement="${root}/releases/${version}-discord.md"
if [ "$channel" = "stable" ] && [ ! -s "$announcement" ]; then
  echo "Discord announcement is missing or empty: $announcement" >&2
  exit 1
fi
github_notes="$swiftui_notes"
if [ -s "$gpui_notes" ]; then
  github_notes=$(mktemp)
  trap 'rm -f "$github_notes"' EXIT
  {
    printf '<h2>macOS</h2>\n\n'
    cat "$swiftui_notes"
    printf '\n\n<h2>Linux GPUI</h2>\n\n'
    cat "$gpui_notes"
  } >"$github_notes"
fi

if [ "${channel}" = "beta" ]; then prerelease_flag="--prerelease=true"; else prerelease_flag="--prerelease=false"; fi
if gh release view "v${version}" &>/dev/null; then
  gh release edit "v${version}" --notes-file "$github_notes" "$prerelease_flag"
  echo "Updated release notes for v${version}"
else
  gh release create "v${version}" --verify-tag --title "v${version}" --notes-file "$github_notes" "$prerelease_flag"
fi
