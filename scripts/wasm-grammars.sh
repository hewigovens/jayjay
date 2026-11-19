#!/usr/bin/env bash
# Rebuild the WebAssembly grammars jj-diff bundles from the grammar crates locked in Cargo.lock.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$root/crates/jj-diff/grammars/wasm"
registry="${CARGO_HOME:-$HOME/.cargo}/registry/src"

# name, crate, grammar directory inside the crate
grammars=(
  "kotlin tree-sitter-kotlin-sg ."
  "c_sharp tree-sitter-c-sharp ."
  "sql tree-sitter-sequel ."
  "ruby tree-sitter-ruby ."
  "php tree-sitter-php php"
)

locked_version() {
  local versions
  versions="$(awk -v name="$1" '$0 == "name = \"" name "\"" { getline; gsub(/version = |"/, ""); print }' "$root/Cargo.lock")"
  if [[ -z "$versions" || "$versions" == *$'\n'* ]]; then
    echo "expected exactly one locked version of $1, found: ${versions:-none}" >&2
    exit 1
  fi
  echo "$versions"
}

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

cli_version="$(locked_version tree-sitter)"
cargo fetch --locked --manifest-path "$root/Cargo.toml"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

: > "$work/versions"
for entry in "${grammars[@]}"; do
  read -r name crate dir <<< "$entry"
  version="$(locked_version "$crate")"
  src="$(ls -d "$registry"/*/"$crate-$version" | head -n 1)"
  cp -R "$src" "$work/$name"
  chmod -R u+w "$work/$name"
  (cd "$work/$name/$dir" && npx -y "tree-sitter-cli@$cli_version" build --wasm -o "$work/$name.wasm" .)
  gzip -9 -n -c "$work/$name.wasm" > "$out/$name.wasm.gz"
  cp "$src/queries/highlights.scm" "$out/$name.scm"
  if [[ -f "$src/LICENSE" ]]; then
    cp "$src/LICENSE" "$out/$name.LICENSE"
  else
    mit_license "$src" "$crate" "$version" > "$out/$name.LICENSE"
  fi
  echo "$crate $version" >> "$work/versions"
done
cp "$work/versions" "$out/versions"
ls -l "$out"
