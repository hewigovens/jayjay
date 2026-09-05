# Release recipes — imported by ./justfile.
# Workspace variables (version, app_name, derived_data, release_dir, signing_identity, ...)
# are inherited from the importing justfile.

channel := if version =~ '-beta\.' { "beta" } else { "stable" }

verify-release-base:
  bash "{{root}}/scripts/verify-release.sh" base "{{root}}" '@'

verify-publish-base:
  bash "{{root}}/scripts/verify-release.sh" base "{{root}}" 'v{{version}}'

release: verify-release-base ffi-release help sync-icon
  #!/usr/bin/env bash
  set -euo pipefail
  signing_identity="${DEVELOPER_ID_APPLICATION:-{{signing_identity}}}"
  notary_profile="${NOTARY_PROFILE:-notarytool}"
  # 0. Abort if any version source drifted from shell/justfile (catches a stale build_number).
  bash "{{root}}/scripts/version.sh" check "{{version}}" "{{build_number}}"
  # 1. Build Release
  app_path="{{derived_data}}/Build/Products/Release/{{app_name}}.app"
  rm -f "$app_path/Contents/MacOS/jayjay-cli"
  xcodegen --spec "{{project}}/project.yml" --project "{{project}}"
  xcodebuild \
    -project "{{project}}/JayJay.xcodeproj" \
    -scheme JayJay \
    -configuration Release \
    -sdk macosx \
    -derivedDataPath "{{derived_data}}" \
    ARCHS=arm64 \
    ONLY_ACTIVE_ARCH=NO \
    build | xcbeautify
  # 1b. Bundle the Rust CLI launcher and trim unused preview resources.
  cargo build --release -p jayjay-cli
  JAYJAY_STRIP_APP=1 bash "{{root}}/scripts/prepare-app-bundle.sh" "$app_path" "{{target_dir}}/release/jayjay"
  # 2. Sign with Developer ID (deep — includes Sparkle framework binaries)
  echo "Signing with ${signing_identity}..."
  find "$app_path" -type f \( -name "*.dylib" -o -perm +111 \) -not -name "*.swift*" | while read binary; do
    codesign --force --timestamp --options runtime --sign "${signing_identity}" "$binary" 2>/dev/null || true
  done
  find "$app_path" \( -name "*.xpc" -o -name "*.app" -o -name "*.framework" -o -name "*.plugin" -o -name "*.docktileplugin" \) | sort -r | while read bundle; do
    codesign --force --timestamp --options runtime --sign "${signing_identity}" "$bundle"
  done
  codesign --force --timestamp --options runtime --sign "${signing_identity}" "$app_path"
  codesign --verify --deep --strict --verbose=2 "$app_path"
  # 3. Notarize
  mkdir -p "{{release_dir}}"
  bash "{{root}}/scripts/archive-release-dsym.sh" "$app_path.dSYM" \
    "{{release_dir}}/{{app_name}}-{{version}}.dSYM.zip"
  bash "{{root}}/scripts/package-release-zip.sh" "$app_path" \
    "{{release_dir}}/{{app_name}}-{{version}}.zip"
  echo "Submitting for notarization with profile ${notary_profile}..."
  xcrun notarytool submit "{{release_dir}}/{{app_name}}-{{version}}.zip" \
    --keychain-profile "${notary_profile}" --wait
  xcrun stapler staple "$app_path"
  xcrun stapler validate "$app_path"
  # 4. Re-zip stapled app + sha256
  bash "{{root}}/scripts/package-release-zip.sh" "$app_path" \
    "{{release_dir}}/{{app_name}}-{{version}}.zip"
  bash "{{root}}/scripts/verify-release.sh" archive \
    "{{release_dir}}/{{app_name}}-{{version}}.zip" notarized

  # 5. Generate Sparkle appcast
  zip_path="{{release_dir}}/{{app_name}}-{{version}}.zip"
  sparkle_bin=$(find "{{derived_data}}" -name "sign_update" -type f 2>/dev/null | head -1)
  if [ -z "$sparkle_bin" ] || [ ! -x "$sparkle_bin" ]; then
    echo "Could not locate Sparkle sign_update under {{derived_data}}." >&2
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
  python3 "{{root}}/scripts/update-appcast.py" "{{version}}" "{{build_number}}" "{{app_name}}" "$zip_path" "{{root}}/docs/appcast.xml" "$sig" "{{channel}}"

  echo
  echo "Local artifacts ready. After committing, push the v{{version}} tag first,"
  echo "run 'just shell::publish' to create a public GitHub release, upload the zip, verify the asset,"
  echo "then push main and the Homebrew tap."

# Publish the already-built release: create a public GitHub release, upload the zip, verify it,
# and bump the Homebrew cask.
# Run AFTER pushing the v<version> git tag — `--verify-tag` will fail otherwise.
# Beta versions publish as GitHub pre-releases; update-cask itself skips them.
publish: verify-publish-base create-release upload-release verify-public-release update-cask

# Refresh the pacman repos from the Arch packages CI attached to the release. Run after the AppImage workflow
# has finished; needs the OrbStack Arch machine from the release guide (`orb create arch arch-pkg`).
publish-arch tag=("v" + version) arches="x86_64 aarch64":
  #!/usr/bin/env bash
  set -euo pipefail
  repo=${GH_REPO:-$(gh repo view --json nameWithOwner --jq .nameWithOwner)}
  work="{{root}}/build/arch-repo"
  rm -rf "$work" && mkdir -p "$work"
  for arch in {{arches}}; do
    if ! gh release download "{{tag}}" -p "jayjay-appimage-*-${arch}.pkg.tar.zst" -D "$work"; then
      echo "no ${arch} package on {{tag}} yet; wait for the AppImage workflow to attach it" >&2
      exit 1
    fi
    package=$(find "$work" -name "jayjay-appimage-*-${arch}.pkg.tar.zst" -print -quit)
    # The token travels over stdin into the machine's environment, never through argv where `ps` could show it.
    gh auth token | orb run -m arch-pkg bash -c 'export GH_TOKEN=$(cat); exec "$@"' publish-arch \
      env GH_REPO="$repo" REPO_TAG="${REPO_TAG:-arch-repo-$arch}" "{{root}}/dist/arch/publish-repo.sh" "$arch" "$package"
  done

# Create or refresh the public GitHub release with SwiftUI and optional GPUI Beta notes.
# Requires the v<version> tag to be pushed first; `--verify-tag` aborts otherwise when creating.
create-release:
  #!/usr/bin/env bash
  set -euo pipefail
  swiftui_notes="{{root}}/releases/{{version}}.html"
  gpui_notes="{{root}}/releases/{{version}}-gpui.md"
  if [ ! -s "$swiftui_notes" ]; then
    echo "Release notes are missing or empty: $swiftui_notes" >&2
    exit 1
  fi
  github_notes="$swiftui_notes"
  if [ -s "$gpui_notes" ]; then
    github_notes=$(mktemp)
    trap 'rm -f "$github_notes"' EXIT
    {
      printf '<h2>macOS</h2>\n\n'
      cat "$swiftui_notes"
      printf '\n\n<h2>Linux GPUI Beta</h2>\n\n'
      cat "$gpui_notes"
    } >"$github_notes"
  fi

  if [ "{{channel}}" = "beta" ]; then prerelease_flag="--prerelease=true"; else prerelease_flag="--prerelease=false"; fi
  if gh release view "v{{version}}" &>/dev/null; then
    gh release edit "v{{version}}" --notes-file "$github_notes" "$prerelease_flag"
    echo "Updated release notes for v{{version}}"
  else
    gh release create "v{{version}}" --verify-tag --title "v{{version}}" --notes-file "$github_notes" "$prerelease_flag"
  fi

# Upload the release zip to the GitHub release.
upload-release:
  #!/usr/bin/env bash
  set -euo pipefail
  zip_path="{{release_dir}}/{{app_name}}-{{version}}.zip"
  if [ ! -f "$zip_path" ]; then
    echo "Zip file not found at $zip_path. Run 'just shell::release' first." >&2
    exit 1
  fi
  echo "Uploading $zip_path to release v{{version}}"
  gh release upload "v{{version}}" "$zip_path" --clobber

# Verify the uploaded asset is publicly reachable at the URL Sparkle downloads.
verify-public-release:
  #!/usr/bin/env bash
  set -euo pipefail
  asset_url="https://github.com/hewigovens/jayjay/releases/download/v{{version}}/{{app_name}}-{{version}}.zip"
  status=$(curl -I -L -s -o /dev/null -w '%{http_code}' "$asset_url")
  if [ "$status" != "200" ]; then
    echo "Release asset is not publicly downloadable ($status): $asset_url" >&2
    exit 1
  fi
  echo "Verified public release asset: $asset_url"

# Update the Homebrew cask at ../tap/Casks/jayjay.rb with the current version and SHA.
update-cask:
  #!/usr/bin/env bash
  set -euo pipefail
  if [ "{{channel}}" = "beta" ]; then
    echo "Beta release: skipping Homebrew cask update."
    exit 0
  fi
  zip_path="{{release_dir}}/{{app_name}}-{{version}}.zip"
  if [ ! -f "$zip_path" ]; then
    echo "Expected archive at $zip_path. Run 'just shell::release' first." >&2
    exit 1
  fi
  cask_path="{{root}}/../tap/Casks/jayjay.rb"
  template="{{root}}/dist/jayjay.rb.template"
  if [ ! -d "$(dirname "$cask_path")" ]; then
    echo "Tap not found at $(dirname "$cask_path") — skipping." >&2
    exit 0
  fi
  sha256=$(shasum -a 256 "$zip_path" | cut -d' ' -f1)
  # Render the whole cask from the in-repo template so its structure is reviewed here, not hand-edited in the tap.
  sed -e "s/__VERSION__/{{version}},{{build_number}}/" \
      -e "s|__SHA256__|$sha256|" \
      "$template" > "$cask_path"
  echo "Rendered $cask_path from template (version {{version}},{{build_number}}, sha256 $sha256)"

release-dry-run: verify-release-base ffi-release help sync-icon
  #!/usr/bin/env bash
  set -euo pipefail
  # 1. Build Release (ad-hoc signed)
  app_path="{{derived_data}}/Build/Products/Release/{{app_name}}.app"
  rm -f "$app_path/Contents/MacOS/jayjay-cli"
  xcodegen --spec "{{project}}/project.yml" --project "{{project}}"
  xcodebuild \
    -project "{{project}}/JayJay.xcodeproj" \
    -scheme JayJay \
    -configuration Release \
    -sdk macosx \
    -derivedDataPath "{{derived_data}}" \
    ARCHS=arm64 \
    ONLY_ACTIVE_ARCH=NO \
    build | xcbeautify
  cargo build --release -p jayjay-cli
  JAYJAY_STRIP_APP=1 bash "{{root}}/scripts/prepare-app-bundle.sh" "$app_path" "{{target_dir}}/release/jayjay"
  # 2. Create release directory
  mkdir -p "{{release_dir}}"
  bash "{{root}}/scripts/archive-release-dsym.sh" "$app_path.dSYM" \
    "{{release_dir}}/{{app_name}}-{{version}}.dSYM.zip"
  rm -rf "{{release_dir}}/{{app_name}}.app"
  cp -R "$app_path" "{{release_dir}}/"
  codesign --force --sign - "{{release_dir}}/{{app_name}}.app/Contents/MacOS/jayjay-cli"
  codesign --force --sign - "{{release_dir}}/{{app_name}}.app"
  # 3. Create zip + sha256 (no notarization)
  bash "{{root}}/scripts/package-release-zip.sh" \
    "{{release_dir}}/{{app_name}}.app" \
    "{{release_dir}}/{{app_name}}-{{version}}.zip"
  bash "{{root}}/scripts/verify-release.sh" archive \
    "{{release_dir}}/{{app_name}}-{{version}}.zip" signed
  shasum -a 256 "{{release_dir}}/{{app_name}}-{{version}}.zip" | cut -d' ' -f1 > "{{release_dir}}/{{app_name}}-{{version}}.zip.sha256"
  echo "Dry run release: {{release_dir}}/{{app_name}}-{{version}}.zip"
  cat "{{release_dir}}/{{app_name}}-{{version}}.zip.sha256"
