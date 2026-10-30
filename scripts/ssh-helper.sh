#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
target="${1:-$(rustc -vV | sed -n 's/^host: //p')}"
profile="${2:-debug}"
case "$profile" in debug) args=(--profile dev) ;; release) args=(--release) ;; *) echo "usage: $0 [target] [debug|release]" >&2; exit 2 ;; esac
destination="$root/build/ssh-helpers/$target"
mkdir -p "$destination"
cd "$root"

if [[ "$target" == *-linux-musl && "$(uname -s)" == Darwin ]]; then
  command -v container >/dev/null || { echo "Install Apple's container CLI: brew install container; container system start" >&2; exit 1; }
  case "$target" in aarch64-*) platform=linux/arm64 ;; x86_64-*) platform=linux/amd64 ;; *) echo "unsupported target: $target" >&2; exit 2 ;; esac
  rust_version="$(sed -n 's/^rust-version = "\([^"]*\)"/\1/p' Cargo.toml)"
  cache="jayjay-ssh-helper-$(printf '%s' "$root$platform" | shasum -a 256 | cut -c 1-16)"
  container volume inspect "$cache" >/dev/null 2>&1 || container volume create -s 16G "$cache" >/dev/null
  exec container run --rm --platform "$platform" \
    --cpus "${JAYJAY_LINUX_CPUS:-6}" --memory "${JAYJAY_LINUX_MEMORY:-10G}" \
    --mount "type=bind,source=$root,target=/source,readonly" \
    --mount "type=bind,source=$destination,target=/out" \
    --mount "type=volume,source=$cache,target=/cache" \
    --env CARGO_HOME=/cache/cargo --env CARGO_TARGET_DIR=/cache/target \
    "rust:$rust_version-slim-trixie" bash -euo pipefail -c '
      apt-get update -qq && apt-get install -y -qq --no-install-recommends musl-tools >/dev/null
      rustup target add "$0" >/dev/null
      mkdir /workspace
      tar -C /source --exclude=./.git --exclude=./.jj --exclude=./target --exclude=./build -cf - . | tar -C /workspace -xf -
      cd /workspace
      cargo build --locked -p jayjay-core --bin jayjay-ssh --target "$0" "${@:2}"
      cp "/cache/target/$0/$1/jayjay-ssh" /out/jayjay-ssh
    ' "$target" "$profile" "${args[@]}"
fi

exe=""
build=(cargo build)
if [[ "$target" == *-windows-msvc ]]; then
  exe=.exe
  # Windows hosts are not expected to have the Visual C++ runtime installed.
  export RUSTFLAGS="-Ctarget-feature=+crt-static"
  if [[ "$(uname -s)" == Darwin ]]; then
    PATH="$(brew --prefix llvm)/bin:$(brew --prefix lld)/bin:$PATH"
    build=(cargo xwin build --cross-compiler clang)
  fi
fi
"${build[@]}" --locked -p jayjay-core --bin jayjay-ssh --target "$target" "${args[@]}"
cp "$root/target/$target/$profile/jayjay-ssh$exe" "$destination/jayjay-ssh$exe"
