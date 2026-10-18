#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

lib_name=libjayjay_uniffi.a
uniffi_config="$root/crates/jayjay-uniffi/uniffi.toml"
bindings_dir="$root/build/bindings"
headers_dir="$bindings_dir/Headers"

wrap() {
  local profile="$1" staticlib="$2"
  local args=(-p jayjay-uniffi --bin uniffi-bindgen)
  mkdir -p "${bindings_dir}" "${headers_dir}" "${swift_out}"
  [[ "$profile" != release ]] || args=(--release "${args[@]}")
  cargo build "${args[@]}"
  "${target_dir}/${profile}/uniffi-bindgen" generate \
    --library "${staticlib}" \
    --language swift \
    --out-dir "${bindings_dir}" \
    --config "${uniffi_config}"
  cp "${bindings_dir}/JayJayFFI.h" "${headers_dir}/"
  cp "${bindings_dir}/JayJayFFI.modulemap" "${headers_dir}/module.modulemap"
  cp "${bindings_dir}/JayJayCore.swift" "${swift_out}/"
  rm -rf "${xcframework}"
  xcodebuild -create-xcframework \
    -library "${staticlib}" \
    -headers "${headers_dir}" \
    -output "${xcframework}" | xcbeautify
}

case "$1" in
  debug)
    staticlib="${target_dir}/debug/${lib_name}"
    cargo build -p jayjay-uniffi
    test -f "$staticlib"
    if cmp -s "$staticlib" "${xcframework}/macos-arm64/${lib_name}" \
       && [[ "${swift_out}/JayJayCore.swift" -nt "$staticlib" \
       && "${swift_out}/JayJayCore.swift" -nt "${uniffi_config}" ]]; then
      echo "Bindings and XCFramework are up to date"
      exit 0
    fi
    wrap debug "$staticlib"
    ;;
  release)
    macos_target="$2"
    deployment_target="$3"
    macos_min_flag="-mmacosx-version-min=$deployment_target"
    staticlib="${target_dir}/${macos_target}/release/${lib_name}"
    MACOSX_DEPLOYMENT_TARGET="${deployment_target}" \
    CFLAGS_aarch64_apple_darwin="${macos_min_flag}" \
    CXXFLAGS_aarch64_apple_darwin="${macos_min_flag}" \
    RUSTFLAGS="-C link-arg=${macos_min_flag}" \
    cargo build --release --target "${macos_target}" -p jayjay-uniffi
    test -f "$staticlib"
    wrap release "$staticlib"
    ;;
  wrap) wrap "$2" "$3" ;;
  *) echo "usage: $0 {debug|release <target> <deployment-target>|wrap <profile> <staticlib>}" >&2; exit 2 ;;
esac
