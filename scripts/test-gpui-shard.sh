#!/usr/bin/env bash
set -euo pipefail

index="$1"
count="$2"

modules=$(cargo test --locked -p jayjay-gpui --test gpui -- --list \
  | awk -F': ' '$2 == "test" { sub(/::.*/, "::", $1); tests[$1]++ } END { for (module in tests) print tests[module] "\t" module }' \
  | LC_ALL=C sort -t "$(printf '\t')" -k1,1nr -k2,2 \
  | awk -F'\t' -v i="${index}" -v n="${count}" '{ m = 1; for (s = 2; s <= n; s++) if (load[s] < load[m]) m = s; load[m] += $1; if (m == i) print $2 }')
[[ -n "$modules" ]] || { echo "GPUI shard ${index}/${count} has no tests" >&2; exit 1; }
filters=()
while IFS= read -r module; do
  filters+=("$module")
done <<<"$modules"
cargo test --locked -p jayjay-gpui --test gpui -- "${filters[@]}"
