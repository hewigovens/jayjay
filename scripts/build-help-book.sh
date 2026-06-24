#!/usr/bin/env bash
set -euo pipefail

root="${1:?repo root required}"
project="$root/shell/mac"
help_bundle="$project/Resources/JayJay.help"
help_lproj="$help_bundle/Contents/Resources/English.lproj"
image_src="$root/docs/imgs"
image_dst="$help_lproj/imgs"
help_icon="$help_bundle/Contents/Resources/help-icon.png"
common_css="$root/docs/help-common.css"
help_book_css="$root/docs/help-book.css"
help_js="$root/docs/help.js"
feature_index="$project/Resources/HelpFeatures.json"
app_version="$(awk -F'"' '/^version :=/ { print $2; exit }' "$root/shell/justfile")"
app_build="$(awk -F'"' '/^build_number :=/ { print $2; exit }' "$root/shell/justfile")"

if ! command -v sips >/dev/null 2>&1; then
  echo "sips is required to convert Help Book screenshots to PNG." >&2
  exit 1
fi

mkdir -p "$image_dst"
cp "$common_css" "$help_lproj/sty/help-common.css"
cp "$help_book_css" "$help_lproj/sty/help-book.css"
cat "$common_css" "$help_book_css" > "$help_lproj/sty/help.css"
cp "$help_js" "$help_lproj/sty/help.js"
find "$image_dst" -type f \( -name "*.png" -o -name "*.webp" \) -delete
# Downscale to a 1600px max edge: source screenshots are 2560px Retina, far larger than the Help Viewer window needs, and full-size PNGs balloon the bundle ~5x.
help_image_max=1600
for image in "$image_src"/*.webp; do
  name="$(basename "${image%.webp}")"
  sips -s format png -Z "$help_image_max" "$image" --out "$image_dst/$name.png" >/dev/null
done
sips -Z "$help_image_max" "$image_src/home.png" --out "$image_dst/home.png" >/dev/null
cp "$root/docs/apple-touch-icon.png" "$help_icon"
find "$image_dst" -name ".DS_Store" -delete
xattr -cr "$help_bundle" 2>/dev/null || true

hash_manifest="$(mktemp)"
find "$help_lproj" -type f \( \
  -name "*.html" -o \
  -name "*.css" -o \
  -name "*.js" -o \
  -name "*.plist" \
\) ! -name "JayJay.helpindex" -print | sort > "$hash_manifest"
# Hash the committed SOURCE images, not the sips-generated PNGs whose metadata varies per run and would drift CFBundleVersion on every build.
find "$image_src" -type f \( -name "*.webp" -o -name "*.png" \) -print | sort >> "$hash_manifest"
printf '%s\n' "$root/docs/apple-touch-icon.png" >> "$hash_manifest"
help_checksum="$(
  {
    printf '%s\n' "hiutil:corespotlight-anchors-v1"
    xargs shasum -a 256 < "$hash_manifest"
  } | shasum -a 256 | cksum | awk '{ print $1 }'
)"
rm -f "$hash_manifest"

plutil -replace CFBundleShortVersionString -string "$app_version" "$help_bundle/Contents/Info.plist"
plutil -replace CFBundleVersion -string "$app_build.$help_checksum" "$help_bundle/Contents/Info.plist"

plutil -lint \
  "$project/Info.plist" \
  "$help_bundle/Contents/Info.plist" \
  "$help_lproj/ExactMatch.plist" >/dev/null

if command -v jq >/dev/null 2>&1; then
  jq empty "$feature_index"
else
  python3 -m json.tool "$feature_index" >/dev/null
fi

rm -f "$help_lproj/JayJay.helpindex"
hiutil -I corespotlight -Caf "$help_lproj/JayJay.helpindex" "$help_lproj"
