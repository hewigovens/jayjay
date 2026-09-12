#!/usr/bin/env bash
set -euo pipefail

root="${1:?repo root required}"
project="$root/shell/mac"
help_source="$project/Resources/JayJayHelpBook"
help_bundle="$root/build/help.noindex/JayJay.help"
help_lproj="$help_bundle/Contents/Resources/English.lproj"
image_src="$root/docs/imgs"
image_dst="$help_lproj/imgs"
image_cache="$root/build/help.noindex/imgs"
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
# Tips' WebKit needs JPEG; keep small captures at native size and cap larger ones.
help_image_max=1600
help_jpeg_quality=85
# The bundle is rebuilt every run, so the JPEGs live outside it; the stamp retires them when the encode settings change.
image_stamp="$image_cache/.settings"
image_settings="jpeg:no-upscale:max=${help_image_max}:quality=${help_jpeg_quality}"
if [[ "$(cat "$image_stamp" 2>/dev/null || true)" != "$image_settings" ]]; then
  rm -rf "$image_cache"
fi
mkdir -p "$image_cache"
printf '%s\n' "$image_settings" > "$image_stamp"
image_manifest="$(mktemp)"
hash_manifest="$(mktemp)"
trap 'rm -f "$image_manifest" "$hash_manifest"' EXIT
python3 - "$help_lproj" "$image_src" > "$image_manifest" <<'PY'
from html.parser import HTMLParser
from pathlib import Path
import sys

pages, source = map(Path, sys.argv[1:])
names = set()

class Images(HTMLParser):
    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        src = attrs.get("src", "")
        if tag == "img" and src.startswith(("imgs/", "../imgs/")) and src.endswith(".jpg"):
            light = Path(src).stem
            dark = Path(attrs.get("data-dark-src", f"{light}-dark.jpg")).stem
            names.update((light, dark))

for page in pages.rglob("*.html"):
    Images().feed(page.read_text())
for name in sorted(names):
    image = source / ("home.png" if name == "home" else f"{name}.webp")
    if not image.is_file():
        raise FileNotFoundError(image)
    print(image)
PY
while IFS= read -r image; do
  name="$(basename "${image%.*}")"
  encoded="$image_cache/$name.jpg"
  if [[ ! -f "$encoded" || "$image" -nt "$encoded" ]]; then
    largest_dimension="$(sips -g pixelWidth -g pixelHeight "$image" | awk '/pixelWidth:|pixelHeight:/ { if ($2 > max) max = $2 } END { print max }')"
    image_options=(-s format jpeg -s formatOptions "$help_jpeg_quality")
    if (( largest_dimension > help_image_max )); then
      image_options+=(-Z "$help_image_max")
    fi
    sips "${image_options[@]}" "$image" --out "$encoded" >/dev/null
  fi
  cp "$encoded" "$image_dst/$name.jpg"
done < "$image_manifest"
cp "$root/docs/apple-touch-icon.png" "$help_icon"
find "$image_dst" -name ".DS_Store" -delete
xattr -cr "$help_bundle" 2>/dev/null || true

find "$help_lproj" -type f \( \
  -name "*.html" -o \
  -name "*.css" -o \
  -name "*.js" -o \
  -name "*.plist" \
\) ! -name "JayJay.helpindex" -print | sort > "$hash_manifest"
# Hash the committed SOURCE images, not the sips-generated JPEGs whose metadata varies per run and would drift CFBundleVersion on every build.
cat "$image_manifest" >> "$hash_manifest"
printf '%s\n' "$root/docs/apple-touch-icon.png" >> "$hash_manifest"
help_checksum="$(
  {
    printf '%s\n' "hiutil:corespotlight-anchors-v1"
    printf '%s\n' "help-images:$image_settings"
    xargs shasum -a 256 < "$hash_manifest"
  } | shasum -a 256 | cksum | awk '{ print $1 }'
)"

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
