#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

case "$1" in
  reset)
    bundle_id="$2"
    killall Tips "Help Viewer" HelpViewer helpd tipsd 2>/dev/null || true

    cache_roots=(
      "$HOME/Library/Group Containers/group.com.apple.helpviewer.content/Library/Caches"
      "$HOME/Library/Caches/com.apple.helpd/Generated"
      "$HOME/Library/Caches/com.apple.helpd/fsCachedData"
      "$HOME/Library/Containers/com.apple.helpviewer/Data/Library/Caches/com.apple.helpviewer"
      "$HOME/Library/Containers/com.apple.helpviewer/Data/Library/HTTPStorages/com.apple.helpviewer"
      "$HOME/Library/HTTPStorages"
      "$HOME/Library/HTTPStorages/com.apple.helpd"
    )

    for cache_root in "${cache_roots[@]}"; do
      if [[ -d "$cache_root" ]]; then
        find "$cache_root" -mindepth 1 -maxdepth 2 \( \
          -iname "*jayjay*" -o \
          -iname "${bundle_id}*" -o \
          -iname "dev.hewig.jayjay.manual*" -o \
          -iname "dev.hewig.jayjay.help*" \
        \) -exec rm -rf {} +
      fi
    done

    rm -f \
      "$HOME/Library/Caches/com.apple.helpd/CSHelpIndex.plist" \
      "$HOME/Library/Caches/com.apple.helpd/HelpCache.plist" \
      "$HOME/Library/Caches/com.apple.helpd/HSTS.plist" \
      "$HOME/Library/Caches/com.apple.helpd/Cache.db" \
      "$HOME/Library/Caches/com.apple.helpd/Cache.db-shm" \
      "$HOME/Library/Caches/com.apple.helpd/Cache.db-wal"

    help_pref="$HOME/Library/Preferences/com.apple.help.plist"
    if [[ -f "$help_pref" ]]; then
      python3 - "$help_pref" "$bundle_id" <<'PY'
import pathlib
import plistlib
import sys

path = pathlib.Path(sys.argv[1])
bundle_id = sys.argv[2]
with path.open("rb") as source:
    plist = plistlib.load(source)
registered = plist.get("RegisteredBooks")
keys = [key for key in registered if isinstance(key, str) and key.startswith(bundle_id)] if isinstance(registered, dict) else []
for key in keys:
    registered.pop(key)
order = plist.get("RegistrationOrder")
filtered = [item for item in order if not (isinstance(item, str) and item.startswith(bundle_id))] if isinstance(order, list) else order
if isinstance(order, list):
    plist["RegistrationOrder"] = filtered
if keys or filtered != order:
    path.write_bytes(plistlib.dumps(plist))
PY
    fi

    killall Tips "Help Viewer" HelpViewer helpd tipsd 2>/dev/null || true
    ;;
  debug)
    source_help="$project/Resources/JayJayHelpBook"
    generated_help="$root/build/help.noindex/JayJay.help"
    embedded_help="${app}/Contents/Resources/JayJay.help"

    print_help_bundle() {
      local label="$1"
      local bundle="$2"
      echo
      echo "== $label =="
      echo "$bundle"
      if [[ ! -d "$bundle" ]]; then
        echo "missing"
        return
      fi
      plutil -p "$bundle/Contents/Info.plist" | grep -E 'CFBundleVersion|CFBundleShortVersionString|HPDBookTitle|HPDBookIndexPath|HPDBookAccessPath' || true
      du -sh "$bundle" "$bundle/Contents/Resources/English.lproj/imgs" 2>/dev/null || true
      echo "image counts:"
      find "$bundle/Contents/Resources/English.lproj/imgs" -maxdepth 1 -type f \( -name '*.jpg' -o -name '*.png' -o -name '*.webp' \) -print 2>/dev/null \
        | sed 's/.*\.//' \
        | sort \
        | uniq -c || true
      echo "help index:"
      file "$bundle/Contents/Resources/English.lproj/JayJay.helpindex" 2>/dev/null || true
    }

    print_help_bundle "source Help Book templates" "$source_help"
    print_help_bundle "generated Help Book" "$generated_help"
    print_help_bundle "embedded Debug app Help Book" "$embedded_help"

    echo
    echo "== JayJay Help caches =="
    cache_roots=(
      "$HOME/Library/Group Containers/group.com.apple.helpviewer.content/Library/Caches"
      "$HOME/Library/Caches/com.apple.helpd"
      "$HOME/Library/Containers/com.apple.helpviewer"
      "$HOME/Library/HTTPStorages"
    )
    found_cache=0
    for cache_root in "${cache_roots[@]}"; do
      if [[ -d "$cache_root" ]]; then
        while IFS= read -r path; do
          found_cache=1
          echo "$path"
        done < <(find "$cache_root" -maxdepth 4 -iname '*jayjay*' -print 2>/dev/null)
      fi
    done
    if [[ "$found_cache" == "0" ]]; then
      echo "none"
    fi

    echo
    echo "== Spotlight Help Book records =="
    spotlight_records="$(mdfind 'kMDItemCFBundleIdentifier == "dev.hewig.jayjay.manual"' 2>/dev/null || true)"
    if [[ -n "$spotlight_records" ]]; then
      echo "$spotlight_records"
    else
      echo "none"
    fi

    echo
    echo "== Registered JayJay Help books =="
    help_pref="$HOME/Library/Preferences/com.apple.help.plist"
    if [[ -f "$help_pref" ]]; then
      if ! plutil -p "$help_pref" 2>/dev/null \
        | grep -E -C 3 'dev\.hewig\.jayjay|JayJay\.help|JayJayHelpBook|build/help\.noindex|Resources/JayJay\.help'; then
        echo "none"
      fi
    else
      echo "none"
    fi

    echo
    echo "== Recent Help logs =="
    log show --last 10m --style compact --predicate 'process != "log" AND (eventMessage CONTAINS[c] "JayJay.help" OR eventMessage CONTAINS[c] "JayJay Manual" OR eventMessage CONTAINS[c] "outside the sandbox" OR eventMessage CONTAINS[c] "Failed to index")' 2>/dev/null \
      | tail -50 || true
    ;;
  *) echo "usage: $0 {reset <bundle-id>|debug}" >&2; exit 2 ;;
esac
