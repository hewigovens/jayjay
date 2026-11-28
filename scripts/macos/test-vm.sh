#!/usr/bin/env bash
# Runs a ui-test-build output in a clone of a VMPal macOS VM, away from the host's desktop and lock screen; the clone outlives the run and is deleted after idle_seconds without one.
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

bundle_id="${1:?usage: test-vm.sh <bundle-id> [test-ids...]}"
shift
vm="${JAYJAY_UI_TEST_VM:-JayJay UI}"
products="${derived_data#"$root"/}/Build/Products/Debug"
xctest="$(xcode-select -p)/Platforms/MacOSX.platform/Developer"
# The runner is sandboxed, so its own container is the only place it can leave screenshots.
runner_artifacts="Library/Containers/com.jayjay.JayJayUITests.xctrunner/Data/Library/Caches/ui-test-artifacts"
out="$root/build/ui-test-vm"
clone="$vm run"
stamp="$HOME/Library/Caches/jayjay-ui-test-vm-${clone// /-}.stamp"
idle_seconds=900

command -v vmpal >/dev/null || { echo "vmpal not found; link VMPal's command line onto PATH (see agents/testing.md)." >&2; exit 1; }

cd "$root"
rm -rf "$out"
mkdir -p "$out"
# The bundles travel as one tarball: VMPal copies through a shared folder, which breaks the symlinks inside frameworks. The VM has no Xcode, so the host's XCTest frameworks go with them.
tar -cf "$out/payload.tar" \
  --exclude='*.dSYM' --exclude='*.o' --exclude='*.a' --exclude='*.swiftmodule' --exclude=include \
  "$products" scripts/ui-test-fixtures.sh tests/fixtures/formats CONTRIBUTING.md \
  -C "$xctest" Library/Frameworks Library/PrivateFrameworks usr/lib

# The runner app's tool mode runs the scenes without xcodebuild, once it can load the XCTest frameworks.
guest_script='
set -euo pipefail
export PATH="/usr/local/bin:$PATH"
bundle_id="$1"
products="$PWD/jayjay/$2"
artifacts="$PWD/$3"
shift 3
rm -rf jayjay "$artifacts"
mkdir jayjay
cd jayjay
tar -xf ../jayjay-payload.tar
bash scripts/ui-test-fixtures.sh "$bundle_id" >fixtures.log 2>&1 || { cat fixtures.log >&2; exit 1; }
scope="$(IFS=,; echo "${*#JayJayUITests/}")"
status=0
env DYLD_FRAMEWORK_PATH="$products:$products/PackageFrameworks:$PWD/Library/Frameworks" DYLD_LIBRARY_PATH="$products:$PWD/usr/lib" \
  JAYJAY_UI_TEST_APP="$products/JayJay.app" JAYJAY_UI_TEST_ARTIFACTS="$artifacts" \
  "$products/JayJayUITests-Runner.app/Contents/MacOS/JayJayUITests-Runner" --run-as-tool ${scope:+-XCTest "$scope"} 2>&1 || status=$?
mkdir artifacts
[[ ! -d "$artifacts" ]] || cp -R "$artifacts/" artifacts
exit "$status"
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
vmpal exec "$clone" -- /bin/bash -c "$guest_script" test-vm "$bundle_id" "$products" "$runner_artifacts" "$@" | tee "$out/ui.log" | xcbeautify || status=$?
# 125 means the guest script never ran, so the clone could only hand back an earlier run's result.
[[ $status -eq 125 ]] || vmpal cp "$clone:jayjay/artifacts" "$out/" >/dev/null || [[ $status -ne 0 ]]
exit "$status"
