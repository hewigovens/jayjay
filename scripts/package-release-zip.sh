#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "Usage: package-release-zip.sh <app_path> <zip_path>" >&2
    exit 2
fi

app_path="$1"
zip_path="$2"

if [[ ! -d "$app_path" ]]; then
    echo "Error: app bundle not found: $app_path" >&2
    exit 1
fi

appledouble_file="$(find "$app_path" -name '._*' -print -quit)"
if [[ -n "$appledouble_file" ]]; then
    echo "Error: refusing to package AppleDouble file inside app bundle: $appledouble_file" >&2
    exit 1
fi

macosx_dir="$(find "$app_path" -name '__MACOSX' -type d -print -quit)"
if [[ -n "$macosx_dir" ]]; then
    echo "Error: refusing to package metadata directory inside app bundle: $macosx_dir" >&2
    exit 1
fi

mkdir -p "$(dirname "$zip_path")"
rm -f "$zip_path"

# --norsrc is required for signed app bundles: default ditto zip mode stores
# resource-fork metadata as ._* entries, which can break nested framework seals
# after extraction.
COPYFILE_DISABLE=1 ditto -c -k --norsrc --keepParent "$app_path" "$zip_path"

zip_listing="$(zipinfo -1 "$zip_path")"
metadata_entry="$(
    printf '%s\n' "$zip_listing" | grep -E '(^|/)\._|(^|/)__MACOSX(/|$)' | head -n 1 || true
)"
if [[ -n "$metadata_entry" ]]; then
    echo "Error: archive contains AppleDouble metadata entry: $metadata_entry" >&2
    exit 1
fi

echo "Created $zip_path ($(du -sh "$zip_path" | cut -f1))"
shasum -a 256 "$zip_path"
