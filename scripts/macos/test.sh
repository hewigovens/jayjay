#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

action="$1"
shift
scheme=JayJayUITests
[[ "$action" != test ]] || scheme=JayJay

case "$action" in
  test|ui-test)
    xcodegen --spec "$project/project.yml" --project "$project"
    operation="test"
    ;;
  ui-test-build)
    operation=build-for-testing
    ;;
  ui-test-prebuilt)
    xctestruns=("$derived_data"/Build/Products/JayJayUITests_*.xctestrun)
    xctestrun="${xctestruns[0]}"
    [[ -f "$xctestrun" ]] || { echo "No UI test build found; run 'just shell::ui-test-build' first." >&2; exit 1; }
    operation=test-without-building
    ;;
  *) echo "usage: $0 {test|ui-test|ui-test-build|ui-test-prebuilt} [test-ids...]" >&2; exit 2 ;;
esac
args=()
for test_id in "$@"; do
  args+=(-only-testing:"$test_id")
done
args+=("$operation")
if [[ "$action" == ui-test-prebuilt ]]; then
  xcodebuild -xctestrun "$xctestrun" -destination "platform=macOS" -derivedDataPath "$derived_data" "${args[@]}" | xcbeautify
else
  xcode "$scheme" Debug "${args[@]}"
fi
if [[ "$action" == test ]]; then
  cd "$root/shell/mac/Packages/JayJayDiffUI"
  swift test
fi
