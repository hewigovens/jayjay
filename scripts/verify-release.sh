#!/usr/bin/env bash

set -euo pipefail

usage() {
    echo "Usage:" >&2
    echo "  verify-release.sh base [repo_root] [release_revision]" >&2
    echo "  verify-release.sh archive <zip_path> [signed|notarized]" >&2
}

resolve_one() {
    local repo_root="$1"
    local revision="$2"
    local label="$3"
    local commits
    local count

    if ! commits=$(jj --ignore-working-copy log \
        --repository "$repo_root" \
        --revisions "$revision" \
        --no-graph \
        --template 'commit_id ++ "\n"'); then
        echo "Release blocked: could not resolve ${label} (${revision})." >&2
        return 1
    fi

    count=$(printf '%s\n' "$commits" | sed '/^$/d' | wc -l | tr -d ' ')
    if [[ "$count" != "1" ]]; then
        echo "Release blocked: ${label} (${revision}) must resolve to exactly one commit; found ${count}." >&2
        return 1
    fi

    printf '%s' "$commits"
}

verify_base() {
    [[ $# -le 2 ]] || { usage; return 2; }

    local repo_root="${1:-$(pwd)}"
    local release_revision="${2:-@}"
    local main_revision='main' remote_name='origin'
    local remote_main_revision="${main_revision}@${remote_name}"
    local release_parent local_main remote_main

    if ! jj --ignore-working-copy --repository "$repo_root" git fetch --remote "$remote_name"; then
        echo "Release blocked: could not refresh ${remote_main_revision}." >&2
        return 1
    fi

    release_parent=$(resolve_one "$repo_root" "${release_revision}-" 'the release change parent')
    local_main=$(resolve_one "$repo_root" "$main_revision" 'local main')
    remote_main=$(resolve_one "$repo_root" "$remote_main_revision" 'remote main')

    if [[ "$local_main" != "$remote_main" ]]; then
        echo "Release blocked: ${main_revision} (${local_main:0:12}) does not match ${remote_main_revision} (${remote_main:0:12})." >&2
        echo "Publish or synchronize main before cutting the release." >&2
        return 1
    fi

    if [[ "$release_parent" != "$local_main" ]]; then
        echo "Release blocked: the release change parent (${release_parent:0:12}) is not ${main_revision} (${local_main:0:12})." >&2
        echo "Start the release change directly on main before building release artifacts." >&2
        return 1
    fi

    echo "Release base verified: ${release_revision}- = ${main_revision} = ${remote_main_revision} (${local_main:0:12})."
}

verify_archive() {
    [[ $# -ge 1 && $# -le 2 ]] || { usage; return 2; }

    local zip_path="$1" mode="${2:-signed}"
    local zip_listing metadata_entry app_path dock_plugin principal_class
    local appledouble_file macosx_dir
    local -a apps=()

    case "$mode" in
        signed | notarized) ;;
        *)
            echo "Error: mode must be 'signed' or 'notarized', got '$mode'" >&2
            return 2
            ;;
    esac

    if [[ ! -f "$zip_path" ]]; then
        echo "Error: archive not found: $zip_path" >&2
        return 1
    fi

    zip_listing="$(zipinfo -1 "$zip_path")"
    metadata_entry="$(
        printf '%s\n' "$zip_listing" | grep -E '(^|/)\._|(^|/)__MACOSX(/|$)' | head -n 1 || true
    )"
    if [[ -n "$metadata_entry" ]]; then
        echo "Error: archive contains AppleDouble metadata entry: $metadata_entry" >&2
        return 1
    fi

    tmp_dir="$(mktemp -d)"
    trap 'rm -rf "$tmp_dir"' EXIT
    ditto -x -k "$zip_path" "$tmp_dir"

    while IFS= read -r -d '' app; do
        apps+=("$app")
    done < <(find "$tmp_dir" -maxdepth 1 -type d -name '*.app' -print0)

    if [[ "${#apps[@]}" -ne 1 ]]; then
        echo "Error: expected one top-level .app in archive, found ${#apps[@]}" >&2
        return 1
    fi

    app_path="${apps[0]}"
    dock_plugin="$app_path/Contents/PlugIns/JayJayDockTilePlugin.docktileplugin"
    if [[ ! -x "$dock_plugin/Contents/MacOS/JayJayDockTilePlugin" ]]; then
        echo "Error: Dock tile plugin is missing from the app bundle: $dock_plugin" >&2
        return 1
    fi

    principal_class="$(defaults read "$dock_plugin/Contents/Info" NSPrincipalClass 2>/dev/null || true)"
    if [[ "$principal_class" != "JayJayDockTilePlugin" ]]; then
        echo "Error: Dock tile plugin has an unexpected principal class: $principal_class" >&2
        return 1
    fi

    appledouble_file="$(find "$app_path" -name '._*' -print -quit)"
    if [[ -n "$appledouble_file" ]]; then
        echo "Error: extracted app contains AppleDouble file: $appledouble_file" >&2
        return 1
    fi

    macosx_dir="$(find "$app_path" -name '__MACOSX' -type d -print -quit)"
    if [[ -n "$macosx_dir" ]]; then
        echo "Error: extracted app contains metadata directory: $macosx_dir" >&2
        return 1
    fi

    codesign --verify --deep --strict --verbose=2 "$app_path"

    if [[ "$mode" == "notarized" ]]; then
        xcrun stapler validate "$app_path"
        spctl --assess --type execute --verbose "$app_path"
    fi

    echo "Verified $zip_path ($mode)"
}

command="${1:-}"
case "$command" in
    base) shift; verify_base "$@" ;;
    archive) shift; verify_archive "$@" ;;
    *)
        usage
        exit 2
        ;;
esac
