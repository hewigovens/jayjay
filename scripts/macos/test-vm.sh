#!/usr/bin/env bash
# Runs a ui-test-build output in a clone of a VMPal macOS VM, away from the host's desktop and lock screen; the clone outlives the run and is deleted after idle_seconds without one.
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

bundle_id="${1:?usage: test-vm.sh <bundle-id> [test-ids...]}"
shift
vm="${JAYJAY_UI_TEST_VM:-JayJay UI}"
products="${derived_data#"$root"/}/Build/Products"
out="$root/build/ui-test-vm"
clone="$vm run"
stamp="$HOME/Library/Caches/jayjay-ui-test-vm-${clone// /-}.stamp"
idle_seconds=900

command -v vmpal >/dev/null || { echo "vmpal not found; link VMPal's command line onto PATH (see agents/testing.md)." >&2; exit 1; }

cd "$root"
xctestrun="$(ls -t "$products"/JayJayUITests_*.xctestrun | head -1)"
rm -rf "$out"
mkdir -p "$out"
# The bundles travel as one tarball: VMPal copies through a shared folder, which breaks the symlinks inside frameworks.
tar -cf "$out/payload.tar" \
  --exclude='*.dSYM' --exclude='*.o' --exclude='*.a' --exclude='*.swiftmodule' --exclude=include \
  "$products/Debug" "$xctestrun" scripts/ui-test-fixtures.sh tests/fixtures/formats CONTRIBUTING.md

guest_script='
set -euo pipefail
export PATH="/usr/local/bin:$PATH"
bundle_id="$1"
xctestrun="$2"
shift 2
rm -rf jayjay
mkdir jayjay
cd jayjay
tar -xf ../jayjay-payload.tar
bash scripts/ui-test-fixtures.sh "$bundle_id" >fixtures.log 2>&1 || { cat fixtures.log >&2; exit 1; }
args=()
for test_id in "$@"; do
  args+=(-only-testing:"$test_id")
done
args+=(test-without-building)
xcodebuild -xctestrun "$xctestrun" -destination "platform=macOS" \
  -derivedDataPath build/DerivedData -resultBundlePath ui.xcresult "${args[@]}"
'

# Every run rewrites the stamp, so only the reaper of the latest run finds its own token and deletes the clone.
claim_clone() {
  token="$$ $(date +%s)"
  echo "$token" > "$stamp"
}

reap_clone_when_idle() {
  claim_clone
  nohup bash -c 'sleep "$1"; [[ "$(cat "$2" 2>/dev/null)" == "$3" ]] && vmpal delete "$4" --force --permanently' \
    reap "$idle_seconds" "$stamp" "$token" "$clone" >/dev/null 2>&1 &
}

claim_clone
trap reap_clone_when_idle EXIT
vmpal info "$clone" >/dev/null 2>&1 || vmpal clone "$vm" --name "$clone" -q >/dev/null
vmpal start "$clone" --background --wait >/dev/null
vmpal cp "$out/payload.tar" "$clone:jayjay-payload.tar" >/dev/null
rm "$out/payload.tar"
status=0
vmpal exec "$clone" -- /bin/bash -c "$guest_script" test-vm "$bundle_id" "$xctestrun" "$@" | xcbeautify || status=$?
# 125 means the guest script never ran, so the clone could only hand back an earlier run's result.
[[ $status -eq 125 ]] || vmpal cp "$clone:jayjay/ui.xcresult" "$out/" >/dev/null || [[ $status -ne 0 ]]
exit "$status"
