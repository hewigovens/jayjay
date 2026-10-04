#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
test_dir="$(mktemp -d)"
trap 'rm -rf "$test_dir"' EXIT
legacy_sources="$test_dir/wasm/unsupported/tree-sitter-0.26"
mkdir -p "$legacy_sources"
for source in stdio stdlib string; do
  printf '#error legacy libc must not compile\n' > "$legacy_sources/$source.c"
done

compile() {
  CARGO_PKG_NAME="$1" DEP_TREE_SITTER_LANGUAGE_WASM_SRC="$legacy_sources" \
    "$repo_root/scripts/llvm-clang" --target=wasm32-unknown-unknown -c "$2" -o "$test_dir/output.o"
}

for grammar in tree-sitter-c tree-sitter-c-sharp tree-sitter-css tree-sitter-md; do
  for source in stdio stdlib string; do
    compile "$grammar" "$legacy_sources/$source.c"
    test -s "$test_dir/output.o"
  done
done

for package in tree-sitter tree-sitter-fish; do
  if compile "$package" "$legacy_sources/stdio.c" > "$test_dir/error.log" 2>&1; then
    echo "Unexpectedly bypassed the libc guard for $package" >&2
    exit 1
  fi
  grep -q 'legacy libc must not compile' "$test_dir/error.log"
done

printf '#error grammar errors must propagate\n' > "$test_dir/parser.c"
if compile tree-sitter-c "$test_dir/parser.c" > "$test_dir/error.log" 2>&1; then
  echo "Unexpectedly suppressed a grammar compilation error" >&2
  exit 1
fi
grep -q 'grammar errors must propagate' "$test_dir/error.log"
echo "WASM compiler compatibility checks passed"
