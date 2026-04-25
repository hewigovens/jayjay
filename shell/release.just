# Release recipes — imported by ./justfile.
# Workspace variables (version, app_name, derived_data, release_dir, signing_identity, ...)
# are inherited from the importing justfile.

release: ffi-release
  #!/usr/bin/env bash
  set -euo pipefail
  signing_identity="${DEVELOPER_ID_APPLICATION:-{{signing_identity}}}"
  notary_profile="${NOTARY_PROFILE:-notarytool}"
  # 1. Build Release
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
  # 1b. Bundle CLI binary
  app_path="{{derived_data}}/Build/Products/Release/{{app_name}}.app"
  cargo build -p jayjay-cli --release
  cp "{{target_dir}}/release/jayjay" "$app_path/Contents/MacOS/jayjay-cli"
  # 2. Sign with Developer ID (deep — includes Sparkle framework binaries)
  echo "Signing with ${signing_identity}..."
  find "$app_path" -type f \( -name "*.dylib" -o -perm +111 \) -not -name "*.swift*" | while read binary; do
    codesign --force --timestamp --options runtime --sign "${signing_identity}" "$binary" 2>/dev/null || true
  done
  find "$app_path" -name "*.xpc" -o -name "*.app" -o -name "*.framework" | sort -r | while read bundle; do
    codesign --force --timestamp --options runtime --sign "${signing_identity}" "$bundle"
  done
  codesign --force --timestamp --options runtime --sign "${signing_identity}" "$app_path"
  codesign --verify --deep --strict --verbose=2 "$app_path"
  # 3. Notarize
  mkdir -p "{{release_dir}}"
  rm -f "{{release_dir}}/{{app_name}}-{{version}}.zip"
  ditto -c -k --keepParent "{{derived_data}}/Build/Products/Release/{{app_name}}.app" \
    "{{release_dir}}/{{app_name}}-{{version}}.zip"
  echo "Submitting for notarization with profile ${notary_profile}..."
  xcrun notarytool submit "{{release_dir}}/{{app_name}}-{{version}}.zip" \
    --keychain-profile "${notary_profile}" --wait
  xcrun stapler staple "{{derived_data}}/Build/Products/Release/{{app_name}}.app"
  xcrun stapler validate "{{derived_data}}/Build/Products/Release/{{app_name}}.app"
  # 4. Re-zip stapled app + sha256
  rm -f "{{release_dir}}/{{app_name}}-{{version}}.zip"
  ditto -c -k --keepParent "{{derived_data}}/Build/Products/Release/{{app_name}}.app" \
    "{{release_dir}}/{{app_name}}-{{version}}.zip"
  echo "Created notarized {{release_dir}}/{{app_name}}-{{version}}.zip"
  shasum -a 256 "{{release_dir}}/{{app_name}}-{{version}}.zip"

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
  python3 "{{root}}/scripts/update-appcast.py" "{{version}}" "{{build_number}}" "{{app_name}}" "$zip_path" "{{root}}/docs/appcast.xml" "$sig"

  echo
  echo "Local artifacts ready. After committing, pushing main, and pushing the v{{version}} tag,"
  echo "run 'just shell::publish' to create the GitHub release, upload the zip, and update the Homebrew cask."

# Publish the already-built release: create a GitHub draft, upload the zip, and bump the Homebrew cask.
# Run AFTER pushing the v<version> git tag — `--verify-tag` will fail otherwise.
publish:
  just --justfile "{{justfile()}}" create-release
  just --justfile "{{justfile()}}" upload-release
  just --justfile "{{justfile()}}" update-cask

# Create a GitHub draft release for the current version if it doesn't exist.
# Requires the v<version> tag to be pushed first; `--verify-tag` aborts otherwise.
create-release:
  #!/usr/bin/env bash
  set -euo pipefail
  notes_path="{{root}}/releases/{{version}}.html"
  if gh release view "v{{version}}" &>/dev/null; then
    echo "Release v{{version}} already exists"
    exit 0
  fi
  if [ -f "$notes_path" ]; then
    gh release create "v{{version}}" --draft --verify-tag --title "v{{version}}" --notes-file "$notes_path"
  else
    echo "Warning: no release notes found at $notes_path; creating draft with fallback notes." >&2
    gh release create "v{{version}}" --draft --verify-tag --title "v{{version}}" --notes "Release {{version}}"
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

# Update the Homebrew cask at ../tap/Casks/jayjay.rb with the current version and SHA.
update-cask:
  #!/usr/bin/env bash
  set -euo pipefail
  zip_path="{{release_dir}}/{{app_name}}-{{version}}.zip"
  if [ ! -f "$zip_path" ]; then
    echo "Expected archive at $zip_path. Run 'just shell::release' first." >&2
    exit 1
  fi
  cask_path="{{root}}/../tap/Casks/jayjay.rb"
  if [ ! -f "$cask_path" ]; then
    echo "Cask not found at $cask_path — skipping." >&2
    exit 0
  fi
  sha256=$(shasum -a 256 "$zip_path" | cut -d' ' -f1)
  sed -i '' \
    -e 's/version "[^"]*"/version "{{version}},{{build_number}}"/' \
    -e "s/sha256 \"[^\"]*\"/sha256 \"$sha256\"/" \
    "$cask_path"
  echo "Updated $cask_path with version {{version}},{{build_number}} sha256 $sha256"

release-dry-run: ffi-release
  # 1. Build Release (ad-hoc signed)
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
  # 2. Create release directory
  mkdir -p "{{release_dir}}"
  rm -rf "{{release_dir}}/{{app_name}}.app"
  cp -R "{{derived_data}}/Build/Products/Release/{{app_name}}.app" "{{release_dir}}/"
  codesign --force --sign - "{{release_dir}}/{{app_name}}.app"
  # 3. Create zip + sha256 (no notarization)
  cd "{{release_dir}}" && ditto -c -k --keepParent "{{app_name}}.app" "{{app_name}}-{{version}}.zip"
  shasum -a 256 "{{release_dir}}/{{app_name}}-{{version}}.zip" | cut -d' ' -f1 > "{{release_dir}}/{{app_name}}-{{version}}.zip.sha256"
  @echo "Dry run release: {{release_dir}}/{{app_name}}-{{version}}.zip"
  @cat "{{release_dir}}/{{app_name}}-{{version}}.zip.sha256"
