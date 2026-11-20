#!/usr/bin/env bash
# Rebuild the WebAssembly grammars jj-diff bundles from the grammar crates locked in Cargo.lock or the git commits pinned below.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$root/crates/jj-diff/grammars/wasm"
source "$root/scripts/grammar-wasm.sh"

# name, crate in Cargo.lock or git repository@commit, grammar directory, highlight query
grammars=(
  "kotlin tree-sitter-kotlin-sg . queries/highlights.scm"
  "c_sharp tree-sitter-c-sharp . queries/highlights.scm"
  "sql tree-sitter-sequel . queries/highlights.scm"
  "ruby tree-sitter-ruby . queries/highlights.scm"
  "php tree-sitter-php php queries/highlights.scm"
  "containerfile tree-sitter-containerfile . queries/highlights.scm"
  "dart tree-sitter-dart . queries/highlights.scm"
  "proto tree-sitter-proto . queries/highlights.scm"
  "r tree-sitter-r . queries/highlights.scm"
  # The tree-sitter-just crate depends on tree-sitter 0.25 and omits the query files its bindings include, so it cannot be locked here.
  "just https://github.com/casey/tree-sitter-just@5685543a6e64f66335e25518c9ae8ffa1dae3d01 . queries-flavored/helix/highlights.scm"
)

# A crate published without a LICENSE file gets the one at its repository's release tag, which carries the copyright notice MIT asks to keep.
repository_license() {
  local repository
  repository="$(sed -n 's/^repository = "\(.*\)"$/\1/p' "$1/Cargo.toml")"
  repository="${repository%.git}"
  curl -fsSL "https://raw.githubusercontent.com/${repository#https://github.com/}/v$2/LICENSE"
}

cargo fetch --locked --manifest-path "$root/Cargo.toml"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

: > "$work/versions"
for entry in "${grammars[@]}"; do
  read -r name crate dir query <<< "$entry"
  src="$work/$name"
  version="$(source_version "$crate")"
  fetch_source "$crate" "$src"
  crate="${crate%@*}"
  build_wasm "$src/$dir" "$work/$name.wasm" "$name"
  gzip -9 -n -c "$work/$name.wasm" > "$out/$name.wasm.gz"
  cp "$src/$query" "$out/$name.scm"
  if compgen -G "$src/LICENSE*" > /dev/null; then
    cat "$src"/LICENSE* > "$out/$name.LICENSE"
  else
    repository_license "$src" "$version" > "$out/$name.LICENSE"
  fi
  echo "$crate $version" >> "$work/versions"
done
cp "$work/versions" "$out/versions"
ls -l "$out"
