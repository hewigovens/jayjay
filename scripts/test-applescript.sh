#!/usr/bin/env bash
# Exercise the scripting dictionary with real Apple events against an isolated copy of a built app.
# Set JAYJAY_APPLESCRIPT_SHOT=<file.png> to also capture the window after `show file`.
set -euo pipefail

[[ $# -eq 1 && -d "$1" ]] || { echo "usage: $0 /path/to/JayJay.app" >&2; exit 2; }

bundle_id=dev.hewig.jayjay.scripting-test
root="$(cd "$(mktemp -d "${TMPDIR:-/tmp}/jayjay-scripting.XXXXXX")" && pwd -P)"
app="$root/ScriptingTest.app"
pid=""

cleanup() {
    if [[ -n "$pid" ]]; then
        kill "$pid" 2>/dev/null || true
        for _ in {1..100}; do
            kill -0 "$pid" 2>/dev/null || break
            sleep 0.1
        done
        kill -9 "$pid" 2>/dev/null || true
    fi
    defaults delete "$bundle_id" >/dev/null 2>&1 || true
    rm -rf "$root"
    printf 'y\n' | JJ_INTERACTIVE=1 jj config gc >/dev/null 2>&1 || true
}
trap cleanup EXIT
trap 'exit 130' INT TERM

cp -cR "$1" "$app"
/usr/libexec/PlistBuddy -c "Set :CFBundleIdentifier $bundle_id" "$app/Contents/Info.plist"
codesign --force --sign - "$app" >/dev/null 2>&1
defaults write "$bundle_id" jayjay.hasCompletedOnboarding -bool YES
defaults write "$bundle_id" jayjay.sendsAnonymousStats -bool NO
defaults write "$bundle_id" SUEnableAutomaticChecks -bool NO

quoted() {
    local value=${1//\\/\\\\}
    printf '"%s"' "${value//\"/\\\"}"
}

# Runs the script on stdin inside `tell application`; a script-level `on error` is not needed because `attempt` captures failures.
run() {
    {
        printf 'using terms from application %s\ntell application id "%s"\n' "$(quoted "$app")" "$bundle_id"
        cat -
        printf '\nend tell\nend using terms from\n'
    } > "$root/script.applescript"
    osascript "$root/script.applescript"
}

# AppleScript that runs a statement and leaves "ok" or the error number in a variable.
attempt() {
    printf 'try\n%s\nset %s to "ok"\non error number errorNumber\nset %s to errorNumber as text\nend try\n' "$2" "$1" "$1"
}

expect() {
    if [[ "$2" != "$1" ]]; then
        echo "FAIL: $3: expected [$1], got [$2]" >&2
        exit 1
    fi
    echo "PASS: $3"
}

first="$root/first repo"
second="$root/second repo"
jj_first() { jj -R "$first" "$@" >/dev/null 2>&1; }
for repo in "$first" "$second"; do
    jj git init "$repo" >/dev/null 2>&1
    jj -R "$repo" config set --repo user.name "JayJay Scripting Test" >/dev/null 2>&1
    jj -R "$repo" config set --repo user.email "scripting@example.com" >/dev/null 2>&1
done
mkdir -p "$first/sub dir"
echo one > "$first/a.txt"
echo two > "$first/sub dir/b.txt"
jj_first describe -m "two files"
jj_first new -m "empty tip"
first="$(cd "$first" && pwd -P)"
second="$(cd "$second" && pwd -P)"
tip="$(jj -R "$first" log -r @ --no-graph -T change_id)"
parent="$(jj -R "$first" log -r '@-' --no-graph -T change_id)"

open -n "$app"
for _ in {1..100}; do
    pid="$(pgrep -f "$app/Contents/MacOS/JayJay" | head -1 || true)"
    [[ -z "$pid" ]] || break
    sleep 0.1
done
[[ -n "$pid" ]] || { echo "The isolated app did not launch" >&2; exit 1; }

expect "true|0|-1728|-1700|-1728" "$(run <<EOF
$(attempt selectionResult 'select change "@"')
$(attempt relativeResult 'open repository "relative/path"')
$(attempt missingResult 'open repository "/this/path/does/not/exist"')
return ((front repository is missing value) as text) & "|" & (count repositories) & "|" & selectionResult & "|" & relativeResult & "|" & missingResult
EOF
)" "nothing open: no front repository, and bad arguments fail"

expect "$first|$first|$tip|true|true|true" "$(run <<EOF
set r to open repository $(quoted "$first/sub dir")
set ids to get selected change ids of r
return (path of r) & "|" & (id of r) & "|" & (ids as text) & "|" & ((window id of r) > 0) & "|" & (error message of r is missing value) & "|" & (notice of r is missing value)
EOF
)" "open repository returns the loaded workspace root with @ selected, a window id, and no messages"

expect "$first|$second,$first|present(@)|ok|-1700|-1700" "$(run <<EOF
set firstRepo to repository id $(quoted "$first")
open repository $(quoted "$second")
set listed to {}
repeat with r in repositories
	set end of listed to path of r
end repeat
set text item delimiters of AppleScript to ","
set listedText to listed as text
set revset of firstRepo to "present(@)"
$(attempt refreshedResult "refresh repository id $(quoted "$first")")
$(attempt everyResult 'refresh repositories')
$(attempt everyRevsetResult 'set revset of every repository to "all()"')
return (path of firstRepo) & "|" & listedText & "|" & (revset of firstRepo) & "|" & refreshedResult & "|" & everyResult & "|" & everyRevsetResult
EOF
)" "a reference stays on its repository while another is frontmost; a list of repositories is refused"

window_id="$(run <<EOF
set r to repository id $(quoted "$first")
set revset of r to "all()"
select change "@-" in r
show file "sub dir/b.txt" in r
return window id of r
EOF
)"
[[ -z "${JAYJAY_APPLESCRIPT_SHOT:-}" ]] || screencapture -x -o -l "$window_id" "$JAYJAY_APPLESCRIPT_SHOT"

expect "all()|$parent|-1728|-10000|all()" "$(run <<EOF
set r to repository id $(quoted "$first")
set ids to get selected change ids of r
$(attempt noFileResult 'show file "missing.txt" in r')
$(attempt badRevsetResult 'set revset of r to "(("')
return (revset of r) & "|" & (ids as text) & "|" & noFileResult & "|" & badRevsetResult & "|" & (revset of r)
EOF
)" "set revset, select change and show file settle before returning; a missing file and an invalid revset fail"

expect "-1728|-1728|-1728|$tip" "$(run <<EOF
set r to repository id $(quoted "$first")
$(attempt unknownResult 'select change "zzzzzzzzzzzzzzzz" in r')
$(attempt severalResult 'select change "all()" in r')
set revset of r to "@"
$(attempt outsideResult 'select change "@-" in r')
set ids to get selected change ids of r
return unknownResult & "|" & severalResult & "|" & outsideResult & "|" & (ids as text)
EOF
)" "unknown, ambiguous and out-of-graph revisions fail and leave the selection alone"

jj_first new -m "made behind the app"
expect "$(jj -R "$first" log -r @ --no-graph -T change_id)" "$(run <<EOF
set r to repository id $(quoted "$first")
refresh r
select change "@" in r
set ids to get selected change ids of r
return ids as text
EOF
)" "refresh picks up a change made with the jj CLI"

echo "All AppleScript automation cases passed"
