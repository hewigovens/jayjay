#!/usr/bin/env bash
# Installs JayJay for Linux: the Debian package on apt-based systems or a local AppImage elsewhere. Arch packages are available from AUR.
# Usage: curl -fsSL https://jayjay.hewig.dev/install.sh | bash
#        JAYJAY_VERSION=0.3.19 bash install.sh   # a specific release instead of the latest stable
#        curl -fsSL https://jayjay.hewig.dev/install.sh | bash -s -- --beta   # the latest beta when it is newer than stable
set -euo pipefail
shopt -s inherit_errexit

repo="hewigovens/jayjay"

die() {
  echo "$*" >&2
  exit 1
}

require_tools() {
  local tool
  for tool in "$@"; do
    command -v "$tool" >/dev/null || die "$tool is required."
  done
}

run_as_root() {
  if [ "$(id -u)" -eq 0 ]; then
    "$@"
  else
    sudo "$@"
  fi
}

download_with_checksum() {
  local tag="$1" asset="$2" base expected
  base="https://github.com/$repo/releases/download/$tag"
  curl -fsSL --retry 3 -o "$tmp/$asset" "$base/$asset"
  # Compare hashes only: GitHub renames some assets (a Debian ~ becomes .), so the name inside the .sha256 can differ.
  expected="$(curl -fsSL --retry 3 "$base/$asset.sha256" | cut -d' ' -f1)"
  [ "$(sha256sum "$tmp/$asset" | cut -d' ' -f1)" = "$expected" ] || die "Checksum mismatch for $asset."
}

latest_stable_tag() {
  local tag
  tag="$(curl -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/$repo/releases/latest")"
  tag="${tag##*/}"
  case "$tag" in v[0-9]*) printf '%s\n' "$tag" ;; *) die "Could not determine the latest release." ;; esac
}

# JAYJAY_VERSION, else the latest stable; --beta picks the newest beta only when it is ahead of stable, so it never downgrades.
release_tag() {
  local stable beta_tag newest
  if [ -n "${JAYJAY_VERSION:-}" ]; then
    printf 'v%s\n' "${JAYJAY_VERSION#v}"
    return
  fi
  stable="$(latest_stable_tag)"
  if "$beta"; then
    beta_tag="$(curl -fsSL --retry 3 -H 'Accept: application/vnd.github+json' "https://api.github.com/repos/$repo/releases?per_page=100" \
      | sed -n 's/.*"tag_name": *"\(v[0-9]*\.[0-9]*\.[0-9]*-beta\.[0-9]*\)".*/\1/p' | sort -V | tail -n 1)"
    newest="$(printf '%s\n' "${stable#v}" "${beta_tag#v}" | sed 's/-beta\..*//' | sort -V | tail -n 1)"
    if [ -n "$beta_tag" ] && [ "$newest" != "${stable#v}" ]; then
      printf '%s\n' "$beta_tag"
      return
    fi
    echo "No beta is newer than $stable; installing the stable release." >&2
  fi
  printf '%s\n' "$stable"
}

print_next_steps() {
  echo "Run: jayjay /path/to/repo"
  if "$beta"; then
    echo "Upgrade: rerun this script with --beta when a new beta is out."
  else
    echo "Upgrade: run this script again when a new release is out."
  fi
}

install_deb() {
  local tag="$1" deb_version installed deb_arch asset
  deb_version="${tag#v}"
  deb_version="${deb_version/-beta./~beta.}-1"
  installed="$(dpkg-query -W -f='${Version}' jayjay 2>/dev/null || true)"
  if [ -z "${JAYJAY_FORCE:-}" ] && [ -n "$installed" ] && dpkg --compare-versions "$installed" ge "$deb_version"; then
    echo "JayJay $installed is already installed as the jayjay package. Set JAYJAY_FORCE=1 to install $tag anyway."
    return
  fi
  case "$arch" in x86_64) deb_arch=amd64 ;; *) deb_arch=arm64 ;; esac
  # GitHub stores the ~ of a Debian pre-release version as a dot in asset names.
  asset="jayjay_${deb_version/\~/.}_$deb_arch.deb"
  if ! curl -fsL -r 0-0 -o /dev/null "https://github.com/$repo/releases/download/$tag/$asset"; then
    echo "$tag has no Debian package; installing the AppImage instead."
    install_appimage "$tag"
    return
  fi
  require_tools sha256sum
  echo "Installing JayJay $tag for $deb_arch..."
  download_with_checksum "$tag" "$asset"
  # apt reads local packages as its unprivileged _apt user.
  chmod a+rx "$tmp"
  chmod a+r "$tmp/$asset"
  run_as_root apt-get install "$tmp/$asset" </dev/tty
  echo "Installed JayJay $tag as the jayjay package."
  print_next_steps
}

install_appimage() {
  local tag="$1" launcher target asset version_file desktop_entry line
  if command -v pacman >/dev/null && pacman -Q jayjay-appimage >/dev/null 2>&1; then
    echo "JayJay is installed as the jayjay-appimage package; upgrade it through AUR: https://aur.archlinux.org/packages/jayjay-appimage" >&2
    die "To switch to an AppImage (including a beta), remove jayjay-appimage with pacman first, then rerun this installer."
  fi
  require_tools curl sha256sum
  if ! command -v fusermount3 >/dev/null && ! command -v fusermount >/dev/null; then
    die "AppImages need FUSE. Install the fuse3 package (Arch: fuse3, Debian/Ubuntu: fuse3, Fedora: fuse) and run this again."
  fi

  # Mirrors the app's Settings > CLI rule: only a symlink to a JayJay AppImage may be replaced; anything else is the user's.
  for launcher in "$bin_dir/jayjay" "$bin_dir/jayjay-gpui"; do
    if [ -e "$launcher" ] || [ -L "$launcher" ]; then
      target="$(readlink "$launcher" || true)"
      case "$(basename "$target" | tr '[:upper:]' '[:lower:]')" in
        *jayjay*.appimage) ;;
        *) die "$launcher already exists and is not a JayJay launcher; move it aside and run this again." ;;
      esac
    fi
  done

  version_file="$app_dir/VERSION"
  if [ -z "${JAYJAY_FORCE:-}" ] && [ -x "$app_dir/JayJay.AppImage" ] && [ -L "$bin_dir/jayjay" ] && [ "$(cat "$version_file" 2>/dev/null)" = "$tag" ]; then
    echo "JayJay $tag is already installed at $app_dir/JayJay.AppImage. Set JAYJAY_FORCE=1 to reinstall."
    return
  fi
  asset="jayjay-gpui-$arch.AppImage"

  echo "Installing JayJay $tag for $arch..."
  download_with_checksum "$tag" "$asset"
  chmod +x "$tmp/$asset"

  # The runtime extracts named files without FUSE, so the desktop entry and icons come from the image itself. Older images keep both at the root.
  (cd "$tmp" && for pattern in 'usr/share/applications/*' 'usr/share/icons/*' '*.desktop' '*.svg'; do "./$asset" --appimage-extract "$pattern" >/dev/null 2>&1 || true; done)
  desktop_entry="$(find "$tmp/squashfs-root" -type f -name 'dev.hewig.JayJay.desktop' | head -1)"
  [ -n "$desktop_entry" ] || die "The image has no desktop entry."

  mkdir -p "$app_dir" "$bin_dir" "$data_dir/applications" "$data_dir/icons/hicolor/scalable/apps"
  install -m755 "$tmp/$asset" "$app_dir/JayJay.AppImage"
  ln -sfn "$app_dir/JayJay.AppImage" "$bin_dir/jayjay"
  ln -sfn "$app_dir/JayJay.AppImage" "$bin_dir/jayjay-gpui"
  # The menu entry must not depend on the session PATH, so it launches the image by its absolute path, quoted per the desktop-entry rules.
  while IFS= read -r line || [ -n "$line" ]; do
    case "$line" in
      Exec=*) printf 'Exec="%s" %%F\n' "$exec_path" ;;
      *) printf '%s\n' "$line" ;;
    esac
  done < "$desktop_entry" > "$data_dir/applications/dev.hewig.JayJay.desktop"
  if [ -d "$tmp/squashfs-root/usr/share/icons" ]; then
    cp -R "$tmp/squashfs-root/usr/share/icons/." "$data_dir/icons/"
  else
    install -m644 "$tmp/squashfs-root/dev.hewig.JayJay.svg" "$data_dir/icons/hicolor/scalable/apps/dev.hewig.JayJay.svg"
  fi
  command -v update-desktop-database >/dev/null && update-desktop-database "$data_dir/applications" 2>/dev/null || true
  printf '%s\n' "$tag" > "$version_file"

  echo "Installed JayJay $tag to $app_dir/JayJay.AppImage"
  print_next_steps
  case ":$PATH:" in
    *":$bin_dir:"*) ;;
    *) echo "Note: add $bin_dir to your PATH to run jayjay by name." ;;
  esac
}

main() {
  local beta=false option
  local prefix data_dir bin_dir app_dir exec_path arch machine tag
  for option in "$@"; do
    case "$option" in
      --pacman-repo | --no-install) die "The custom pacman repo has been retired. Remove the [jayjay] section from /etc/pacman.conf and install jayjay-appimage from AUR: https://aur.archlinux.org/packages/jayjay-appimage" ;;
      --beta) beta=true ;;
      *) die "Unknown option: $option" ;;
    esac
  done

  # The default install follows the XDG data directory the desktop searches; a custom prefix keeps everything under itself.
  if [ -n "${JAYJAY_PREFIX:-}" ]; then
    prefix="$JAYJAY_PREFIX"
    case "$prefix" in /*) ;; *) prefix="$PWD/$prefix" ;; esac
    data_dir="$prefix/share"
    bin_dir="$prefix/bin"
  else
    data_dir="${XDG_DATA_HOME:-$HOME/.local/share}"
    bin_dir="$HOME/.local/bin"
  fi
  app_dir="$data_dir/jayjay"
  exec_path="$(printf '%s' "$app_dir/JayJay.AppImage" | sed 's/[\\"$`]/\\&/g')"
  machine="$(uname -m)"
  case "$machine" in
    x86_64 | amd64) arch="x86_64" ;;
    aarch64 | arm64) arch="aarch64" ;;
    *) die "JayJay ships Linux builds for x86_64 and aarch64 only; this machine is $machine." ;;
  esac

  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  tag="$(release_tag)"
  if command -v apt-get >/dev/null && command -v dpkg-query >/dev/null; then
    install_deb "$tag"
  else
    install_appimage "$tag"
  fi
}

main "$@"
