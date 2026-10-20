#!/usr/bin/env bash
# Builds the screenshot fixture with a GitHub remote whose pushes land in a local bare repo and whose API calls go to video/fake-github.py.
# Usage: stacked-pr-fixture.sh <root> <port>
set -euo pipefail
root="$1" port="$2"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

bash "$here/../scripts/screenshot-fixture.sh" "$root"
git init --quiet --bare "$root/flightdeck-remote.git"
cat > "$root/gitconfig" <<GIT
[url "file://$root/flightdeck-remote.git"]
	pushInsteadOf = https://github.com/jayjay-demo/flightdeck.git
GIT
cd "$root/flightdeck"
export JJ_CONFIG="$root/jj-config.toml" GIT_CONFIG_GLOBAL="$root/gitconfig"
jj bookmark track main --remote origin >/dev/null 2>&1 || true
jj git push --bookmark main

pkill -f "fake-github.py $port" || true
nohup python3 "$here/fake-github.py" "$port" >"$root/fake-github.log" 2>&1 &
