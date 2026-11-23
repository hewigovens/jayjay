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
  licenses="$app_path/Contents/Resources/Licenses.html"
  cargo fetch --locked
  cargo about generate --offline --locked --fail --manifest-path "$root/crates/jayjay-uniffi/Cargo.toml" \
    --config "$root/dist/macos/about.toml" --output-file "$licenses" "$root/dist/licenses.hbs"
  mermaid="$project/Packages/JayJayDiffUI/Sources/JayJayDiffUI/Resources/Mermaid"
  bundled=(
    "Sparkle $derived_data/SourcePackages/checkouts/Sparkle/LICENSE"
    "Seti $root/assets/fonts/Seti.LICENSE"
    "Octicons $root/shell/gpui/assets/fonts/Octicons.LICENSE"
    "ToolbarIcons $project/Resources/ToolbarIcons.LICENSE"
    "DashboardIcons $project/Resources/DashboardIcons.LICENSE"
    "Ondine $mermaid/ondine-LICENSE.txt"
    "Pele $mermaid/pele-LICENSE.txt"
    "tree-sitter-fluent $root/crates/jj-diff/grammars/fluent/LICENSE"
  )
  for license in "$root"/crates/jj-diff/grammars/wasm/*.LICENSE; do
    bundled+=("tree-sitter-$(basename "$license" .LICENSE) $license")
  done
  for entry in "${bundled[@]}"; do
    printf '<h2>%s</h2>\n<pre>' "${entry%% *}"
    sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g' "${entry#* }"
    printf '</pre>\n'
  done >> "$licenses"
else
  xcode JayJay Debug build
fi
bash "$root/scripts/prepare-app-bundle.sh" "$app_path" "$target_dir/$profile/jayjay"
if [[ "$configuration" == Debug ]]; then
  bash "$root/scripts/macos/sign-debug.sh" "$app_path/Contents/MacOS/jayjay-cli"
  bash "$root/scripts/macos/sign-debug.sh" "$app_path"
  echo "Built $app_path"
fi
