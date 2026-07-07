#!/usr/bin/env bash
# Deterministic jj fixture repos for the XCUITest scenes (see
# shell/mac/Tests/JayJayUITests/Support/SceneBase.swift for how scenes pick
# one), plus the user-defaults setup that keeps fresh machines from showing
# onboarding mid-test.
#
# Usage: ui-test-fixtures.sh <bundle-id>
set -euo pipefail

bundle_id="${1:?usage: ui-test-fixtures.sh <bundle-id>}"
fixtures=/tmp/jayjay-test-fixtures
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "$script_dir/.." && pwd)"
format_fixtures="$project_root/tests/fixtures/formats"

# Identity for the fixture commits; silences jj's "Name and email not configured" on CI.
export JJ_USER="JayJay CI" JJ_EMAIL="ci@jayjay.local"

setup_defaults() {
  defaults write "$bundle_id" jayjay.hasCompletedOnboarding -bool YES
  defaults write "$bundle_id" jayjay.sideBySideDiff -bool NO
  defaults write "$bundle_id" jayjay.ignoreWhitespace -bool NO
  defaults write "$bundle_id" jayjay.treeFileList -bool NO
  # Start each run with the command palette at its default (centered) position.
  defaults delete "$bundle_id" commandPalette.frameOrigin 2>/dev/null || true
}

# Simple: three commits + an active working copy with two new files.
# Clone once, then copy per-class for tests that mutate repo state so leaks
# don't bleed across scenes (NewChangeScene creates a new @).
fixture_simple() {
  jj git init --colocate "$fixtures/simple"
  (
    cd "$fixtures/simple"
    echo "# Sample project" > README.md
    jj describe -m "initial"
    jj new -m "add hello"
    echo "hello" > hello.txt
    jj new -m "add feature"
    echo "feature" > feature.txt
    jj bookmark create main -r @
    jj new
    echo "wip 1" > wip1.txt
    echo "wip 2" > wip2.txt
  )
  cp -R "$fixtures/simple" "$fixtures/simple-newchange"
  cp -R "$fixtures/simple" "$fixtures/simple-commit"
  cp -R "$fixtures/simple" "$fixtures/simple-editdesc"
  cp -R "$fixtures/simple" "$fixtures/simple-save-description"
  cp -R "$fixtures/simple" "$fixtures/simple-diffstats"
  cp -R "$fixtures/simple" "$fixtures/simple-bookmark-diff"
  cp -R "$fixtures/simple" "$fixtures/simple-review-notes"
  cp -R "$fixtures/simple" "$fixtures/simple-formats"
  (
    cd "$fixtures/simple-bookmark-diff"
    jj bookmark create bookmark-diff -r @
  )
}

# Simple plus structured files for projection/rendering checks.
fixture_formats() {
  (
    cd "$fixtures/simple-formats"
    cp "$format_fixtures/analysis.ipynb" analysis.ipynb
    cp "$format_fixtures/notes.md" notes.md
    cp "$format_fixtures/release.html" release.html
    cp "$format_fixtures/data.csv" data.csv
    cp "$format_fixtures/results.sarif" results.sarif
    cp "$format_fixtures/Info.plist" Info.plist
    plutil -convert binary1 Info.plist
    cp "$format_fixtures/PlainInfo.plist" PlainInfo.plist
  )
}

# A long function so ReviewNotesScene can verify the diff expands around embedded note rows.
fixture_review_notes() {
  (
    cd "$fixtures/simple-review-notes"
    printf '%s\n' \
      'func fibonacciReport(limit: Int) -> String {' \
      '    var values: [Int] = []' \
      '    var a = 0' \
      '    var b = 1' \
      '    while values.count < limit {' \
      '        values.append(a)' \
      '        let next = a + b' \
      '        a = b' \
      '        b = next' \
      '    }' \
      '    var lines: [String] = []' \
      '    for (index, value) in values.enumerated() {' \
      '        if value % 2 == 0 {' \
      '            lines.append("\(index): \(value) even")' \
      '        } else {' \
      '            lines.append("\(index): \(value) odd")' \
      '        }' \
      '    }' \
      '    var total = 0' \
      '    for value in values {' \
      '        total += value' \
      '    }' \
      '    lines.append("total: \(total)")' \
      '    return lines.joined(separator: "\n")' \
      '}' > scoring.swift
  )
}

# Conflict: @ is a rebased change with one file containing multiple conflicts.
fixture_conflict() {
  jj git init --colocate "$fixtures/conflict"
  (
    cd "$fixtures/conflict"
    write_conflict_file base
    jj describe -m "base"
    jj bookmark create main -r @
    jj new -m "main: conflicting edits"
    write_conflict_file main
    jj bookmark set main -r @
    jj new -r 'main-' -m "feature: conflicting edits"
    write_conflict_file feature
    jj rebase -r @ -d main
  )
  cp -R "$fixtures/conflict" "$fixtures/conflict-use-ours"
}

# Three sections editing the same keys so every side of the rebase collides.
write_conflict_file() {
  local variant="$1"
  printf '%s\n' \
    "project = jayjay" \
    "" \
    "[alpha]" \
    "value = $variant-alpha" \
    "keep = alpha context" \
    "" \
    "[beta]" \
    "value = $variant-beta" \
    "keep = beta context" \
    "" \
    "[gamma]" \
    "value = $variant-gamma" \
    "keep = gamma context" \
    > file.txt
}

setup_defaults
rm -rf "$fixtures"
mkdir -p "$fixtures"
fixture_simple
fixture_formats
fixture_review_notes
fixture_conflict
