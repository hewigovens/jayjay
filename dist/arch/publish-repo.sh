#!/usr/bin/env bash
# Publish one architecture's package to the rolling pacman repo, hosted as assets of the `arch-repo-<arch>` release.
# Runs from the release pipeline on an Arch machine (`just shell::publish-arch`), one release at a time.
# Every asset goes up as `<name>.new` and is renamed into place, so a failed upload never removes a file pacman
# clients can still fetch; jayjay.db is replaced last because it is the file pacman reads.
# Usage: GH_REPO=owner/repo publish-repo.sh <arch> <package.pkg.tar.zst>   (REPO_TAG overrides the release tag;
# PUBLISH_REPO_FAIL_AT=<asset name> aborts before that asset's rename, to exercise the rollback)
set -Eeuo pipefail

arch=$1
package=$(realpath "$2")
repo_tag=${REPO_TAG:-arch-repo-$arch}
: "${GH_REPO:?set GH_REPO to owner/repo}"

work=$(mktemp -d)
cd "$work"
cp "$package" .
package=$(basename "$package")

if ! gh release view "$repo_tag" >/dev/null 2>&1; then
  gh release create "$repo_tag" --title "Arch package repo (${arch})" --prerelease \
    --notes "pacman repository for JayJay. Add to /etc/pacman.conf: [jayjay] / SigLevel = Optional TrustAll / Server = https://github.com/${GH_REPO}/releases/download/arch-repo-\$arch"
fi

# The release's own API URL also resolves drafts, which the tags endpoint does not.
release_api=$(gh release view "$repo_tag" --json apiUrl --jq .apiUrl)
asset_id() {
  gh api "${release_api}/assets?per_page=100" --jq ".[] | select(.name==\"$1\") | .id" | head -1
}
rename_asset() { gh api -X PATCH "repos/${GH_REPO}/releases/assets/$1" -f name="$2" >/dev/null; }
delete_asset() { gh api -X DELETE "repos/${GH_REPO}/releases/assets/$1" >/dev/null; }

# An existing index must download; only a repo without one starts fresh, so a transient failure cannot bypass the version guard below.
if [ -n "$(asset_id jayjay.db.tar.zst)" ]; then
  gh release download "$repo_tag" -p 'jayjay.db.tar.zst' -p 'jayjay.files.tar.zst'
fi

# Republishing an older release must not downgrade the index.
new_version=$(tar -xOf "$package" .PKGINFO | sed -n 's/^pkgver = //p')
current=$(tar -tf jayjay.db.tar.zst 2>/dev/null | sed -n 's|^jayjay-appimage-\(.*\)/desc$|\1|p' | head -1 || true)
if [ -n "$current" ] && [ "$(vercmp "$new_version" "$current")" -lt 0 ]; then
  echo "repo already serves jayjay-appimage ${current}; not downgrading to ${new_version}"
  exit 0
fi

repo-add jayjay.db.tar.zst "$package"
# repo-add leaves the un-suffixed names as symlinks; pacman fetches those names, so upload real files.
rm -f jayjay.db jayjay.files
cp jayjay.db.tar.zst jayjay.db
cp jayjay.files.tar.zst jayjay.files

# Two-phase swap: every previous asset stays as <name>.prev until the last index is live, then all are deleted;
# any failure before that restores the swapped assets in reverse order, so the live jayjay.db always matches
# the package bytes it names.
swapped=()
rollback() {
  local name id
  for ((i = ${#swapped[@]} - 1; i >= 0; i--)); do
    name=${swapped[i]}
    for id in $(asset_id "$name") $(asset_id "${name}.new"); do delete_asset "$id"; done
    id=$(asset_id "${name}.prev")
    [ -z "$id" ] || rename_asset "$id" "$name"
  done
  echo "publish failed; previous assets restored" >&2
}
trap rollback ERR

swap_asset() {
  local name=$1 stale previous
  for stale in "${name}.new" "${name}.prev"; do
    stale=$(asset_id "$stale")
    [ -z "$stale" ] || delete_asset "$stale"
  done
  cp "$name" "${name}.new"
  gh release upload "$repo_tag" "${name}.new"
  previous=$(asset_id "$name")
  if [ -n "$previous" ]; then
    rename_asset "$previous" "${name}.prev"
    swapped+=("$name")
  fi
  [ "${PUBLISH_REPO_FAIL_AT:-}" != "$name" ]
  rename_asset "$(asset_id "${name}.new")" "$name"
}

# The package before the indexes that reference it; jayjay.db last.
for name in "$package" jayjay.files.tar.zst jayjay.files jayjay.db.tar.zst jayjay.db; do
  swap_asset "$name"
done
trap - ERR
for name in "${swapped[@]}"; do
  delete_asset "$(asset_id "${name}.prev")"
done
echo "published jayjay-appimage ${new_version} to ${repo_tag}"
