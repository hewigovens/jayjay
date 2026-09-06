#!/usr/bin/env bash
# Add a signed package to the append-only pacman repo at an rclone path and rebuild its signed index from every package there.
# Packages are never overwritten and go up before the index, so a partial run leaves the previous index valid.
# Usage: GPGKEY=<fingerprint> publish-repo.sh <package.pkg.tar.zst> <rclone-path>   e.g. r2:jayjay-packages/arch/x86_64
set -euo pipefail
package=$(realpath "$1")
remote=$2
work=$(mktemp -d)
cd "$work"
name=$(basename "$package")
rclone copyto --ignore-existing "$package" "${remote}/${name}"
rclone copyto --ignore-existing "$package.sig" "${remote}/${name}.sig"
rclone copy "$remote" . --include '*.pkg.tar.zst' --include '*.pkg.tar.zst.sig'
# Ascending order so the newest package wins the index entry.
# shellcheck disable=SC2046
repo-add --sign --key "$GPGKEY" jayjay.db.tar.zst $(ls jayjay-appimage-*.pkg.tar.zst | sort -V)
rm -f jayjay.db jayjay.db.sig jayjay.files jayjay.files.sig
cp jayjay.db.tar.zst jayjay.db
cp jayjay.db.tar.zst.sig jayjay.db.sig
cp jayjay.files.tar.zst jayjay.files
cp jayjay.files.tar.zst.sig jayjay.files.sig
for index in jayjay.files.tar.zst jayjay.files jayjay.db.tar.zst jayjay.db; do
  rclone copyto "$index.sig" "${remote}/${index}.sig"
  rclone copyto "$index" "${remote}/${index}"
done
echo "index now serves $(tar -tf jayjay.db.tar.zst | sed -n 's|^\(.*\)/desc$|\1|p')"
