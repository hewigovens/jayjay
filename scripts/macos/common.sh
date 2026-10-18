#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
project="$root/shell/mac"
target_dir="$root/target"
derived_data="$root/build/DerivedData"
app_name=JayJay
app="$derived_data/Build/Products/Debug/$app_name.app"
release_dir="$root/build/release"
xcframework="$project/JayJayFFI.xcframework"
swift_out="$project/Sources/JayJayCore"

xcode() {
  local scheme="$1" configuration="$2"
  shift 2
  xcodebuild \
    -project "$project/JayJay.xcodeproj" \
    -scheme "$scheme" \
    -configuration "$configuration" \
    -sdk macosx \
    -derivedDataPath "$derived_data" \
    "$@" | xcbeautify
}
