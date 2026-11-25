#!/usr/bin/env bash
# Generates the web engine's run-time grammar assets in web/review/src/wasm from jj-diff's language table; files jj-diff bundles are reused and the rest are built from their pinned sources once per version.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$root/web/review/src/wasm"
bundled="$root/crates/jj-diff/grammars/wasm"
cache="${JAYJAY_GRAMMAR_CACHE:-$root/target/web-grammars}"
source "$root/scripts/grammar-wasm.sh"
# The build steps are part of every cache key, so editing them rebuilds each grammar.
steps="$(cat "$root/scripts/grammar-wasm.sh" "${BASH_SOURCE[0]}" | shasum -a 256 | cut -c1-16)"

# grammar file, crate in Cargo.lock, grammar directory
sources=(
  "bash tree-sitter-bash ."
  "batch tree-sitter-batch ."
  "c tree-sitter-c ."
  "cmake tree-sitter-cmake ."
  "cpp tree-sitter-cpp ."
  "css tree-sitter-css ."
  "fish tree-sitter-fish ."
  "fluent tree-sitter-fluent ."
  "go tree-sitter-go ."
  "html tree-sitter-html ."
  "ini tree-sitter-ini ."
  "java tree-sitter-java ."
  "javascript tree-sitter-javascript ."
  "json tree-sitter-json ."
  "lua tree-sitter-lua ."
  "make tree-sitter-make ."
  "markdown tree-sitter-md tree-sitter-markdown"
  "markdown_inline tree-sitter-md tree-sitter-markdown-inline"
  "nix tree-sitter-nix ."
  "powershell tree-sitter-powershell ."
  "python tree-sitter-python ."
  "rust tree-sitter-rust ."
  "solidity tree-sitter-solidity ."
  "swift tree-sitter-swift ."
  "toml tree-sitter-toml-ng ."
  "tsx tree-sitter-typescript tsx"
  "typescript tree-sitter-typescript typescript"
  "xml tree-sitter-xml xml"
  "yaml tree-sitter-yaml ."
  "zig tree-sitter-zig ."
)

mkdir -p "$out/grammars" "$cache"
cargo fetch --locked --manifest-path "$root/Cargo.toml"
imports="$(ls -d "$registry"/*/"tree-sitter-$(locked_version tree-sitter)" | head -n 1)/src/wasm-stdlib/imports.txt"
files="$(cargo run --locked --quiet -p jj-diff-web --bin export-web-grammars -- "$out" "$imports")"
rm -f "$out"/grammars/*.wasm
notices="$out/grammar-licenses.txt"
: > "$notices"
notice() {
  printf 'tree-sitter-%s\n\n' "$1"
  cat "$2"
  printf '\n\n'
}
current=()
for file in $files; do
  if [[ -f "$bundled/$file.wasm.gz" ]]; then
    notice "$file" "$bundled/$file.LICENSE" >> "$notices"
    gunzip -c "$bundled/$file.wasm.gz" > "$out/grammars/$file.wasm"
    continue
  fi
  entry="$(printf '%s\n' "${sources[@]}" | awk -v file="$file" '$1 == file')"
  if [[ -z "$entry" ]]; then
    echo "no source for the $file grammar; add it to scripts/web-grammars.sh" >&2
    exit 1
  fi
  read -r _ crate dir <<< "$entry"
  license="$(ls -d "$registry"/*/"$crate-$(locked_version "$crate")" | head -n 1)/LICENSE"
  # Several crates publish no license file, so their upstream one is kept in the repository.
  [[ -f "$license" ]] || license="$root/crates/jj-diff/grammars/licenses/$crate.LICENSE"
  notice "$file" "$license" >> "$notices"
  key="$(printf '%s %s %s %s %s' "$steps" "$crate" "$(source_version "$crate")" "$dir" "$(locked_version tree-sitter)" | shasum -a 256 | cut -c1-16)"
  cached="$cache/$file-$key.wasm"
  if [[ ! -f "$cached" ]]; then
    work="$(mktemp -d)"
    fetch_source "$crate" "$work/src"
    if [[ "$file" == swift ]]; then
      # The scanner reports allocation failure through stdio, which Wasm parsers cannot import.
      perl -0pi -e 's/fprintf\(stderr,[^;]*;\s*exit\(1\);/abort();/' "$work/src/src/scanner.c"
    fi
    build_wasm "$work/src/$dir" "$work/$file.wasm" "$file"
    mv "$work/$file.wasm" "$cached"
    rm -rf "$work"
  fi
  cp "$cached" "$out/grammars/$file.wasm"
  current+=("$cached")
done
for entry in "$cache"/*.wasm; do
  [[ " ${current[*]} " == *" $entry "* ]] || rm -f "$entry"
done
