#!/usr/bin/env bash
# Manage the app version + build number across every source of truth (set | check; check used by release).
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
justfile="$root/shell/justfile"
workspace_cargo="$root/Cargo.toml"
projyml="$root/shell/mac/project.yml"

# base fields carry X.Y.Z without any -beta.N suffix: installed builds and their pings never see it.
# file | line prefix | version|base|build | label
fields=(
  "$justfile|^version := |version|justfile version"
  "$justfile|^build_number := |build|justfile build_number"
  "$workspace_cargo|^version = |base|Cargo workspace version"
  "$projyml|MARKETING_VERSION: |base|project.yml marketing"
  "$projyml|CURRENT_PROJECT_VERSION: |build|project.yml build"
)

# Read/replace the version-or-build number on the matching line, leaving any quotes intact.
read_field() { grep -m1 "$2" "$1" | grep -oE '[0-9][0-9.]*(-beta\.[0-9]+)?' | head -1; }
write_field() { sed -i '' -E "s|($2[^0-9]*)[0-9][0-9.]*(-beta\.[0-9]+)?|\\1$3|" "$1"; }
want() {
  case "$1" in
    build) printf %s "$build" ;;
    base) printf %s "${version%%-beta.*}" ;;
    *) printf %s "$version" ;;
  esac
}

cmd="${1:-}" version="${2:-}" build="${3:-}"

case "$cmd" in
  set)
    [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$ && "$build" =~ ^[0-9]+$ ]] || { echo "usage: version.sh set <X.Y.Z[-beta.N]> <build>" >&2; exit 2; }
    for f in "${fields[@]}"; do
      IFS='|' read -r file prefix which _ <<<"$f"
      write_field "$file" "$prefix" "$(want "$which")"
    done
    echo "Set $version (build $build). Cargo.lock and the Xcode project regenerate on the next build."
    ;;
  check)
    [ -n "$version" ] || { version="$(read_field "$justfile" '^version := ')" build="$(read_field "$justfile" '^build_number := ')"; }
    ok=1
    for f in "${fields[@]}"; do
      IFS='|' read -r file prefix which label <<<"$f"
      actual="$(read_field "$file" "$prefix")"
      if [ "$actual" = "$(want "$which")" ]; then
        printf '  ✓ %s: %s\n' "$label" "$actual"
      else
        printf '  ✗ %s: expected %s, found %s\n' "$label" "$(want "$which")" "$actual" >&2
        ok=0
      fi
    done
    [ "$ok" -eq 1 ] || { echo "Version sources disagree — run 'just set-version $version $build'." >&2; exit 1; }
    echo "All version sources agree: $version (build $build)."
    ;;
  *)
    echo "usage: version.sh {set|check} <version> <build>" >&2
    exit 2
    ;;
esac
