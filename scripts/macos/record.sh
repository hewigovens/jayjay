#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

cd "${root}"
swift build -c release --package-path video --product director
shopt -s nullglob
paths=(video/scenes/[!_]*.json)
if [ $# -gt 0 ]; then
  paths=()
  for scene in "$@"; do
    case "$scene" in
      *.json) paths+=("$scene") ;;
      *) paths+=("video/scenes/$scene.json") ;;
    esac
  done
fi
if [ ${#paths[@]} -eq 0 ]; then
  echo "No scenes in video/scenes; pass a scene file, such as video/examples/hero.json." >&2
  exit 1
fi
video/.build/release/director record --app "${app}" --out build/recordings "${paths[@]}"
