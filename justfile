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
  @echo "just test-rust crate   Package-scoped cargo test (inner loop)"
  @echo "just test-wasm         Link the portable UniFFI WASM surface with LLVM clang"
  @echo "just test-ui [test-ids] UI tests; pass test ids to run some scenes"
  @echo "just test-ui-shard i n UI tests for shard i of n, balanced by test count; CI shards"
  @echo "just test              All workspace Rust tests (publish)"
  @echo "just test-app          Run macOS app tests"
  @echo "just test-gpui         Run GPUI shell tests (via shell::gpui-test, needs jj on PATH)"
  @echo "just test-linux        Run the Rust tests on Linux in the OrbStack machine gpui-test"
  @echo "just ffi               Rebuild UniFFI Swift bindings"
  @echo "just format            Format Rust and Swift sources (publish)"
  @echo "just lint              Lint Rust (clippy) and Swift (swiftlint) (publish)"
  @echo "just clean             Remove generated build artifacts"
  @echo "just build             Build the macOS app"
  @echo "just run               Build and launch the app"
  @echo "just run /path/to/repo Build and launch the app for a repo"
  @echo "just release           Build, sign, notarize, and package for release"
  @echo "just release-dry-run   Build and package without signing/notarization"
  @echo "just install-cli       Install the jayjay launcher into ~/.local/bin"
  @echo "just gpui [path]       Build and launch GPUI; reopen the last repo when no path is given"
  @echo "just gpui-appimage     Build the GPUI Linux AppImage"
  @echo "just worker::list      Show Cloudflare Worker/D1 recipes"

# Inner-loop Rust tests. Example: just test-rust jayjay-core
# just test-rust jayjay-core working_copy
# just test-rust jayjay-core --lib wrap
test-rust crate *args:
  cargo test -p "{{crate}}" {{args}}

test-wasm:
  #!/usr/bin/env bash
  set -euo pipefail
  JAYJAY_WASM_HEADERS="$(cargo metadata --locked --format-version 1 --filter-platform wasm32-unknown-unknown | python3 -c 'import json, pathlib, sys; p = next(p for p in json.load(sys.stdin)["packages"] if p["name"] == "tree-sitter-language"); print(pathlib.Path(p["manifest_path"]).parent / "wasm/include")')"
  export JAYJAY_WASM_HEADERS
  # Apple's ar silently discards WASM objects, so use the Rust toolchain's LLVM archiver.
  llvm_ar="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin/llvm-ar"
  if [[ ! -x "$llvm_ar" ]]; then
    echo "WASM linking needs LLVM tools; run rustup component add llvm-tools" >&2
    exit 1
  fi
  # Bump the define when the wrapper-injected sysroot changes; grammar build scripts cannot track those headers themselves.
  AR_wasm32_unknown_unknown="$llvm_ar" CC_wasm32_unknown_unknown="{{root}}/scripts/llvm-clang" CXX_wasm32_unknown_unknown="{{root}}/scripts/llvm-clang" CFLAGS_wasm32_unknown_unknown="-DJAYJAY_WASM_SYSROOT_REV=5" CXXFLAGS_wasm32_unknown_unknown="-DJAYJAY_WASM_SYSROOT_REV=5" cargo build --locked -p jayjay-uniffi --no-default-features --features wasm --target wasm32-unknown-unknown --lib

test:
  cargo test --workspace

test-app:
  just shell::test

test-ui *test_ids:
  just shell::ui-test {{test_ids}}

# Balance whole UI scene classes across count shards by test-method count (every test relaunches the app), so CI can split the serial XCUITest bundle across runners; index is 1-based and `recipe` selects the shell runner (ui-test builds, ui-test-prebuilt reuses a ui-test-build output).
test-ui-shard index count recipe="shell::ui-test":
  #!/usr/bin/env bash
  set -euo pipefail
  scenes=$(cd "{{root}}/shell/mac/Tests/JayJayUITests/Scenes" && grep -c '^ *func test' *.swift | LC_ALL=C sort -t: -k2,2nr -k1,1 \
    | awk -F: -v i="{{index}}" -v n="{{count}}" '{ m = 1; for (s = 2; s <= n; s++) if (load[s] < load[m]) m = s; load[m] += $2; if (m == i) { sub(/\.swift$/, "", $1); print "JayJayUITests/" $1 } }')
  [[ -n "$scenes" ]] || { echo "UI shard {{index}}/{{count}} has no scenes" >&2; exit 1; }
  just {{recipe}} $scenes

test-gpui:
  just shell::gpui-test

# Linux-only modules never compile on macOS, so run their tests in the OrbStack machine from agents/testing.md. Example: just test-linux -p jayjay-gpui menu_item
test-linux *args='--workspace':
  orb run -m gpui-test bash -lc 'cd "{{justfile_directory()}}" && CARGO_TARGET_DIR="$HOME/target-linux" RUSTC_WRAPPER= cargo test {{args}}'

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

gpui repo='':
  just shell::gpui-run "$1"

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

verify-release-base:
  just shell::verify-release-base

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
