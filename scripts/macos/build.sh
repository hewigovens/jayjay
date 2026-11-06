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
if [[ "$configuration" == Release ]]; then
  xcode JayJay Release ARCHS=arm64 ONLY_ACTIVE_ARCH=NO build
  cargo build --release -p jayjay-cli
  export JAYJAY_STRIP_APP=1
  licenses="$app_path/Contents/Resources/Licenses"
  rm -rf "$licenses"
  mkdir -p "$licenses"
  cargo fetch --locked
  for crate in jayjay-uniffi jayjay-cli; do
    cargo about generate --offline --locked --fail --manifest-path "$root/crates/$crate/Cargo.toml" \
      --config "$root/dist/macos/about.toml" --output-file "$licenses/THIRD-PARTY-$crate.html" "$root/dist/licenses.hbs"
  done
  mermaid="$project/Packages/JayJayDiffUI/Sources/JayJayDiffUI/Resources/Mermaid"
  cp "$derived_data/SourcePackages/checkouts/Sparkle/LICENSE" "$licenses/Sparkle.LICENSE"
  cp "$root/assets/fonts/Seti.LICENSE" "$licenses/Seti.LICENSE"
  cp "$mermaid/ondine-LICENSE.txt" "$licenses/Ondine.LICENSE"
  cp "$mermaid/pele-LICENSE.txt" "$licenses/Pele.LICENSE"
  cp "$root/crates/jj-diff/grammars/fluent/LICENSE" "$licenses/tree-sitter-fluent.LICENSE"
else
  xcode JayJay Debug build
fi
bash "$root/scripts/prepare-app-bundle.sh" "$app_path" "$target_dir/$profile/jayjay"
if [[ "$configuration" == Debug ]]; then
  bash "$root/scripts/macos/sign-debug.sh" "$app_path/Contents/MacOS/jayjay-cli"
  bash "$root/scripts/macos/sign-debug.sh" "$app_path"
  echo "Built $app_path"
fi
