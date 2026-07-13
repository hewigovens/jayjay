#!/usr/bin/env bash
# Manage the app version + build number across every source of truth (set | check; check used by release).
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
justfile="$root/shell/justfile"
cli_cargo="$root/crates/jayjay-cli/Cargo.toml"
gpui_cargo="$root/shell/gpui/Cargo.toml"
projyml="$root/shell/mac/project.yml"

# file | line prefix | version|build | label
fields=(
  "$justfile|^version := |version|justfile version"
  "$justfile|^build_number := |build|justfile build_number"
  "$cli_cargo|^version = |version|CLI Cargo.toml version"
  "$gpui_cargo|^version = |version|GPUI Cargo.toml version"
  "$projyml|MARKETING_VERSION: |version|project.yml marketing"
  "$projyml|CURRENT_PROJECT_VERSION: |build|project.yml build"
)

# Read/replace the version-or-build number on the matching line, leaving any quotes intact.
read_field() { grep -m1 "$2" "$1" | grep -oE '[0-9][0-9.]*' | head -1; }
write_field() { sed -i '' -E "s|($2[^0-9]*)[0-9][0-9.]*|\\1$3|" "$1"; }
want() { [ "$1" = build ] && printf %s "$build" || printf %s "$version"; }

cmd="${1:-}" version="${2:-}" build="${3:-}"

case "$cmd" in
  set)
    [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ && "$build" =~ ^[0-9]+$ ]] || { echo "usage: version.sh set <X.Y.Z> <build>" >&2; exit 2; }
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
