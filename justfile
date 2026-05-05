set unstable
set shell := ["bash", "-euo", "pipefail", "-c"]
set positional-arguments

root := justfile_directory()

mod shell

default:
  @just list

list:
  @echo "just list              Show available commands"
  @echo "just test              Run Rust tests"
  @echo "just test-app          Run macOS app tests"
  @echo "just test-ui           Run macOS app UI tests (needs fixture — see shell/mac/Tests/JayJayUITests/Support/SceneBase.swift)"
  @echo "just test-gpui         Run GPUI shell tests (component tests via gpui::test, needs jj on PATH)"
  @echo "just format            Format Rust and Swift sources"
  @echo "just lint              Lint Rust (clippy) and Swift (swiftlint)"
  @echo "just clean             Remove generated build artifacts"
  @echo "just build             Build the macOS app"
  @echo "just run               Build and launch the app"
  @echo "just run /path/to/repo Build and launch the app for a repo"
  @echo "just release           Build, sign, notarize, and package for release"
  @echo "just release-dry-run   Build and package without signing/notarization"
  @echo "just install-cli       Install the jayjay launcher into ~/.local/bin"

test:
  cargo test --workspace

test-app:
  just shell::test

test-ui:
  just shell::ui-test

test-gpui:
  cargo test -p jayjay-gpui

build:
  just shell::build

fix:
  jj fix

ffi:
  just shell::ffi

run repo='':
  @if [[ -n "{{repo}}" ]]; then \
    just shell::run "{{repo}}"; \
  else \
    just shell::run; \
  fi

format:
  cargo fmt
  just shell::format

lint:
  cargo clippy --workspace
  just shell::lint

clean:
  cargo clean
  just shell::clean

release:
  just shell::release

release-dry-run:
  just shell::release-dry-run

install-cli:
  cargo build --release -p jayjay-cli
  mkdir -p "$HOME/.local/bin"
  cp "target/release/jayjay" "$HOME/.local/bin/jayjay"
  @echo "Installed jayjay to $HOME/.local/bin/jayjay"

# Build a .app bundle for the GPUI shell (ad-hoc signed so launchd accepts it).
gpui-bundle:
  @cargo build -p jayjay-gpui
  @mkdir -p build/gpui/JayJay.app/Contents/MacOS
  @mkdir -p build/gpui/JayJay.app/Contents/Resources
  @cp shell/gpui/Info.plist build/gpui/JayJay.app/Contents/Info.plist
  @cp assets/AppIcon.icns build/gpui/JayJay.app/Contents/Resources/AppIcon.icns
  @cp target/debug/jayjay-gpui build/gpui/JayJay.app/Contents/MacOS/jayjay-gpui
  @xattr -dr com.apple.quarantine build/gpui/JayJay.app 2>/dev/null || true
  @codesign --force --deep --sign - build/gpui/JayJay.app
  @touch build/gpui/JayJay.app
  @echo "Built build/gpui/JayJay.app"

# Run the GPUI shell .app. Pass a repo path or default to cwd.
# Kills any previously-launched jayjay-gpui first so we don't end up with
# two copies of the alpha shell racing each other.
gpui repo='': gpui-bundle
  @pkill -f jayjay-gpui 2>/dev/null || true
  @repo="{{repo}}"; if [[ -z "$repo" ]]; then repo="$PWD"; fi; \
    open -n -a "$PWD/build/gpui/JayJay.app" --args "$repo"
