# Release recipes — imported by ./justfile.
# Version and signing settings are inherited from the importing justfile.

channel := if version =~ '-beta\.' { "beta" } else { "stable" }

verify-release-base:
  bash "{{root}}/scripts/verify-release.sh" base "{{root}}" '@'

verify-publish-base:
  bash "{{root}}/scripts/verify-release.sh" base "{{root}}" 'v{{version}}'

release: verify-release-base ffi-release help sync-icon
  bash "{{root}}/scripts/macos/release.sh" release "{{version}}" "{{channel}}" "{{build_number}}" "{{signing_identity}}"

# Publish the already-built release: create a public GitHub release, upload the zip, verify it,
# and bump the Homebrew cask.
# Run AFTER pushing the v<version> git tag — `--verify-tag` will fail otherwise.
# Beta versions publish as GitHub pre-releases; update-cask itself skips them.
publish: verify-publish-base create-release upload-release verify-public-release update-cask

# Create or refresh the public GitHub release with SwiftUI and optional GPUI notes.
# Requires the v<version> tag to be pushed first; `--verify-tag` aborts otherwise when creating.
create-release:
  bash "{{root}}/scripts/macos/create-release.sh" "{{version}}" "{{channel}}"

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
  bash "{{root}}/scripts/macos/update-cask.sh" "{{version}}" "{{channel}}" "{{build_number}}"

release-dry-run: verify-release-base ffi-release help sync-icon
  bash "{{root}}/scripts/macos/release.sh" dry-run "{{version}}"
