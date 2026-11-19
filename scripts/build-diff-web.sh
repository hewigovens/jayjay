#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
npm run themes
cargo run --locked -p jj-diff-web --bin export-web-labels -- web/review/src/wasm/labels
bindgen="${WASM_BINDGEN:-$root/target/web-tools/bin/wasm-bindgen}"
if [[ ! -x "$bindgen" && -z "${WASM_BINDGEN:-}" ]]; then bindgen=wasm-bindgen; fi
version="$(bash "$root/scripts/wasm-bindgen-version.sh")"
if ! command -v "$bindgen" >/dev/null || [[ "$("$bindgen" --version)" != "wasm-bindgen $version" ]]; then
  echo "From the repository root, run: cargo install --locked --version $version wasm-bindgen-cli --root target/web-tools" >&2
  echo "Or set WASM_BINDGEN to a matching executable." >&2
  exit 1
fi
bash "$root/scripts/web-grammars.sh"
# Grammars link into the engine at run time, sharing its memory, growable function table, stack pointer, and the C library functions tree-sitter lets them import.
link=(-C link-arg=--export-table -C link-arg=--growable-table -C link-arg=--export=__stack_pointer)
for symbol in $(node -p 'require("./web/review/src/wasm/grammars.json").imports.join(" ")'); do
  link+=(-C "link-arg=--export=$symbol")
done
bash "$root/scripts/with-wasm-toolchain.sh" cargo rustc --locked -p jj-diff-web --lib --crate-type cdylib --target wasm32-unknown-unknown --release -- "${link[@]}"
"$bindgen" target/wasm32-unknown-unknown/release/jj_diff_web.wasm \
  --target web --out-dir web/review/src/wasm/engine --out-name jj_diff --experimental-reset-state-function
