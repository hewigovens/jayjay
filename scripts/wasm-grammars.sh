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

# Crates without a LICENSE file get the MIT text cargo-about generated for them while they were linked natively.
mit_license() {
  grep -q '^license = "MIT"$' "$1/Cargo.toml" || { echo "$1 is not MIT licensed" >&2; exit 1; }
  printf '%s %s (%s)\n\n' "$2" "$3" "$(sed -n 's/^repository = "\(.*\)"$/\1/p' "$1/Cargo.toml")"
  cat <<'EOF'
Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
EOF
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
    mit_license "$src" "$crate" "$version" > "$out/$name.LICENSE"
  fi
  echo "$crate $version" >> "$work/versions"
done
cp "$work/versions" "$out/versions"
ls -l "$out"
