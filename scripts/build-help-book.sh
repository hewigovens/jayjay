#!/usr/bin/env bash
set -euo pipefail

root="${1:?repo root required}"
project="$root/shell/mac"
help_source="$project/Resources/JayJayHelpBook"
help_bundle="$root/build/help.noindex/JayJay.help"
help_lproj="$help_bundle/Contents/Resources/English.lproj"
image_src="$root/docs/imgs"
image_dst="$help_lproj/imgs"
help_icon="$help_bundle/Contents/Resources/help-icon.png"
common_css="$root/docs/css/help-common.css"
help_book_css="$root/docs/css/help-book.css"
help_js="$root/docs/js/help.js"
feature_index="$project/Resources/HelpFeatures.json"
app_version="$(awk -F'"' '/^version :=/ { print $2; exit }' "$root/shell/justfile")"
app_version="${app_version%%-beta.*}"
app_build="$(awk -F'"' '/^build_number :=/ { print $2; exit }' "$root/shell/justfile")"

if ! command -v sips >/dev/null 2>&1; then
  echo "sips is required to convert Help Book screenshots to JPEG." >&2
  exit 1
fi

rm -rf "$help_bundle"
mkdir -p "$(dirname "$help_bundle")"
rsync -a --delete "$help_source/" "$help_bundle/"

mkdir -p "$image_dst" "$help_lproj/sty"
# The pages link only sty/help.css, so emit just the concatenation of the two sources.
cat "$common_css" "$help_book_css" > "$help_lproj/sty/help.css"
cp "$help_js" "$help_lproj/sty/help.js"
find "$image_dst" -type f \( -name "*.png" -o -name "*.webp" -o -name "*.jpg" \) -delete
# JPEG, downscaled to 1600px: Tips' WebKit renders JPEG (but not WebP), and JPEG is far smaller than PNG for these screenshots while the Help window is well under the 2560px Retina sources.
help_image_max=1600
help_jpeg_quality=90
for image in "$image_src"/*.webp; do
  name="$(basename "${image%.webp}")"
  sips -s format jpeg -s formatOptions "$help_jpeg_quality" -Z "$help_image_max" "$image" --out "$image_dst/$name.jpg" >/dev/null
done
sips -s format jpeg -s formatOptions "$help_jpeg_quality" -Z "$help_image_max" "$image_src/home.png" --out "$image_dst/home.jpg" >/dev/null
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
# Hash the committed SOURCE images, not the sips-generated JPEGs whose metadata varies per run and would drift CFBundleVersion on every build.
find "$image_src" -type f \( -name "*.webp" -o -name "*.png" \) -print | sort >> "$hash_manifest"
printf '%s\n' "$root/docs/apple-touch-icon.png" >> "$hash_manifest"
help_checksum="$(
  {
    printf '%s\n' "hiutil:corespotlight-anchors-v1"
    printf '%s\n' "help-images:jpeg:max=${help_image_max}:quality=${help_jpeg_quality}"
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
