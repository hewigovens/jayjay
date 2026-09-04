#!/usr/bin/env bash
# Packs a release jayjay-gpui binary into an AppImage that loads Vulkan, Wayland, and X11 from the host.
set -euo pipefail

usage() {
  echo "usage: $0 <jayjay-gpui binary> <x86_64|aarch64> <output dir>" >&2
  exit 2
}
[[ $# -eq 3 ]] || usage
binary=$1
arch=$2
out=$3

runtime_release=20251108
case "$arch" in
  x86_64) runtime_sha256=2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d ;;
  aarch64) runtime_sha256=00cbdfcf917cc6c0ff6d3347d59e0ca1f7f45a6df1a428a0d6d8a78664d87444 ;;
  *) usage ;;
esac

# GPU and display libraries must be dlopened from the host; linking them pins the bundle to the build machine's driver stack.
if readelf -d "$binary" | grep NEEDED | grep -E 'libvulkan|libwayland|libX11|libGL|libEGL'; then
  echo "error: $binary links a host GPU or display library directly" >&2
  exit 1
fi

root=$(cd "$(dirname "$0")/../.." && pwd)
app_id=dev.hewig.JayJay
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
appdir=$work/AppDir

install -Dm755 "$binary" "$appdir/usr/bin/jayjay-gpui"
install -Dm644 "$root/shell/gpui/linux/$app_id.desktop" "$appdir/usr/share/applications/$app_id.desktop"
install -Dm644 "$root/shell/gpui/linux/$app_id.metainfo.xml" "$appdir/usr/share/metainfo/$app_id.metainfo.xml"
install -Dm644 "$root/docs/icon.svg" "$appdir/usr/share/icons/hicolor/scalable/apps/$app_id.svg"
for size in 64 128 256; do
  install -d "$appdir/usr/share/icons/hicolor/${size}x${size}/apps"
  rsvg-convert --width "$size" --height "$size" \
    --output "$appdir/usr/share/icons/hicolor/${size}x${size}/apps/$app_id.png" "$root/docs/icon.svg"
done
ln -s usr/bin/jayjay-gpui "$appdir/AppRun"
ln -s "usr/share/applications/$app_id.desktop" "$appdir/$app_id.desktop"
ln -s "usr/share/icons/hicolor/scalable/apps/$app_id.svg" "$appdir/$app_id.svg"
ln -s "$app_id.svg" "$appdir/.DirIcon"

runtime=$work/runtime
curl -fsSL --retry 3 -o "$runtime" \
  "https://github.com/AppImage/type2-runtime/releases/download/$runtime_release/runtime-$arch"
echo "$runtime_sha256  $runtime" | sha256sum -c --quiet

mkdir -p "$out"
name=jayjay-gpui-$arch-linux.AppImage
mksquashfs "$appdir" "$work/payload.squashfs" -comp zstd -b 128K -root-owned -noappend -no-progress
cat "$runtime" "$work/payload.squashfs" >"$out/$name"
chmod +x "$out/$name"
(cd "$out" && sha256sum "$name" | tee "$name.sha256")
ls -lh "$out/$name"
