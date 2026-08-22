#!/usr/bin/env bash

set -euo pipefail

repo_root="${1:-$(pwd)}"
release_revision="${RELEASE_CHANGE_REVISION:-@}"
main_revision='main'
remote_name='origin'
remote_main_revision="${main_revision}@${remote_name}"

if ! jj --ignore-working-copy --repository "$repo_root" git fetch --remote "$remote_name"; then
    echo "Release blocked: could not refresh ${remote_main_revision}." >&2
    exit 1
fi

resolve_one() {
    local revision="$1"
    local label="$2"
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

release_parent=$(resolve_one "${release_revision}-" 'the release change parent')
local_main=$(resolve_one "$main_revision" 'local main')
remote_main=$(resolve_one "$remote_main_revision" 'remote main')

if [[ "$local_main" != "$remote_main" ]]; then
    echo "Release blocked: ${main_revision} (${local_main:0:12}) does not match ${remote_main_revision} (${remote_main:0:12})." >&2
    echo "Publish or synchronize main before cutting the release." >&2
    exit 1
fi

if [[ "$release_parent" != "$local_main" ]]; then
    echo "Release blocked: the release change parent (${release_parent:0:12}) is not ${main_revision} (${local_main:0:12})." >&2
    echo "Start the release change directly on main before building release artifacts." >&2
    exit 1
fi

echo "Release base verified: ${release_revision}- = ${main_revision} = ${remote_main_revision} (${local_main:0:12})."
