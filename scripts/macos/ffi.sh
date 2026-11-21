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
  cargo run -q -p jayjay-i18n --bin l10n-swift > "${swift_out}/Localized.swift"
  rm -rf "${xcframework}"
  xcodebuild -create-xcframework \
    -library "${staticlib}" \
    -headers "${headers_dir}" \
    -output "${xcframework}" | xcbeautify
}

portable() {
  local out="${1:?portable needs an output directory}"
  shift
  local targets=("$@")
  [[ ${#targets[@]} -gt 0 ]] || targets=(aarch64-apple-darwin aarch64-apple-ios aarch64-apple-ios-sim)
  # Its own target folder: the library's path under a target is the desktop build's too.
  local build="${CARGO_TARGET_DIR:-$target_dir}/portable"
  local bindings="$build/bindings" libraries=() target
  export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-26.0}" IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-26.0}"
  cd "$root"
  for target in "${targets[@]}"; do
    cargo build --locked --release --target "$target" -p jayjay-uniffi --no-default-features --target-dir "$build"
    libraries+=(-library "$build/$target/release/$lib_name" -headers "$bindings/Headers")
  done
  cargo build --locked --release -p jayjay-uniffi --no-default-features --features bindgen --bin uniffi-bindgen --target-dir "$build"
  rm -rf "$bindings" "$out/Package.swift" "$out/Sources" "$out/Packages" "$out/JayJayFFI.xcframework"
  mkdir -p "$bindings/Headers" "$out/Sources/JayJayCore" "$out/Packages"
  "$build/release/uniffi-bindgen" generate \
    --library "$build/${targets[0]}/release/$lib_name" \
    --language swift \
    --out-dir "$bindings" \
    --config "$uniffi_config"
  cp "$bindings/JayJayFFI.h" "$bindings/Headers/"
  cp "$bindings/JayJayFFI.modulemap" "$bindings/Headers/module.modulemap"
  cp "$project/Package.swift" "$out/"
  cp "$swift_out"/*+*.swift "$bindings/JayJayCore.swift" "$out/Sources/JayJayCore/"
  cargo run --locked -q --release -p jayjay-i18n --bin l10n-swift --target-dir "$build" > "$out/Sources/JayJayCore/Localized.swift"
  rsync -a --exclude .build --exclude .swiftpm --exclude .DS_Store "$project/Packages/" "$out/Packages/"
  xcodebuild -create-xcframework "${libraries[@]}" -output "$out/JayJayFFI.xcframework" >/dev/null
  echo "$out"
}

case "$1" in
  debug)
    staticlib="${target_dir}/debug/${lib_name}"
    cargo build -p jayjay-uniffi
    bash "$root/scripts/ssh-helper.sh" "" debug
    test -f "$staticlib"
    if cmp -s "$staticlib" "${xcframework}/macos-arm64/${lib_name}" \
       && [[ "${swift_out}/JayJayCore.swift" -nt "$staticlib" \
       && "${swift_out}/JayJayCore.swift" -nt "${uniffi_config}" \
       && "${swift_out}/Localized.swift" -nt "$staticlib" ]]; then
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
  portable)
    shift
    portable "$@"
    ;;
  *) echo "usage: $0 {debug|release <target> <deployment-target>|wrap <profile> <staticlib>|portable <out-dir> [target...]}" >&2; exit 2 ;;
esac
