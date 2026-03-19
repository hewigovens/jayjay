set shell := ["bash", "-euo", "pipefail", "-c"]
set positional-arguments

root := justfile_directory()
project := root / "shell" / "mac"
derived_data := root / "build" / "DerivedData"
app := derived_data / "Build" / "Products" / "Debug" / "JayJay.app"
cli := root / "bin" / "jayjay"
crate_name := "jayjay_uniffi"
lib_name := "lib" + crate_name + ".a"
bindings_dir := root / "build" / "bindings"
headers_dir := bindings_dir / "Headers"
xcframework := project / "JayJayFFI.xcframework"
swift_out := project / "Sources" / "JayJayBindings"
macos_target := "aarch64-apple-darwin"
deployment_target := "14.0"
bindgen_bin := root / "target" / "release" / "uniffi-bindgen"
macos_min_flag := "-mmacosx-version-min=" + deployment_target

default:
  @just list

list:
  @echo "just list              Show available commands"
  @echo "just test              Run Rust tests"
  @echo "just ffi               Rebuild Rust FFI, bindings, and XCFramework"
  @echo "just build             Build the macOS app"
  @echo "just run               Build and launch the app"
  @echo "just run /path/to/repo Build and launch the app for a repo"
  @echo "just install-cli       Install the jayjay launcher into ~/.local/bin"

test:
  cargo test --workspace

ffi:
  mkdir -p "{{bindings_dir}}" "{{headers_dir}}" "{{swift_out}}"
  MACOSX_DEPLOYMENT_TARGET="{{deployment_target}}" \
  CFLAGS_aarch64_apple_darwin="{{macos_min_flag}}" \
  CXXFLAGS_aarch64_apple_darwin="{{macos_min_flag}}" \
  RUSTFLAGS="-C link-arg={{macos_min_flag}}" \
  cargo build --release --target "{{macos_target}}" -p jayjay-uniffi
  cargo build --release -p jayjay-uniffi --bin uniffi-bindgen
  "{{bindgen_bin}}" generate \
    --library "target/{{macos_target}}/release/{{lib_name}}" \
    --language swift \
    --out-dir "{{bindings_dir}}" \
    --config "crates/jayjay-uniffi/uniffi.toml"
  cp "{{bindings_dir}}/JayJayFFI.h" "{{headers_dir}}/"
  cp "{{bindings_dir}}/JayJayFFI.modulemap" "{{headers_dir}}/module.modulemap"
  cp "{{bindings_dir}}/JayJayBindings.swift" "{{swift_out}}/"
  rm -rf "{{xcframework}}"
  xcodebuild -create-xcframework \
    -library "target/{{macos_target}}/release/{{lib_name}}" \
    -headers "{{headers_dir}}" \
    -output "{{xcframework}}" | xcbeautify

build:
  xcodegen --spec "{{project}}/project.yml" --project "{{project}}"
  just ffi
  xcodebuild \
    -project "{{project}}/JayJay.xcodeproj" \
    -scheme JayJay \
    -configuration Debug \
    -sdk macosx \
    -derivedDataPath "{{derived_data}}" \
    build | xcbeautify
  codesign --force --sign - "{{app}}"
  @echo "Built {{app}}"

run repo='': build
  @if [[ -n "{{repo}}" ]]; then \
    repo_path="$(python3 -c 'import os,sys; print(os.path.abspath(sys.argv[1]))' "{{repo}}")"; \
    open -n "{{app}}" --args --repo "$repo_path"; \
  else \
    open -n "{{app}}"; \
  fi

install-cli:
  cargo build --release -p jayjay-cli
  mkdir -p "$HOME/.local/bin"
  cp "target/release/jayjay" "$HOME/.local/bin/jayjay"
  @echo "Installed jayjay to $HOME/.local/bin/jayjay"
