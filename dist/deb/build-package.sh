#!/usr/bin/env bash
# Build a Debian package from the release AppImage without requiring FUSE at runtime.
set -euo pipefail

usage() {
  echo "usage: $0 <AppImage> <amd64|arm64> <output dir> [tag]" >&2
  exit 2
}
[[ $# -ge 3 && $# -le 4 ]] || usage
appimage=$(realpath "$1")
arch=$2
out=$(realpath -m "$3")
[[ $arch == amd64 || $arch == arm64 ]] || usage
root=$(cd "$(dirname "$0")/../.." && pwd)
tag=${4:-v$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -1)}
[[ $tag =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$ ]] || { echo "unsupported release tag: $tag" >&2; exit 1; }
version=${tag#v}
version=${version/-beta./~beta.}-1

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cd "$work"
install -m755 "$appimage" "$work/input.AppImage"
"$work/input.AppImage" --appimage-extract >/dev/null
appdir=$work/squashfs-root

package=$work/package
app_id=dev.hewig.JayJay
install -Dm755 "$appdir/AppRun" "$package/opt/jayjay/AppRun"
install -Dm755 "$appdir/usr/bin/jayjay-gpui" "$package/opt/jayjay/usr/bin/jayjay-gpui"
install -d "$package/opt/jayjay/usr/lib"
cp -a "$appdir/usr/lib/." "$package/opt/jayjay/usr/lib/"
if [[ -d "$appdir/usr/share/jayjay" ]]; then
  install -d "$package/opt/jayjay/usr/share"
  cp -a "$appdir/usr/share/jayjay" "$package/opt/jayjay/usr/share/"
fi
install -d "$package/usr/bin"
ln -s /opt/jayjay/AppRun "$package/usr/bin/jayjay"
ln -s /opt/jayjay/AppRun "$package/usr/bin/jayjay-gpui"
install -Dm644 "$appdir/usr/share/applications/$app_id.desktop" "$package/usr/share/applications/$app_id.desktop"
install -Dm644 "$appdir/usr/share/metainfo/$app_id.metainfo.xml" "$package/usr/share/metainfo/$app_id.metainfo.xml"
install -d "$package/usr/share/icons"
cp -a "$appdir/usr/share/icons/." "$package/usr/share/icons/"
install -Dm644 "$root/LICENSE" "$package/usr/share/doc/jayjay/copyright"

install -d "$package/DEBIAN"
cat >"$package/DEBIAN/control" <<EOF
Package: jayjay
Version: $version
Architecture: $arch
Maintainer: JayJay <packages@hewig.dev>
Section: devel
Priority: optional
Homepage: https://jayjay.hewig.dev/
Installed-Size: $(du -sk "$package" | cut -f1)
Depends: libc6 (>= 2.31), libgcc-s1, libstdc++6, libfontconfig1, libxcb1, libx11-6, libx11-xcb1, libwayland-client0, libwayland-cursor0, libvulkan1, mesa-vulkan-drivers | vulkan-icd, xkb-data, xdg-utils
Description: Native desktop client for Jujutsu version control
 JayJay provides a graphical interface for working with Jujutsu repositories.
EOF
(cd "$package" && find opt usr -type f -print0 | LC_ALL=C sort -z | xargs -0 md5sum >DEBIAN/md5sums)
mkdir -p "$out"
# GitHub publishes ~ as . in asset names; match it so the checksum and the rerun dedupe name the published file.
name=jayjay_${version//\~/.}_${arch}.deb
dpkg-deb --root-owner-group -Zxz --build "$package" "$out/$name"
(cd "$out" && sha256sum "$name" >"$name.sha256")
echo "built $out/$name"
