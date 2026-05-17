#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
    echo "Usage: verify-release-archive.sh <zip_path> [signed|notarized]" >&2
    exit 2
fi

zip_path="$1"
mode="${2:-signed}"

case "$mode" in
    signed | notarized) ;;
    *)
        echo "Error: mode must be 'signed' or 'notarized', got '$mode'" >&2
        exit 2
        ;;
esac

if [[ ! -f "$zip_path" ]]; then
    echo "Error: archive not found: $zip_path" >&2
    exit 1
fi

zip_listing="$(zipinfo -1 "$zip_path")"
metadata_entry="$(
    printf '%s\n' "$zip_listing" | grep -E '(^|/)\._|(^|/)__MACOSX(/|$)' | head -n 1 || true
)"
if [[ -n "$metadata_entry" ]]; then
    echo "Error: archive contains AppleDouble metadata entry: $metadata_entry" >&2
    exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

ditto -x -k "$zip_path" "$tmp_dir"

apps=()
while IFS= read -r app; do
    apps+=("$app")
done < <(find "$tmp_dir" -maxdepth 1 -type d -name '*.app' -print)

if [[ "${#apps[@]}" -ne 1 ]]; then
    echo "Error: expected one top-level .app in archive, found ${#apps[@]}" >&2
    exit 1
fi

app_path="${apps[0]}"

appledouble_file="$(find "$app_path" -name '._*' -print -quit)"
if [[ -n "$appledouble_file" ]]; then
    echo "Error: extracted app contains AppleDouble file: $appledouble_file" >&2
    exit 1
fi

macosx_dir="$(find "$app_path" -name '__MACOSX' -type d -print -quit)"
if [[ -n "$macosx_dir" ]]; then
    echo "Error: extracted app contains metadata directory: $macosx_dir" >&2
    exit 1
fi

codesign --verify --deep --strict --verbose=2 "$app_path"

if [[ "$mode" == "notarized" ]]; then
    xcrun stapler validate "$app_path"
    spctl -av "$app_path"
fi

echo "Verified $zip_path ($mode)"
