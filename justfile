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
  @echo "just test-linux        Run the Rust tests on Linux with Apple container"
  @echo "just ffi               Rebuild UniFFI Swift bindings"
  @echo "just format            Format Rust and Swift sources (publish)"
  @echo "just lint              Lint Rust (clippy) and Swift (swiftlint) (publish)"
  @echo "just profile diff|refresh [--alloc] args  Hotpath profile of jj-diff or a repo refresh"
  @echo "just clean             Remove generated build artifacts"
  @echo "just build             Build the macOS app"
  @echo "just run               Build and launch the app"
  @echo "just run /path/to/repo Build and launch the app for a repo"
  @echo "just release           Build, sign, notarize, and package for release"
  @echo "just release-dry-run   Build and package without signing/notarization"
  @echo "just install-cli       Install the jayjay launcher into ~/.local/bin"
  @echo "just gpui [path]       Build and launch GPUI; reopen the last repo when no path is given"
  @echo "just gpui-appimage     Build the GPUI Linux AppImage"
  @echo "just gpui-windows      Cross-build the x64 Windows GPUI exe for VM testing"
  @echo "just worker::list      Show Cloudflare Worker/D1 recipes"

# Inner-loop Rust tests. Example: just test-rust jayjay-core
# just test-rust jayjay-core working_copy
# just test-rust jayjay-core --lib wrap
test-rust crate *args:
  cargo test -p "{{crate}}" {{args}}

test-wasm:
  bash "{{root}}/scripts/test-wasm.sh"

test:
  cargo test --workspace

# Example: just profile diff sbs; just profile refresh ~/src/repo graph; just profile refresh --alloc ~/src/repo
# Scenarios and caveats: crates/jj-diff/benches/profile_diff.md, crates/jayjay-core/benches/profile_refresh.md
profile kind *args:
  #!/usr/bin/env bash
  set -euo pipefail
  shift
  features=hotpath
  if [[ "${1:-}" == --alloc ]]; then
    features=hotpath-alloc
    shift
  fi
  case "{{kind}}" in
    diff) crate=jj-diff bench=profile_diff ;;
    refresh) crate=jayjay-core bench=profile_refresh ;;
    *) echo "usage: just profile diff|refresh [--alloc] [args...]" >&2; exit 1 ;;
  esac
  HOTPATH_METRICS_SERVER_OFF=1 cargo bench --locked -p "$crate" --bench "$bench" --features "$features" -- "$@"

test-app:
  just shell::test

test-ui *test_ids:
  just shell::ui-test {{test_ids}}

# Balance whole UI scene classes across count shards by test-method count (every test relaunches the app), so CI can split the serial XCUITest bundle across runners; index is 1-based and `recipe` selects the shell runner (ui-test builds, ui-test-prebuilt reuses a ui-test-build output).
test-ui-shard index count recipe="shell::ui-test":
  bash "{{root}}/scripts/test-ui-shard.sh" "$@"

test-gpui:
  just shell::gpui-test

# Linux-only modules never compile on macOS. Example: just test-linux -p jayjay-gpui menu_item
test-linux *args='--workspace':
  bash "{{root}}/scripts/test-linux.sh" "$@"

build:
  just shell::build

fix:
  jj fix

ffi:
  just shell::ffi

run repo='':
  @just shell::run "$1"

gpui repo='':
  just shell::gpui-run "$1"

gpui-appimage:
  just shell::gpui-appimage

gpui-windows:
  just shell::gpui-windows

format:
  cargo fmt
  just shell::format

lint:
  cargo clippy --workspace --all-targets -- -D warnings
  just shell::lint
  just lint-i18n

# Files fully migrated to the jayjay-i18n catalog.
i18n_migrated := "shell/gpui/src/windows/settings shell/gpui/src/windows/identity_dialog.rs shell/gpui/src/repo shell/gpui/src/app/menus.rs shell/gpui/src/diff shell/mac/Sources/JayJay/Settings shell/mac/Sources/JayJay/Repo shell/mac/Sources/JayJay/Detail shell/mac/Sources/JayJay/Diff shell/mac/Sources/JayJay/DiffEdit shell/mac/Packages/JayJayDiffUI/Sources shell/mac/Sources/JayJay/App/Window shell/mac/Sources/JayJay/App/AppCommands.swift shell/mac/Sources/JayJay/App/AppInfoCommands.swift shell/mac/Sources/JayJay/App/HelpCommands.swift shell/gpui/src/windows/command_palette shell/mac/Sources/JayJay/Shared/CommandPalette shell/gpui/src/windows/overview shell/gpui/src/windows/overview.rs shell/mac/Sources/JayJay/Overview shell/gpui/src/windows/evolog shell/gpui/src/windows/evolog.rs shell/gpui/src/windows/operation_log"
# Proper names that stay English in every locale.
i18n_names := "JayJay|Codex CLI|Claude CLI|Codex|Claude|Terminal|Finder|Frog"

lint-i18n:
  #!/usr/bin/env bash
  set -euo pipefail
  status=0
  camel() { awk -F'[-_]' '{ printf "%s", $1; for (i = 2; i <= NF; i++) printf "%s", toupper(substr($i, 1, 1)) substr($i, 2) }' <<< "$1"; }
  for file in crates/jayjay-i18n/locales/en-US/*.ftl; do
    stem=$(basename "$file" .ftl)
    swift_enum=$(camel "$stem")
    swift_enum=$(tr a-z A-Z <<< "${swift_enum:0:1}")${swift_enum:1}
    for id in $(sed -n "s/^\(${stem}-[a-z0-9-]*\) =.*/\1/p" "$file"); do
      name=${id#"$stem"-}
      if ! grep -rqE --include='*.rs' "${stem}::${name//-/_}([^a-z0-9_]|$)" crates shell/gpui/src --exclude-dir=jayjay-i18n \
        && ! grep -rqE --include='*.swift' "Localized\.${swift_enum}\.$(camel "$name")([^A-Za-z0-9_]|$)" shell/mac/Sources shell/mac/Packages; then
        echo "unused message: $id ($file)"
        status=1
      fi
    done
  done
  hits=$(find {{i18n_migrated}} -type f \( -name '*.rs' -o -name '*.swift' \) ! -name 'tests.rs' -print0 \
    | xargs -0 awk -v names='^"({{i18n_names}})"$' '
      FNR == 1 { skip = 0 }
      /#\[cfg\(test\)\]/ { skip = 1 }
      skip || /^[[:space:]]*\/\// || /key_context|KeyContext|assert|precondition|fatalError|\.contains\(/ { next }
      {
        rest = $0
        while (match(rest, /"[A-Z][a-z]+([ ,.…?!&:'\''][^"]*)?"/)) {
          literal = substr(rest, RSTART, RLENGTH)
          if (literal !~ names) { print FILENAME ":" FNR ": " $0; break }
          rest = substr(rest, RSTART + RLENGTH)
        }
      }')
  if [ -n "$hits" ]; then
    echo "$hits"
    echo "hardcoded user-facing text in a migrated file; add it to crates/jayjay-i18n/locales/en-US"
    status=1
  fi
  exit $status

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
