#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 /path/to/JayJay.app [/path/to/jayjay-cli]" >&2
  exit 2
fi

app_path="$1"
cli_source="${2:-}"
macos_dir="$app_path/Contents/MacOS"
resources_dir="$app_path/Contents/Resources"
main_executable="$macos_dir/JayJay"
cli_path="$macos_dir/jayjay-cli"

if [[ ! -x "$main_executable" ]]; then
  echo "JayJay executable not found or not executable: $main_executable" >&2
  exit 1
fi

if [[ -n "$cli_source" ]]; then
  if [[ ! -x "$cli_source" ]]; then
    echo "jayjay CLI launcher not found or not executable: $cli_source" >&2
    exit 1
  fi
  cp "$cli_source" "$cli_path"
  chmod 755 "$cli_path"
fi

if [[ "${JAYJAY_STRIP_APP:-0}" == "1" ]]; then
  codesign --remove-signature "$main_executable" 2>/dev/null || true
  xcrun strip -x "$main_executable"
fi

stale_resource_bundles=(
  "swiftui-math_SwiftUIMath.bundle"
  "SwiftUIMath_SwiftUIMath.bundle"
  "textual_Textual.bundle"
  "Textual_Textual.bundle"
)

for bundle_name in "${stale_resource_bundles[@]}"; do
  rm -rf "$resources_dir/$bundle_name"
done
