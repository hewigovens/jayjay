#!/usr/bin/env bash
# Packs the SSH helpers CI built for main@origin as gzipped release assets; the app bundles only manifest.json and downloads a host's helper on first connect.
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

version="$1"
repo=hewigovens/jayjay
targets=(aarch64-apple-darwin x86_64-apple-darwin aarch64-unknown-linux-musl x86_64-unknown-linux-musl aarch64-pc-windows-msvc x86_64-pc-windows-msvc)
staged="$root/build/ssh-helpers"
assets="$release_dir/ssh-helpers"
commit=$(jj --ignore-working-copy -R "$root" log --no-graph -r 'main@origin' -T 'commit_id')

find_run() {
  gh run list -R "$repo" --workflow ssh-helpers.yml --branch main --commit "$commit" --limit 20 --json databaseId,status,conclusion \
    --jq '[.[] | select(.status != "completed" or .conclusion == "success")][0].databaseId // empty'
}
run=$(find_run)
if [[ -z "$run" ]]; then
  echo "Starting the SSH helpers workflow for $commit"
  gh workflow run ssh-helpers.yml -R "$repo" --ref main
  for _ in {1..24}; do
    sleep 5
    run=$(find_run)
    [[ -n "$run" ]] && break
  done
  [[ -n "$run" ]] || { echo "No SSH helpers run started for $commit; is main@origin still the head of main?" >&2; exit 1; }
fi
gh run watch "$run" -R "$repo" --exit-status --compact -i 30 >/dev/null || { echo "SSH helpers run $run failed: gh run view $run -R $repo" >&2; exit 1; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
gh run download "$run" -R "$repo" --dir "$work/artifacts"
rm -rf "$staged" "$assets"
mkdir -p "$staged" "$assets" "$work/helpers"

entries=()
for target in "${targets[@]}"; do
  tar -xzf "$work/artifacts/ssh-helper-$target/ssh-helper-$target.tgz" -C "$work/helpers"
  executable="$work/helpers/$target/jayjay-ssh"
  [[ "$target" == *-windows-msvc ]] && executable+=.exe
  # A link in the artifact would publish whatever local file it points at.
  if [[ -n "$(find "$work/helpers" -type l)" || ! -f "$executable" ]]; then
    echo "The $target artifact must contain only the helper as a regular file." >&2
    exit 1
  fi
  asset="jayjay-ssh-$target.gz"
  gzip -9 -n -c "$executable" > "$assets/$asset"
  sha256=$(shasum -a 256 "$executable" | cut -d' ' -f1)
  entries+=("\"$target\":{\"url\":\"https://github.com/$repo/releases/download/v$version/$asset\",\"sha256\":\"$sha256\"}")
done

(IFS=,; printf '{%s}\n' "${entries[*]}") > "$staged/manifest.json"
cp "$staged/manifest.json" "$assets/ssh-helpers.json"
