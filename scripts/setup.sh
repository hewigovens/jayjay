#!/usr/bin/env bash
# Dev bootstrap: brew + rust + Brewfile tools, configure jj fix, check Xcode.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
os="$(uname -s)"

# have <cmd> <label> <install-fn>: report if present, else run the install fn.
have() {
  if command -v "$1" >/dev/null 2>&1; then
    echo "✓ $2 already installed"
  else
    echo "→ installing $2"
    "$3"
  fi
}

install_brew() {
  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
  # brew isn't on PATH right after install; source it from the per-platform prefix.
  for path in /opt/homebrew/bin/brew /usr/local/bin/brew /home/linuxbrew/.linuxbrew/bin/brew "$HOME/.linuxbrew/bin/brew"; do
    [ -x "$path" ] && eval "$("$path" shellenv)" && break
  done
}

brew_bundle() {
  HOMEBREW_NO_AUTO_UPDATE=1 brew bundle --quiet --file "$root/Brewfile"
}

install_rust() {
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1091
  . "$HOME/.cargo/env"
}

configure_jj_fix() {
  # A git clone has no .jj yet — colocate a jj workspace so jj works here.
  if [ ! -d "$root/.jj/repo" ]; then
    echo "→ initializing jj workspace (colocated with git)"
    (cd "$root" && jj git init --colocate)
  fi
  if [ -f "$root/.jj/repo/config.toml" ]; then
    echo "✓ jj fix already configured (.jj/repo/config.toml)"
  else
    cp "$root/.jj-config.toml" "$root/.jj/repo/config.toml"
    echo "✓ configured jj fix (copied .jj-config.toml)"
  fi
}

check_xcode() {
  [ "$os" = "Darwin" ] || return 0
  if xcodebuild -version >/dev/null 2>&1; then
    echo "✓ Xcode command-line build available"
  else
    echo "✗ Xcode not found or not selected. Install Xcode 16+ from the App Store, then:"
    echo "    sudo xcode-select -s /Applications/Xcode.app/Contents/Developer"
    echo "    sudo xcodebuild -license accept"
  fi
}

have brew Homebrew install_brew
have cargo Rust install_rust
brew_bundle
configure_jj_fix
check_xcode

echo
echo "Setup complete. Try: just build"
