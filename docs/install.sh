#!/usr/bin/env bash
# Installs JayJay for Linux as a local AppImage or through the signed Arch pacman repo.
# Usage: curl -fsSL https://jayjay.hewig.dev/install.sh | bash
#        JAYJAY_VERSION=0.3.19 bash install.sh   # a specific AppImage release instead of the latest stable
#        curl -fsSL https://jayjay.hewig.dev/install.sh | bash -s -- --pacman-repo   # Arch Linux: signed pacman repo
#        bash install.sh --pacman-repo --no-install   # configure the repo without upgrading or installing
set -euo pipefail

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

case "$(uname -m)" in
  x86_64 | amd64) arch="x86_64" ;;
  aarch64 | arm64) arch="aarch64" ;;
  *) echo "JayJay ships Linux builds for x86_64 and aarch64 only; this machine is $(uname -m)." >&2; exit 1 ;;
esac

if [ "${1:-}" = "--pacman-repo" ]; then
  shift
  no_install=false
  if [ "${1:-}" = "--no-install" ]; then
    no_install=true
    shift
  fi
  [ "$#" -eq 0 ] || { echo "Usage: install.sh --pacman-repo [--no-install]" >&2; exit 1; }
  for tool in pacman pacman-key curl gpg; do
    command -v "$tool" >/dev/null || { echo "$tool is required for the Arch Linux repo setup." >&2; exit 1; }
  done
  elevate=()
  if [ "$(id -u)" -ne 0 ]; then
    elevate=(sudo)
  fi
  if [ -n "${SUDO_USER:-}" ] && [ "$SUDO_USER" != root ]; then
    echo "Run this installer without sudo so it can check your per-user AppImage launchers; it elevates package operations itself." >&2
    exit 1
  fi
  local_artifacts=()
  for launcher in "$bin_dir/jayjay" "$bin_dir/jayjay-gpui"; do
    if [ -L "$launcher" ] && [ "$(readlink "$launcher")" = "$app_dir/JayJay.AppImage" ]; then
      local_artifacts+=("$launcher")
    fi
  done
  desktop_entry="$data_dir/applications/dev.hewig.JayJay.desktop"
  if [ -f "$desktop_entry" ] && grep -Fxq "Exec=\"$exec_path\" %F" "$desktop_entry"; then
    local_artifacts+=("$desktop_entry")
  fi
  if [ "${#local_artifacts[@]}" -ne 0 ]; then
    echo "Existing AppImage launchers would override the pacman package. Remove these launchers, then rerun this command:" >&2
    printf '  rm --' >&2
    printf ' %q' "${local_artifacts[@]}" >&2
    printf '\nThe AppImage itself is left in place.\n' >&2
    exit 1
  fi
  key_fpr="BA65B3D77235F76AAA32AE1EC6D21FA837FDEFEF"
  conf="/etc/pacman.conf"
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  curl -fsSL --retry 3 -o "$tmp/jayjay-packages.asc" https://pkgs.hewig.dev/arch/jayjay-packages.asc
  fingerprint="$(gpg --batch --with-colons --show-keys "$tmp/jayjay-packages.asc" | awk -F: '$1 == "fpr" && !seen { print $10; seen = 1 }')"
  [ "$fingerprint" = "$key_fpr" ] || { echo "Key fingerprint mismatch; refusing to trust the downloaded key." >&2; exit 1; }
  "${elevate[@]}" pacman-key --add "$tmp/jayjay-packages.asc"
  "${elevate[@]}" pacman-key --lsign-key "$key_fpr"
  if grep -q '^\[jayjay\]' "$conf"; then
    echo "[jayjay] is already in $conf; leaving it in place."
  else
    printf '\n[jayjay]\nSigLevel = Required DatabaseOptional\nServer = https://pkgs.hewig.dev/arch/$arch\n' | "${elevate[@]}" tee -a "$conf" >/dev/null
    echo "Added [jayjay] to $conf."
  fi
  if "$no_install"; then
    echo "Repo configured. Run sudo pacman -Syu; if JayJay is not installed, use sudo pacman -Syu jayjay-appimage."
    exit 0
  fi
  packages=()
  # An explicit target would downgrade a newer installed beta to the repo's stable version.
  if ! pacman -Q jayjay-appimage >/dev/null 2>&1; then
    packages=(jayjay-appimage)
  fi
  echo "Updating the system and installing JayJay from the signed pacman repo..."
  # Read confirmation from the terminal even when the installer itself is piped into bash.
  "${elevate[@]}" pacman -Syu "${packages[@]}" </dev/tty
  echo "Repo configured; future stable upgrades arrive with sudo pacman -Syu."
  echo "Betas are not in the repo: install those with pacman -U from the pre-release page."
  exit 0
fi

repo="hewigovens/jayjay"

if command -v pacman >/dev/null && pacman -Q jayjay-appimage >/dev/null 2>&1; then
  echo "JayJay is installed as the jayjay-appimage package; upgrade it with sudo pacman -Syu (stable repo) or pacman -U from the release page (betas)." >&2
  echo "Missing the repo? Run: curl -fsSL https://jayjay.hewig.dev/install.sh | bash -s -- --pacman-repo" >&2
  exit 1
fi
for tool in curl sha256sum; do
  command -v "$tool" >/dev/null || { echo "$tool is required." >&2; exit 1; }
done
if ! command -v fusermount3 >/dev/null && ! command -v fusermount >/dev/null; then
  echo "AppImages need FUSE. Install the fuse3 package (Arch: fuse3, Debian/Ubuntu: fuse3, Fedora: fuse) and run this again." >&2
  exit 1
fi

# Mirrors the app's Settings > CLI rule: only a symlink to a JayJay AppImage may be replaced; anything else is the user's.
for launcher in "$bin_dir/jayjay" "$bin_dir/jayjay-gpui"; do
  if [ -e "$launcher" ] || [ -L "$launcher" ]; then
    target="$(readlink "$launcher" || true)"
    case "$(basename "$target" | tr '[:upper:]' '[:lower:]')" in
      *jayjay*.appimage) ;;
      *) echo "$launcher already exists and is not a JayJay launcher; move it aside and run this again." >&2; exit 1 ;;
    esac
  fi
done

asset="jayjay-gpui-$arch-linux.AppImage"
if [ -n "${JAYJAY_VERSION:-}" ]; then
  tag="v${JAYJAY_VERSION#v}"
else
  tag="$(curl -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/$repo/releases/latest")"
  tag="${tag##*/}"
  case "$tag" in v[0-9]*) ;; *) echo "Could not determine the latest release." >&2; exit 1 ;; esac
fi
base="https://github.com/$repo/releases/download/$tag"

version_file="$app_dir/VERSION"
if [ -z "${JAYJAY_FORCE:-}" ] && [ -x "$app_dir/JayJay.AppImage" ] && [ -L "$bin_dir/jayjay" ] && [ "$(cat "$version_file" 2>/dev/null)" = "$tag" ]; then
  echo "JayJay $tag is already installed at $app_dir/JayJay.AppImage. Set JAYJAY_FORCE=1 to reinstall."
  exit 0
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
echo "Installing JayJay $tag for $arch..."
curl -fsSL --retry 3 -o "$tmp/$asset" "$base/$asset"
curl -fsSL --retry 3 -o "$tmp/$asset.sha256" "$base/$asset.sha256"
(cd "$tmp" && sha256sum -c --quiet "$asset.sha256")
chmod +x "$tmp/$asset"

# The runtime extracts named files without FUSE, so the desktop entry and icons come from the image itself. Older images keep both at the root.
(cd "$tmp" && for pattern in 'usr/share/applications/*' 'usr/share/icons/*' '*.desktop' '*.svg'; do "./$asset" --appimage-extract "$pattern" >/dev/null 2>&1 || true; done)
desktop_entry="$(find "$tmp/squashfs-root" -type f -name 'dev.hewig.JayJay.desktop' | head -1)"
[ -n "$desktop_entry" ] || { echo "The image has no desktop entry." >&2; exit 1; }

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
echo "Run: jayjay /path/to/repo"
echo "Upgrade: run this script again when a new release is out."
case ":$PATH:" in
  *":$bin_dir:"*) ;;
  *) echo "Note: add $bin_dir to your PATH to run jayjay by name." ;;
esac
