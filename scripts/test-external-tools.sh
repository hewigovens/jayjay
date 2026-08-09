#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
mode="merge"
launcher="${JAYJAY_EXTERNAL_TOOL_LAUNCHER:-}"
keep=0

usage() {
  echo "usage: scripts/test-external-tools.sh [diff|edit|merge|all] [--launcher <jayjay-or-app>] [--keep]"
  echo
  echo "Creates temporary jj repositories and runs jj diff, split, or resolve with JayJay."
  echo "It uses an already-built launcher and never invokes Xcode, cargo run, or a test host."
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    diff|edit|merge|all)
      mode="$1"
      shift
      ;;
    --launcher)
      launcher="${2:?missing launcher path}"
      shift 2
      ;;
    --keep)
      keep=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "$launcher" ]]; then
  if [[ -x "$root/build/DerivedData/Build/Products/Debug/JayJay.app/Contents/MacOS/jayjay-cli" ]]; then
    launcher="$root/build/DerivedData/Build/Products/Debug/JayJay.app/Contents/MacOS/jayjay-cli"
  elif command -v jayjay >/dev/null 2>&1; then
    launcher="$(command -v jayjay)"
  elif [[ -x "$root/target/debug/jayjay" ]]; then
    launcher="$root/target/debug/jayjay"
  else
    echo "No JayJay CLI found. Build/install it first, or pass --launcher <path>." >&2
    exit 2
  fi
fi

if [[ -d "$launcher" && "$launcher" == *.app ]]; then
  if [[ -x "$launcher/Contents/MacOS/jayjay-cli" ]]; then
    launcher="$launcher/Contents/MacOS/jayjay-cli"
  else
    echo "App bundle has no jayjay-cli executable: $launcher" >&2
    exit 2
  fi
fi
if [[ ! -x "$launcher" ]]; then
  echo "Launcher is not executable: $launcher" >&2
  exit 2
fi

fixture="$(mktemp -d "${TMPDIR:-/tmp}/jayjay-external-tools.XXXXXX")"
cleanup() {
  if [[ "$keep" == "1" ]]; then
    echo "Fixtures kept at $fixture"
  else
    rm -rf "$fixture"
  fi
}
trap cleanup EXIT

tool_config="$fixture/jayjay-tool.toml"
tool_bin="$fixture/bin"
mkdir -p "$tool_bin"
cat > "$tool_bin/jayjay" <<'SH'
#!/usr/bin/env bash
exec "${JAYJAY_TEST_LAUNCHER:?}" "$@"
SH
chmod +x "$tool_bin/jayjay"
export JAYJAY_TEST_LAUNCHER="$launcher"
export PATH="$tool_bin:$PATH"
tool_definition="$("$launcher" config)"
if [[ "$tool_definition" != *"[merge-tools.jayjay]"* ]]; then
  echo "Launcher does not support 'jayjay config': $launcher" >&2
  echo "Build the current CLI first, or pass a current JayJay.app with --launcher." >&2
  exit 2
fi
printf '%s\n' "$tool_definition" > "$tool_config"

write_base() {
  cat > "$1" <<'SWIFT'
import Foundation

struct ConflictSample {
    static let title = "base build"
    static let stableIdentifier = "jayjay"
    static let retries = 2

    static func greeting(for name: String) -> String {
        let prefix = "hello"
        return "\(prefix), \(name)!"
    }
}
SWIFT
}

write_left() {
  cat > "$1" <<'SWIFT'
import Foundation

struct ConflictSample {
    static let title = "main build"
    static let stableIdentifier = "jayjay"
    static let retries = 2

    static func greeting(for name: String) -> String {
        let prefix = "main hello"
        return "\(prefix), \(name)!"
    }
}
SWIFT
}

write_right() {
  cat > "$1" <<'SWIFT'
import Foundation

struct ConflictSample {
    static let title = "feature build"
    static let stableIdentifier = "jayjay"
    static let retries = 4

    static func greeting(for name: String) -> String {
        let prefix = "feature hello"
        return "\(prefix), \(name)!"
    }
}
SWIFT
}

prepare_working_copy_repo() {
  local repo="$fixture/working-copy"
  mkdir -p "$repo/Sources"
  jj git init "$repo" >/dev/null
  write_base "$repo/Sources/ConflictSample.swift"
  jj -R "$repo" describe -m "base Swift implementation" >/dev/null
  jj -R "$repo" bookmark create fixture-base -r @ >/dev/null
  jj -R "$repo" new fixture-base -m "working-copy Swift changes" >/dev/null
  write_right "$repo/Sources/ConflictSample.swift"
}

prepare_conflict_repo() {
  local repo="$fixture/conflict"
  mkdir -p "$repo/Sources"
  jj git init "$repo" >/dev/null
  write_base "$repo/Sources/ConflictSample.swift"
  jj -R "$repo" describe -m "base Swift implementation" >/dev/null
  jj -R "$repo" bookmark create fixture-base -r @ >/dev/null

  jj -R "$repo" new fixture-base -m "left Swift implementation" >/dev/null
  write_left "$repo/Sources/ConflictSample.swift"
  jj -R "$repo" bookmark create fixture-left -r @ >/dev/null

  jj -R "$repo" new fixture-base -m "right Swift implementation" >/dev/null
  write_right "$repo/Sources/ConflictSample.swift"
  jj -R "$repo" bookmark create fixture-right -r @ >/dev/null
  jj -R "$repo" new fixture-left fixture-right -m "conflicted Swift merge" >/dev/null
}

run_jj() {
  local repo="$1"
  shift
  echo "Launching: jj -R $repo $*"
  local status=0
  jj -R "$repo" --config-file "$tool_config" "$@" || status=$?
  echo "jj exited with status $status"
  return "$status"
}

run_diff() {
  echo "Close JayJay after inspecting the read-only diff."
  run_jj "$fixture/working-copy" diff --tool jayjay
}

run_edit() {
  echo "Choose a subset of lines, then click Done to exercise jj's blocking diff editor."
  run_jj "$fixture/working-copy" split --tool jayjay -m "selected with JayJay"
  jj -R "$fixture/working-copy" log -n 3 --no-graph -T 'description.first_line() ++ "\n"'
}

run_merge() {
  echo "Resolve one or more hunks, optionally inspect Raw mode, then save."
  run_jj "$fixture/conflict" resolve --tool jayjay 'root:"Sources/ConflictSample.swift"'
  jj -R "$fixture/conflict" status
  sed -n '1,120p' "$fixture/conflict/Sources/ConflictSample.swift"
}

case "$mode" in
  diff)
    prepare_working_copy_repo
    run_diff
    ;;
  edit)
    prepare_working_copy_repo
    run_edit
    ;;
  merge)
    prepare_conflict_repo
    run_merge
    ;;
  all)
    prepare_working_copy_repo
    prepare_conflict_repo
    run_diff
    run_edit
    run_merge
    ;;
esac
