#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

mode="$1"
version="$2"
case "$mode" in
  release)
    channel="$3"
    build_number="$4"
    signing_identity="${DEVELOPER_ID_APPLICATION:-$5}"
    notary_profile="${NOTARY_PROFILE:-notarytool}"
    bash "$root/scripts/version.sh" check "$version" "$build_number"
    ;;
  dry-run) ;;
  *) echo "usage: $0 {release <version> <channel> <build> <signing-identity>|dry-run <version>}" >&2; exit 2 ;;
esac

app_path="$derived_data/Build/Products/Release/$app_name.app"
zip_path="$release_dir/$app_name-$version.zip"
bash "$root/scripts/macos/build.sh" Release
if [[ "$mode" == release ]]; then
  echo "Signing with ${signing_identity}..."
  find "$app_path" -type f \( -name "*.dylib" -o -perm +111 \) -not -name "*.swift*" | while IFS= read -r binary; do
    codesign --force --timestamp --options runtime --sign "${signing_identity}" "$binary" 2>/dev/null || true
  done
  find "$app_path" \( -name "*.xpc" -o -name "*.app" -o -name "*.framework" -o -name "*.plugin" -o -name "*.docktileplugin" \) | sort -r | while IFS= read -r bundle; do
    codesign --force --timestamp --options runtime --sign "${signing_identity}" "$bundle"
  done
  codesign --force --timestamp --options runtime --sign "${signing_identity}" "$app_path"
  codesign --verify --deep --strict --verbose=2 "$app_path"
fi
mkdir -p "$release_dir"
bash "$root/scripts/archive-release-dsym.sh" "$app_path.dSYM" "$release_dir/$app_name-$version.dSYM.zip"
if [[ "$mode" == dry-run ]]; then
  rm -rf "${release_dir}/${app_name}.app"
  cp -R "$app_path" "${release_dir}/"
  codesign --force --sign - "${release_dir}/${app_name}.app/Contents/MacOS/jayjay-cli"
  codesign --force --sign - "${release_dir}/${app_name}.app"
  app_path="$release_dir/$app_name.app"
fi
bash "$root/scripts/package-release-zip.sh" "$app_path" "$zip_path"
if [[ "$mode" == dry-run ]]; then
  bash "${root}/scripts/verify-release.sh" archive \
    "$zip_path" signed
  shasum -a 256 "$zip_path" | cut -d' ' -f1 > "$zip_path.sha256"
  echo "Dry run release: $zip_path"
  cat "$zip_path.sha256"
  exit 0
fi

echo "Submitting for notarization with profile ${notary_profile}..."
xcrun notarytool submit "$zip_path" \
  --keychain-profile "${notary_profile}" --wait
xcrun stapler staple "$app_path"
xcrun stapler validate "$app_path"
bash "${root}/scripts/package-release-zip.sh" "$app_path" \
  "$zip_path"
bash "${root}/scripts/verify-release.sh" archive \
  "$zip_path" notarized
shasum -a 256 "$zip_path" | cut -d' ' -f1 > "$zip_path.sha256"

sparkle_bin=$(find "${derived_data}" -name "sign_update" -type f 2>/dev/null | head -1)
if [ -z "$sparkle_bin" ] || [ ! -x "$sparkle_bin" ]; then
  echo "Could not locate Sparkle sign_update under ${derived_data}." >&2
  exit 1
fi
sign_output=$("$sparkle_bin" "$zip_path" 2>&1) || {
  echo "Error: Sparkle sign_update failed: $sign_output" >&2
  exit 1
}
sig=$(printf '%s' "$sign_output" | grep -o 'sparkle:edSignature="[^"]*"' | cut -d'"' -f2 || true)
if [ -z "$sig" ]; then
  echo "Error: no Sparkle edSignature in sign_update output: $sign_output" >&2
  exit 1
fi
python3 "${root}/scripts/update-appcast.py" "${version}" "${build_number}" "${app_name}" "$zip_path" "${root}/docs/appcast.xml" "$sig" "${channel}"

echo
echo "Local artifacts ready. After committing, push the v${version} tag first,"
echo "run 'just shell::publish' to create a public GitHub release, upload the zip, verify the asset,"
echo "then push main and the Homebrew tap."
