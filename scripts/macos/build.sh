#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

configuration="$1"
case "$configuration" in
  Debug) profile=debug ;;
  Release) profile=release ;;
  *) echo "usage: $0 {Debug|Release}" >&2; exit 2 ;;
esac
app_path="$derived_data/Build/Products/$configuration/$app_name.app"
rm -f "$app_path/Contents/MacOS/jayjay-cli"
if [[ "$configuration" == Debug ]]; then
  cargo build -p jayjay-cli
fi
xcodegen --spec "$project/project.yml" --project "$project"
if [[ "$configuration" == Release ]]; then
  xcode JayJay Release ARCHS=arm64 ONLY_ACTIVE_ARCH=NO build
  cargo build --release -p jayjay-cli
  export JAYJAY_STRIP_APP=1
else
  xcode JayJay Debug build
fi
bash "$root/scripts/prepare-app-bundle.sh" "$app_path" "$target_dir/$profile/jayjay"
if [[ "$configuration" == Debug ]]; then
  codesign --force --sign - "$app_path/Contents/MacOS/jayjay-cli"
  codesign --force --sign - "$app_path"
  echo "Built $app_path"
fi
