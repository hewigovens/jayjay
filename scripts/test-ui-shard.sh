#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
index="$1"
count="$2"
recipe="${3:-shell::ui-test}"

scenes=$(cd "${root}/shell/mac/Tests/JayJayUITests/Scenes" && grep -c '^ *func test' -- *.swift | LC_ALL=C sort -t: -k2,2nr -k1,1 \
  | awk -F: -v i="${index}" -v n="${count}" '{ m = 1; for (s = 2; s <= n; s++) if (load[s] < load[m]) m = s; load[m] += $2; if (m == i) { sub(/\.swift$/, "", $1); print "JayJayUITests/" $1 } }')
[[ -n "$scenes" ]] || { echo "UI shard ${index}/${count} has no scenes" >&2; exit 1; }
scene_args=()
while IFS= read -r scene; do
  scene_args+=("$scene")
done <<<"$scenes"
just "$recipe" "${scene_args[@]}"
