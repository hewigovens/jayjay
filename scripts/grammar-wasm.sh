# Sourced by wasm-grammars.sh and web-grammars.sh: fetches pinned grammar sources and compiles them to WebAssembly.
registry="${CARGO_HOME:-$HOME/.cargo}/registry/src"

locked_version() {
  local versions
  versions="$(awk -v name="$1" '$0 == "name = \"" name "\"" { getline; gsub(/version = |"/, ""); print }' "$root/Cargo.lock")"
  if [[ -z "$versions" || "$versions" == *$'\n'* ]]; then
    echo "expected exactly one locked version of $1, found: ${versions:-none}" >&2
    exit 1
  fi
  echo "$versions"
}

# The version a source resolves to: a crate's locked version or a git source's commit.
source_version() {
  case "$1" in
    https://*) echo "${1#*@}" ;;
    *) locked_version "$1" ;;
  esac
}

# Copies a crate locked in Cargo.lock or a git repository@commit to a writable directory.
fetch_source() {
  case "$1" in
    https://*)
      git clone --quiet "${1%@*}" "$2"
      git -C "$2" checkout --quiet "${1#*@}"
      ;;
    *) cp -R "$(ls -d "$registry"/*/"$1-$(locked_version "$1")" | head -n 1)" "$2" ;;
  esac
  chmod -R u+w "$2"
}

# tree-sitter's CLI downloads and caches the WASI SDK it compiles with, and reads the grammar's name from src/grammar.json, which a vendored grammar may lack.
build_wasm() {
  if [[ ! -f "$1/src/grammar.json" && ! -f "$1/grammar.js" ]]; then
    printf '{"name":"%s","rules":{}}\n' "$3" > "$1/src/grammar.json"
  fi
  (cd "$1" && npx -y "tree-sitter-cli@$(locked_version tree-sitter)" build --wasm -o "$2" .)
}
