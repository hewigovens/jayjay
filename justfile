set unstable
set shell := ["bash", "-euo", "pipefail", "-c"]
set positional-arguments

mod shell

default:
  @just list

list:
  @echo "just list              Show available commands"
  @echo "just test              Run Rust tests"
  @echo "just build             Build the macOS app"
  @echo "just run               Build and launch the app"
  @echo "just run /path/to/repo Build and launch the app for a repo"
  @echo "just install-cli       Install the jayjay launcher into ~/.local/bin"
  @echo ""
  @echo "Submodules:"
  @just --list --list-submodules

test:
  cargo test --workspace

ffi:
  just shell::ffi

build:
  just shell::build

run repo='':
  @if [[ -n "{{repo}}" ]]; then \
    just shell::run "{{repo}}"; \
  else \
    just shell::run; \
  fi

install-cli:
  cargo build --release -p jayjay-cli
  mkdir -p "$HOME/.local/bin"
  cp "target/release/jayjay" "$HOME/.local/bin/jayjay"
  @echo "Installed jayjay to $HOME/.local/bin/jayjay"
