#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
bindgen="${WASM_BINDGEN:-$root/target/web-tools/bin/wasm-bindgen}"
if [[ ! -x "$bindgen" && -z "${WASM_BINDGEN:-}" ]]; then bindgen=wasm-bindgen; fi
version="$(bash "$root/scripts/wasm-bindgen-version.sh")"
if ! command -v "$bindgen" >/dev/null || [[ "$("$bindgen" --version)" != "wasm-bindgen $version" ]]; then
  echo "From the repository root, run: cargo install --locked --version $version wasm-bindgen-cli --root target/web-tools" >&2
  echo "Or set WASM_BINDGEN to a matching executable." >&2
  exit 1
fi
for pack in base rust web python swift shell data systems jvm other; do
  features=(--no-default-features)
  if [[ "$pack" != base ]]; then features+=(--features "$pack"); fi
  bash "$root/scripts/with-wasm-toolchain.sh" cargo build --locked -p jj-diff-web "${features[@]}" --target wasm32-unknown-unknown --release
  "$bindgen" target/wasm32-unknown-unknown/release/jj_diff_web.wasm \
    --target web --out-dir "packages/review-web/src/wasm/$pack" --out-name "jj_diff_$pack"
done
