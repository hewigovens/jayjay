#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
context="$root/scripts/linux-test"
rust_version="$(sed -n 's/^rust-version = "\([^"]*\)"/\1/p' "$root/Cargo.toml")"
jj_version="$(sed -n 's/^jj-lib = { version = "\([^"]*\)".*/\1/p' "$root/Cargo.toml")"
[[ -n "$rust_version" && -n "$jj_version" ]] || { echo "Missing Rust or jj-lib version in Cargo.toml" >&2; exit 1; }
command -v container >/dev/null || { echo "Install Apple's container CLI: brew install container; container system start" >&2; exit 1; }

image_key="$( { printf '%s\n' "$rust_version" "$jj_version"; cat "$context/Containerfile"; } | shasum -a 256 | cut -c 1-16)"
workspace_key="$(printf '%s' "$root" | shasum -a 256 | cut -c 1-16)"
image="jayjay-linux-test:$image_key"
cache="jayjay-linux-test-$workspace_key"

if ! container image inspect "$image" >/dev/null 2>&1; then
  container build --platform linux/arm64 \
    --build-arg "RUST_VERSION=$rust_version" --build-arg "JJ_VERSION=$jj_version" \
    --tag "$image" "$context"
fi
if ! container volume inspect "$cache" >/dev/null 2>&1; then
  container volume create -s 16G "$cache"
fi
printf 'Linux test image: %s; workspace cache volume: %s\n' "$image" "$cache" >&2

[[ $# -gt 0 ]] || set -- --workspace
exec container run --rm --name "$cache-$$" --platform linux/arm64 \
  --cpus "${JAYJAY_LINUX_CPUS:-6}" --memory "${JAYJAY_LINUX_MEMORY:-10G}" \
  --mount "type=bind,source=$root,target=/source,readonly" \
  --mount "type=volume,source=$cache,target=/cache" \
  --env CARGO_HOME=/cache/cargo --env CARGO_TARGET_DIR=/cache/target \
  --env RUSTC_WRAPPER= --env CARGO_INCREMENTAL=0 \
  --env CARGO_PROFILE_DEV_DEBUG=0 --env CARGO_PROFILE_TEST_DEBUG=0 \
  "$image" bash -euo pipefail -c '
    mkdir /workspace
    # Host Git/JJ metadata can point outside the bind mount and break Git config reads.
    tar -C /source --exclude=./.git --exclude=./.jj --exclude=./target --exclude=./build --exclude=.build -cf - . | tar -C /workspace -xf -
    cd /workspace
    exec cargo test --locked "$@"
  ' bash "$@"
