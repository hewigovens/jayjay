#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Apple's ar silently discards WASM objects, so use the Rust toolchain's LLVM archiver.
llvm_ar="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin/llvm-ar"
if [[ ! -x "$llvm_ar" ]]; then
  echo "WASM linking needs LLVM tools; run rustup component add llvm-tools" >&2
  exit 1
fi
AR_wasm32_unknown_unknown="$llvm_ar" \
CC_wasm32_unknown_unknown="$root/scripts/llvm-clang" \
"$@"
