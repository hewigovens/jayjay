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

# Six conflicting spots separated by long shared sections, so the merge hunk list overflows the window and scrolling to later hunks can be tested manually.
write_sample() {
  local file="$1" variant="$2" prefix="$3" farewell="$4" status="$5" timeout="$6" banner="$7" retries="$8"
  cat > "$file" <<SWIFT
import Foundation

struct ConflictSample {
    static let title = "${variant} build"
    static let stableIdentifier = "jayjay"
    static let retries = ${retries}

    static func greeting(for name: String) -> String {
        let prefix = "${prefix}"
        return "\(prefix), \(name)!"
    }

    static func normalizedIdentifier(_ raw: String) -> String {
        raw.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
    }

    static func chunked(_ values: [Int], size: Int) -> [[Int]] {
        stride(from: 0, to: values.count, by: size).map {
            Array(values[\$0 ..< min(\$0 + size, values.count)])
        }
    }

    static func farewell(for name: String) -> String {
        let suffix = "${farewell}"
        return "\(suffix), \(name)."
    }

    static func histogram(of values: [Int]) -> [Int: Int] {
        values.reduce(into: [:]) { counts, value in
            counts[value, default: 0] += 1
        }
    }

    static func mergeSorted(_ lhs: [Int], _ rhs: [Int]) -> [Int] {
        var result: [Int] = []
        var left = lhs.makeIterator()
        var right = rhs.makeIterator()
        var l = left.next()
        var r = right.next()
        while let lv = l, let rv = r {
            if lv <= rv { result.append(lv); l = left.next() } else { result.append(rv); r = right.next() }
        }
        while let lv = l { result.append(lv); l = left.next() }
        while let rv = r { result.append(rv); r = right.next() }
        return result
    }

    static func statusMessage() -> String {
        "sync state: ${status}"
    }

    static func fibonacci(_ n: Int) -> Int {
        var (a, b) = (0, 1)
        for _ in 0 ..< max(0, n) {
            (a, b) = (b, a + b)
        }
        return a
    }

    static func runLengthEncode(_ text: String) -> [(Character, Int)] {
        var encoded: [(Character, Int)] = []
        for character in text {
            if let last = encoded.last, last.0 == character {
                encoded[encoded.count - 1].1 += 1
            } else {
                encoded.append((character, 1))
            }
        }
        return encoded
    }

    static let requestTimeout: TimeInterval = ${timeout}

    static func clamp(_ value: Int, lower: Int, upper: Int) -> Int {
        min(max(value, lower), upper)
    }

    static func columns(_ rows: [[String]]) -> [[String]] {
        guard let width = rows.first?.count else { return [] }
        return (0 ..< width).map { column in rows.map { \$0[column] } }
    }

    static func banner() -> String {
        "=== ${banner} ==="
    }
}
SWIFT
}

write_base() {
  write_sample "$1" "base" "hello" "goodbye" "idle" 30 "jayjay sample" 2
}

write_left() {
  write_sample "$1" "main" "main hello" "main goodbye" "main branch ready" 45 "jayjay main sample" 2
}

write_right() {
  write_sample "$1" "feature" "feature hello" "feature farewell" "feature branch syncing" 60 "jayjay feature sample" 4
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
