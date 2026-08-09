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
source "$script_dir/ui-test-external-tools.sh"

# Identity for the fixture commits; silences jj's "Name and email not configured" on CI.
export JJ_USER="JayJay CI" JJ_EMAIL="ci@jayjay.local"

setup_defaults() {
  defaults write "$bundle_id" jayjay.hasCompletedOnboarding -bool YES
  defaults write "$bundle_id" jayjay.sideBySideDiff -bool NO
  defaults write "$bundle_id" jayjay.ignoreWhitespace -bool NO
  defaults write "$bundle_id" jayjay.treeFileList -bool NO
  defaults write "$bundle_id" jayjay.recentRepos -array "$fixtures/formats"
  defaults delete "$bundle_id" jayjay.lastOpenedRepo 2>/dev/null || true
  # Start each run with the command palette at its default (centered) position.
  defaults delete "$bundle_id" commandPalette.frameOrigin 2>/dev/null || true
}

# Simple: three commits + an active working copy with two new files.
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
}

copy_fixture() {
  local source="$1"
  local destination="$2"
  cp -R "$fixtures/$source" "$fixtures/$destination"
}

fixture_mutating_scenes() {
  copy_fixture simple commit
  copy_fixture simple edit-description
  copy_fixture simple save-description
  copy_fixture simple diff-stats
  copy_fixture simple new-change
  copy_fixture simple file-editor
}

fixture_bookmark_diff() {
  copy_fixture simple bookmark-diff
  (
    cd "$fixtures/bookmark-diff"
    jj bookmark create bookmark-diff -r @
  )
}

# Simple plus structured files for projection/rendering checks.
fixture_formats() {
  copy_fixture simple formats
  (
    cd "$fixtures/formats"
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
  copy_fixture simple review-notes
  (
    cd "$fixtures/review-notes"
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

# One boundary edit leaves exactly one 53-line unchanged region for collapsed-context expansion.
fixture_context_expansion() {
  jj git init --colocate "$fixtures/context-expansion"
  (
    cd "$fixtures/context-expansion"
    for line in {1..57}; do
      printf 'baseline line %02d\n' "$line"
    done > context.txt
    jj describe -m "context baseline"
    jj bookmark create main -r @
    jj new
    for line in {1..57}; do
      if [[ "$line" == 1 ]]; then
        printf 'working copy line %02d\n' "$line"
      else
        printf 'baseline line %02d\n' "$line"
      fi
    done > context.txt
  )
}

# Complex: a broad working-copy diff with enough files and changed lines to exercise Diff Edit's large-repository policy.
fixture_complex() {
  jj git init --colocate "$fixtures/complex"
  (
    cd "$fixtures/complex"
    mkdir -p assets config/environments "docs/product specs" docs/guides src/modules

    for module_number in {1..12}; do
      printf -v module '%02d' "$module_number"
      {
        printf 'public enum Module%s {\n' "$module"
        for line in {1..36}; do
          printf '    public static let value%s = "module-%s-base-%s"\n' "$line" "$module" "$line"
        done
        printf '}\n'
      } > "src/modules/module-$module.swift"
    done

    for guide_number in {1..10}; do
      printf -v guide '%02d' "$guide_number"
      {
        printf '# Guide %s\n\n' "$guide"
        for line in {1..24}; do
          printf 'Baseline guide %s paragraph %s with stable documentation context.\n' "$guide" "$line"
        done
      } > "docs/guides/guide-$guide.md"
    done

    for environment in development staging production test; do
      printf '{\n  "environment": "%s",\n  "featureFlags": ["search", "sync", "review"]\n}\n' "$environment" > "config/environments/$environment.json"
    done

    printf '# API overview\n\nBaseline API contract.\n' > "docs/product specs/api overview.md"
    printf '\x00\x01jayjay-baseline-binary\x00' > assets/logo.bin

    jj describe -m "complex baseline"
    jj bookmark create main -r @
    jj new

    for module_number in {1..8}; do
      printf -v module '%02d' "$module_number"
      {
        printf 'public enum Module%s {\n' "$module"
        for line in {1..48}; do
          printf '    public static let revisedValue%s = "module-%s-working-copy-%s"\n' "$line" "$module" "$line"
        done
        printf '}\n'
      } > "src/modules/module-$module.swift"
    done

    printf '{\n  "environment": "development",\n  "featureFlags": ["search", "sync", "review", "debug-menu"],\n  "apiBaseURL": "https://dev.example.test"\n}\n' > config/environments/development.json
    printf '{\n  "environment": "staging",\n  "featureFlags": ["search", "sync", "review", "audit-log"],\n  "apiBaseURL": "https://staging.example.test"\n}\n' > config/environments/staging.json

    for guide_number in {1..4}; do
      printf -v guide '%02d' "$guide_number"
      rm "docs/guides/guide-$guide.md"
    done
    mkdir -p docs/reference
    mv docs/guides/guide-05.md docs/reference/guide-05-renamed.md
    mv docs/guides/guide-06.md docs/reference/guide-06-renamed.md

    for feature_number in {1..24}; do
      printf -v feature '%02d' "$feature_number"
      mkdir -p "src/features/area-$feature"
      {
        printf 'public struct Feature%sComponent {\n' "$feature"
        for line in {1..34}; do
          printf '    public let field%s = "feature-%s-value-%s"\n' "$line" "$feature" "$line"
        done
        printf '}\n'
      } > "src/features/area-$feature/component.swift"
    done

    mkdir -p src/generated/tables
    {
      printf 'public let generatedRows = [\n'
      for ((line = 1; line <= 420; line++)); do
        printf '    "generated-row-%03d",\n' "$line"
      done
      printf ']\n'
    } > src/generated/tables/large_table.swift

    printf '# API overview\n\nWorking-copy API contract with authentication, pagination, retries, and structured errors.\n' > "docs/product specs/api overview.md"
    printf '\x00\x02jayjay-working-copy-binary\x00' > assets/logo.bin
  )
}

# Conflict: @ is a rebased change with one Swift file containing multiple conflicts.
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
  copy_fixture conflict conflict-use-ours
}

fixture_repository_stores() {
  printf '{"repositories":[]}\n' > "$fixtures/repositories-empty.json"
  printf '{"repositories":["%s"]}\n' "$fixtures/formats" > "$fixtures/repositories-pinned.json"
}

# Three code sections edit the same expressions so every side of the rebase collides while syntax highlighting stays testable.
write_conflict_file() {
  local variant="$1"
  local retry_limit=3
  if [[ "$variant" == main ]]; then
    retry_limit=5
  elif [[ "$variant" == feature ]]; then
    retry_limit=7
  fi
  printf '%s\n' \
    "import Foundation" \
    "" \
    "struct ConflictSample {" \
    "    static let title = \"$variant build\"" \
    "    static let stableIdentifier = \"jayjay\"" \
    "" \
    "    static func greeting(for name: String) -> String {" \
    "        let prefix = \"$variant hello\"" \
    '        return "\(prefix), \(name)!"' \
    "    }" \
    "" \
    "    static func retryDelay(attempt: Int) -> Duration {" \
    "        let retryLimit = $retry_limit" \
    "        return .seconds(min(attempt, retryLimit))" \
    "    }" \
    "}" \
    > conflict.swift
}

setup_defaults
rm -rf "$fixtures"
mkdir -p "$fixtures"
fixture_simple
fixture_mutating_scenes
fixture_external_tools
fixture_bookmark_diff
fixture_formats
fixture_review_notes
fixture_context_expansion
fixture_complex
fixture_conflict
fixture_repository_stores
