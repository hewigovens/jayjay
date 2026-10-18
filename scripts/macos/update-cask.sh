#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

version="$1"
channel="$2"
build_number="$3"

if [ "${channel}" = "beta" ]; then
  echo "Beta release: skipping Homebrew cask update."
  exit 0
fi
zip_path="${release_dir}/${app_name}-${version}.zip"
if [ ! -f "$zip_path" ]; then
  echo "Expected archive at $zip_path. Run 'just shell::release' first." >&2
  exit 1
fi
cask_path="${root}/../tap/Casks/jayjay.rb"
template="${root}/dist/jayjay.rb.template"
if [ ! -d "$(dirname "$cask_path")" ]; then
  echo "Tap not found at $(dirname "$cask_path") — skipping." >&2
  exit 0
fi
sha256=$(shasum -a 256 "$zip_path" | cut -d' ' -f1)
# Render the whole cask from the in-repo template so its structure is reviewed here, not hand-edited in the tap.
sed -e "s/__VERSION__/${version},${build_number}/" \
    -e "s|__SHA256__|$sha256|" \
    "$template" > "$cask_path"
echo "Rendered $cask_path from template (version ${version},${build_number}, sha256 $sha256)"
