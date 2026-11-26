#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 <linux target> <output> | $0 aarch64-apple-darwin <output> <Sparkle LICENSE>" >&2
  exit 2
}
[[ $# -ge 2 ]] || usage
target=$1
output=$2
root=$(cd "$(dirname "$0")/.." && pwd)
case "$target" in
  aarch64-apple-darwin)
    [[ $# -eq 3 ]] || usage
    manifest="$root/crates/jayjay-uniffi/Cargo.toml"
    ;;
  x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu)
    [[ $# -eq 2 ]] || usage
    manifest="$root/shell/gpui/Cargo.toml"
    ;;
  *) usage ;;
esac
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

if [[ $(cargo about --version 2>/dev/null) != 'cargo-about 0.9.2' ]]; then
  echo 'Install cargo-about 0.9.2 with: cargo install cargo-about --version 0.9.2 --features cli --locked' >&2
  exit 1
fi
cargo fetch --locked --manifest-path "$root/Cargo.toml"
cargo about generate --offline --locked --fail --manifest-path "$manifest" \
  --config "$root/dist/about.toml" --target "$target" \
  --output-file "$work/Licenses.html" "$root/dist/licenses.hbs"

append_notice() {
  printf '<h2>%s</h2>\n<pre>' "$1"
  sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g' "$2"
  printf '</pre>\n'
}
{
  printf '<h1>Bundled assets</h1>\n'
  append_notice Seti "$root/assets/fonts/Seti.LICENSE"
  append_notice Octicons "$root/shell/gpui/assets/fonts/Octicons.LICENSE"
  append_notice ToolbarIcons "$root/shell/mac/Resources/ToolbarIcons.LICENSE"
  append_notice DashboardIcons "$root/shell/mac/Resources/DashboardIcons.LICENSE"
  if [[ "$target" == aarch64-apple-darwin ]]; then
    mermaid="$root/shell/mac/Packages/JayJayDiffUI/Sources/JayJayDiffUI/Resources/Mermaid"
    append_notice Sparkle "$3"
    append_notice Ondine "$mermaid/ondine-LICENSE.txt"
    append_notice Pele "$mermaid/pele-LICENSE.txt"
  fi
  for license in "$root"/crates/jj-diff/grammars/wasm/*.LICENSE; do
    append_notice "tree-sitter-$(basename "$license" .LICENSE)" "$license"
  done
} >> "$work/Licenses.html"

mkdir -p "$(dirname "$output")"
mv "$work/Licenses.html" "$output"
