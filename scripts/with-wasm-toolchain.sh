#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

JAYJAY_WASM_HEADERS="$(cargo metadata --locked --format-version 1 --filter-platform wasm32-unknown-unknown | python3 -c '
import json
import pathlib
import sys

package = next(p for p in json.load(sys.stdin)["packages"] if p["name"] == "tree-sitter-language")
print(pathlib.Path(package["manifest_path"]).parent / "wasm/include")
')"
export JAYJAY_WASM_HEADERS
# Apple's ar silently discards WASM objects, so use the Rust toolchain's LLVM archiver.
llvm_ar="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin/llvm-ar"
if [[ ! -x "$llvm_ar" ]]; then
  echo "WASM linking needs LLVM tools; run rustup component add llvm-tools" >&2
  exit 1
fi
# Bump the define when the wrapper-injected sysroot changes; grammar build scripts cannot track those headers themselves.
AR_wasm32_unknown_unknown="$llvm_ar" \
CC_wasm32_unknown_unknown="$root/scripts/llvm-clang" \
CXX_wasm32_unknown_unknown="$root/scripts/llvm-clang" \
CFLAGS_wasm32_unknown_unknown="-DJAYJAY_WASM_SYSROOT_REV=5" \
CXXFLAGS_wasm32_unknown_unknown="-DJAYJAY_WASM_SYSROOT_REV=5" \
"$@"
