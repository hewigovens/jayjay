#!/usr/bin/env bash
set -euo pipefail

# The wasm-bindgen CLI must match the crate version Cargo.lock pins, or it rejects the module's schema.
awk '/^name = "wasm-bindgen"$/ { getline; gsub(/"/, "", $3); print $3; exit }' "$(dirname "${BASH_SOURCE[0]}")/../Cargo.lock"
