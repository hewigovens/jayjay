#!/usr/bin/env bash
# Launches an AppImage under Xvfb with the host's Vulkan loader and driver and fails unless it opens a window.
set -euo pipefail
[[ $# -eq 1 ]] || { echo "usage: $0 <AppImage>" >&2; exit 2; }
appimage=$(realpath "$1")

# Xvfb's GLX crashes on Arch with Mesa 26.2; JayJay renders through Vulkan.
xvfb_log=$(mktemp)
Xvfb :99 -screen 0 1280x800x24 -extension GLX >"$xvfb_log" 2>&1 &
xvfb=$!
export DISPLAY=:99
sleep 1
kill -0 "$xvfb" 2>/dev/null || { cat "$xvfb_log"; echo "error: Xvfb exited" >&2; exit 1; }

log=$(mktemp)
cd "$(mktemp -d)"
"$appimage" >"$log" 2>&1 &
app=$!
trap 'kill "$app" "$xvfb" 2>/dev/null || true' EXIT
sleep 10

cat "$log"
kill -0 "$app" 2>/dev/null || { echo "error: $appimage exited before opening a window" >&2; exit 1; }
xwininfo -root -tree | grep -q JayJay || { echo "error: no JayJay window on $DISPLAY" >&2; exit 1; }
echo "smoke test passed"
