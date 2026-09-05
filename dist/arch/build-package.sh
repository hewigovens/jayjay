#!/usr/bin/env bash
# Build jayjay-appimage from an AppImage already placed in dist/arch, then install and remove it as a check.
# Usage: build-package.sh <arch> [tag]   (no tag: version from the workspace Cargo.toml)
set -euo pipefail

arch=$1
tag=${2:-}
cd "$(dirname "$0")"

if [ -z "$tag" ]; then
  tag="v$(sed -n 's/^version = "\(.*\)"/\1/p' ../../Cargo.toml | head -1)"
fi
if ! [[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$ ]]; then
  echo "unsupported release tag: ${tag}" >&2
  exit 1
fi
# pkgver forbids hyphens; pacman orders 0.3.17beta.2 before 0.3.17.
pkgver=${tag#v}
pkgver=${pkgver//-/}
cp ../../LICENSE "LICENSE-${pkgver}"
sed -i \
  -e "s/^pkgver=.*/pkgver=${pkgver}/" \
  -e "s/^_tag=.*/_tag=\"${tag}\"/" \
  -e "s/^sha256sums=.*/sha256sums=('$(sha256sum "LICENSE-${pkgver}" | cut -d' ' -f1)')/" \
  -e "s/^sha256sums_${arch}=.*/sha256sums_${arch}=('$(sha256sum "jayjay-gpui-${arch}-linux.AppImage" | cut -d' ' -f1)')/" \
  PKGBUILD

useradd --create-home builder
chown -R builder:builder .
# Arch Linux ARM defaults PKGEXT to .pkg.tar.xz.
runuser -u builder -- env PKGEXT=.pkg.tar.zst makepkg --noconfirm
package=$(find . -maxdepth 1 -name "jayjay-appimage-*-${arch}.pkg.tar.zst" -print -quit)
test -n "$package"
pacman -U --noconfirm "$package"
test -L /usr/bin/jayjay
pacman -R --noconfirm jayjay-appimage
echo "package=${package#./}" >>"${GITHUB_ENV:-/dev/null}"
echo "built ${package#./}"
