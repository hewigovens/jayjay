#!/usr/bin/env bash
# Build the committed PKGBUILD against the published release assets, then install, upgrade, and remove the package.
set -euo pipefail

cd "$(dirname "$0")"
useradd --create-home builder
chown -R builder:builder .
runuser -u builder -- makepkg --printsrcinfo >/tmp/generated.SRCINFO
diff -u .SRCINFO /tmp/generated.SRCINFO
runuser -u builder -- makepkg --noconfirm
package=$(find . -maxdepth 1 -name "jayjay-appimage-*.pkg.tar.zst" -print -quit)
pacman -U --noconfirm "$package"
test -L /usr/bin/jayjay-gpui
test -L /usr/bin/jayjay
test -f /usr/share/applications/dev.hewig.JayJay.desktop
test -f /usr/share/metainfo/dev.hewig.JayJay.metainfo.xml
test -f /usr/share/icons/hicolor/scalable/apps/dev.hewig.JayJay.svg
test -f /usr/share/licenses/jayjay-appimage/LICENSE
desktop-file-validate /usr/share/applications/dev.hewig.JayJay.desktop
appstreamcli validate --no-net /usr/share/metainfo/dev.hewig.JayJay.metainfo.xml
sed -i "s/^pkgrel=1$/pkgrel=2/" PKGBUILD
runuser -u builder -- makepkg --noconfirm --force
upgrade=$(find . -maxdepth 1 -name "jayjay-appimage-*-2-*.pkg.tar.zst" -print -quit)
pacman -U --noconfirm "$upgrade"
pacman -Q jayjay-appimage | grep -q -- "-2$"
pacman -R --noconfirm jayjay-appimage
for path in /usr/bin/jayjay-gpui /usr/bin/jayjay /opt/jayjay-appimage/JayJay.AppImage \
  /usr/share/applications/dev.hewig.JayJay.desktop /usr/share/metainfo/dev.hewig.JayJay.metainfo.xml \
  /usr/share/icons/hicolor/scalable/apps/dev.hewig.JayJay.svg /usr/share/licenses/jayjay-appimage/LICENSE; do
  test ! -e "$path"
done
