#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bash "$root/scripts/with-wasm-toolchain.sh" cargo build --locked -p jayjay-uniffi --no-default-features --features wasm --target wasm32-unknown-unknown --lib
