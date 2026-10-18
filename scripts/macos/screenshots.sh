#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

screenshot_root="$1"
shift

bash "${root}/scripts/screenshot-fixture.sh" "${screenshot_root}"
xcodegen --spec "${project}/project.yml" --project "${project}"
out="${root}/build/screenshots"
rm -rf "$out"
mkdir -p "$out"
args=()
if [ "$#" -eq 0 ]; then
  args+=(-only-testing:JayJayUITests/ReleaseScreenshots -only-testing:JayJayUITests/OverviewScreenshots)
fi
for test_id in "$@"; do
  case "$test_id" in
    */*) args+=(-only-testing:"JayJayUITests/$test_id") ;;
    *) args+=(-only-testing:"JayJayUITests/ReleaseScreenshots/$test_id") ;;
  esac
done
xcode JayJayUITests Debug "${args[@]}" build-for-testing
# The demo's review marks come from the app binary, so it waits for the build.
JAYJAY_BIN="${app}/Contents/MacOS/JayJay" bash "${root}/scripts/demo-fixture.sh"
status=0
for appearance in light dark; do
  # The runner cannot always delete the previous pass's review marks and notes, so each appearance starts clean here.
  rm -f "${screenshot_root}"/*-review-store.json
  TEST_RUNNER_JAYJAY_SCREENSHOT_ROOT="${screenshot_root}" TEST_RUNNER_JAYJAY_SCREENSHOT_APPEARANCE="$appearance" xcode JayJayUITests Debug "${args[@]}" \
    -resultBundlePath "$out/$appearance.xcresult" test-without-building || status=1
  xcrun xcresulttool export attachments --path "$out/$appearance.xcresult" --output-path "$out/$appearance"
done
python3 - "$out" "${root}/docs/imgs" <<'PY'
import json, pathlib, subprocess, sys
out, imgs = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
for manifest in out.glob("*/manifest.json"):
    for test in json.loads(manifest.read_text()):
        for attachment in test["attachments"]:
            name = attachment["suggestedHumanReadableName"].split("_")[0]
            if " " in name or not attachment["exportedFileName"].endswith(".png"):
                continue
            source = manifest.parent / attachment["exportedFileName"]
            subprocess.run(["cwebp", "-quiet", "-q", "90", str(source), "-o", str(imgs / f"{name}.webp")], check=True)
            print(f"docs/imgs/{name}.webp")
PY
exit "$status"
