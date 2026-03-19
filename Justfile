set shell := ["bash", "-euo", "pipefail", "-c"]
set positional-arguments

root := justfile_directory()
project := root / "macos"
web := project / "Web"
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

default:
  @just list

list:
  @echo "just list              Show available commands"
  @echo "just test              Run Rust tests"
  @echo "just web-assets        Install and stage Monaco web assets"
  @echo "just ffi               Rebuild Rust FFI, bindings, and XCFramework"
  @echo "just build             Build the macOS app"
  @echo "just run               Build and launch the app"
  @echo "just run /path/to/repo Build and launch the app for a repo"
  @echo "just install-cli       Install the jayjay launcher into ~/.local/bin"

test:
  cargo test --workspace

web-assets:
  mkdir -p "{{project}}/Resources/WebDiff/vendor/monaco"
  if [[ ! -d "{{web}}/node_modules/monaco-editor" ]]; then npm --prefix "{{web}}" install --no-fund --no-audit; fi
  rsync -a --delete "{{web}}/node_modules/monaco-editor/min/vs/" "{{project}}/Resources/WebDiff/vendor/monaco/vs/"

ffi:
  mkdir -p "{{bindings_dir}}" "{{headers_dir}}" "{{swift_out}}"
  MACOSX_DEPLOYMENT_TARGET="{{deployment_target}}" \
  cargo build --release --target "{{macos_target}}" -p jayjay-uniffi
  MACOSX_DEPLOYMENT_TARGET="{{deployment_target}}" \
  cargo run --release -p jayjay-uniffi --bin uniffi-bindgen generate \
    --library "target/{{macos_target}}/release/{{lib_name}}" \
    --language swift \
    --out-dir "{{bindings_dir}}"
  cp "{{bindings_dir}}/{{crate_name}}FFI.h" "{{headers_dir}}/"
  cp "{{bindings_dir}}/{{crate_name}}FFI.modulemap" "{{headers_dir}}/module.modulemap"
  cp "{{bindings_dir}}/{{crate_name}}.swift" "{{swift_out}}/"
  rm -rf "{{xcframework}}"
  xcodebuild -create-xcframework \
    -library "target/{{macos_target}}/release/{{lib_name}}" \
    -headers "{{headers_dir}}" \
    -output "{{xcframework}}" | xcbeautify

build:
  xcodegen --spec "{{project}}/project.yml" --project "{{project}}"
  just web-assets
  just ffi
  xcodebuild \
    -project "{{project}}/JayJay.xcodeproj" \
    -scheme JayJay \
    -configuration Debug \
    -sdk macosx \
    -derivedDataPath "{{derived_data}}" \
    build | xcbeautify
  mkdir -p "{{app}}/Contents/Resources/WebDiff"
  rsync -a --delete "{{project}}/Resources/WebDiff/" "{{app}}/Contents/Resources/WebDiff/"
  codesign --force --sign - "{{app}}"
  @echo "Built {{app}}"

run repo='':
  just build
  @if [[ -n "{{repo}}" ]]; then \
    repo_path="$(python3 -c 'import os,sys; print(os.path.abspath(sys.argv[1]))' "{{repo}}")"; \
    open -n "{{app}}" --args --repo "$repo_path"; \
  else \
    open -n "{{app}}"; \
  fi

install-cli:
  mkdir -p "$HOME/.local/bin"
  ln -sf "{{cli}}" "$HOME/.local/bin/jayjay"
  @echo "Installed jayjay to $HOME/.local/bin/jayjay"
