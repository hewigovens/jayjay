set unstable
set shell := ["bash", "-euo", "pipefail", "-c"]
set positional-arguments

root := justfile_directory()

mod shell
mod worker 'infra/worker.just'

default:
  @just list

list:
  @echo "just list              Show available commands"
  @echo "just test              Run Rust tests"
  @echo "just test-app          Run macOS app tests"
  @echo "just test-ui           Run macOS app UI tests (needs fixture — see shell/mac/Tests/JayJayUITests/Support/SceneBase.swift)"
  @echo "just test-gpui         Run GPUI shell tests (via shell::gpui-test, needs jj on PATH)"
  @echo "just format            Format Rust and Swift sources"
  @echo "just lint              Lint Rust (clippy) and Swift (swiftlint)"
  @echo "just clean             Remove generated build artifacts"
  @echo "just build             Build the macOS app"
  @echo "just run               Build and launch the app"
  @echo "just run /path/to/repo Build and launch the app for a repo"
  @echo "just release           Build, sign, notarize, and package for release"
  @echo "just release-dry-run   Build and package without signing/notarization"
  @echo "just install-cli       Install the jayjay launcher into ~/.local/bin"
  @echo "just shell::gpui-run /path Build and launch the GPUI shell (alpha)"
  @echo "just gpui-appimage     Build the GPUI Linux AppImage"
  @echo "just worker::list      Show Cloudflare Worker/D1 recipes"

test:
  cargo test --workspace

test-app:
  just shell::test

test-ui:
  just shell::ui-test

test-gpui:
  just shell::gpui-test

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

gpui:
  just shell::gpui-run

gpui-appimage:
  just shell::gpui-appimage

format:
  cargo fmt
  just shell::format

lint:
  cargo clippy --workspace --all-targets -- -D warnings
  just shell::lint

clean:
  cargo clean
  just shell::clean

set-version new_version new_build:
  just shell::set-version "{{new_version}}" "{{new_build}}"

check-version:
  just shell::check-version

release:
  just worker::check-migrations
  just shell::release

release-dry-run:
  just worker::check-migrations
  just shell::release-dry-run

install-cli:
  cargo build --release -p jayjay-cli
  mkdir -p "$HOME/.local/bin"
  cp "target/release/jayjay" "$HOME/.local/bin/jayjay"
  @echo "Installed jayjay to $HOME/.local/bin/jayjay"
