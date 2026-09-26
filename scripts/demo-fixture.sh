#!/usr/bin/env bash
# Builds the intro-video demo: the flightdeck screenshot repository (scripts/screenshot-fixture.sh) after trunk moved on, with agent workspaces that give Repo Overview several lanes and agent review marks and notes on two of them.
#
# Usage: demo-fixture.sh [root]   (default /tmp/jayjay-demo)
# JAYJAY_BIN is the app binary that serves `review` commands (default: jayjay on PATH).
set -euo pipefail

root="${1:-/tmp/jayjay-demo}"
repo="$root/flightdeck"
fares="$root/flightdeck-fares"
jayjay="${JAYJAY_BIN:-jayjay}"
export JJ_USER="JayJay CI" JJ_EMAIL="ci@jayjay.local"
# The app shows these marks and notes only when launched with the same store, as `just shell::demo` does.
export JAYJAY_REVIEW_STORE_PATH="$root/review-store.json"

ago() {
  date -u -v-"$1" +%Y-%m-%dT%H:%M:%SZ
}

# screenshot-fixture.sh deletes the root first, so refuse a non-empty directory that does not hold a previous demo.
if [ -n "$(ls -A "$root" 2>/dev/null)" ] && [ ! -d "$repo/.jj" ]; then
  echo "error: $root is not empty and holds no previous demo; pass a new directory" >&2
  exit 1
fi
JJ_TIMESTAMP="$(ago 3d)" bash "$(dirname "$0")/screenshot-fixture.sh" "$root"
cd "$repo"

# Trunk moves on without the feature stack, leaving it 3 behind main; the README paragraph conflicts with the working copy's once the stack is rebased.
wc=$(jj log --no-graph -r @ -T change_id)
(
  export JJ_TIMESTAMP="$(ago 2d)"
  jj new main -m "chore: pin the Rust toolchain"
  printf '[toolchain]\nchannel = "1.96"\n' > rust-toolchain.toml
  jj st >/dev/null

  export JJ_TIMESTAMP="$(ago 1d)"
  jj new -m "docs: explain how timetable feeds refresh"
  cat >> README.md <<'MD'

## Timetables

Rail and ferry feeds refresh every 15 minutes. Set `FLIGHTDECK_FEED_MIRROR` to read them from a local mirror.
MD
  jj st >/dev/null

  export JJ_TIMESTAMP="$(ago 6H)"
  jj new -m "ci: run tests on pull requests"
  mkdir -p .github/workflows
  cat > .github/workflows/ci.yml <<'YML'
name: CI
on: [pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - run: cargo test --locked
YML
  jj st >/dev/null
  jj bookmark set main -r @
)
jj edit "$wc"
# A pushed main makes trunk() resolve, so everything below it is immutable and older bases count as behind.
git update-ref refs/remotes/origin/main "$(jj log --no-graph -r main -T commit_id)"
jj bookmark track main@origin

(
  export JJ_TIMESTAMP="$(ago 5H)"
  jj workspace add --name bus-feed -r main-- -m "feat: load timetables from the bus feed" "$root/flightdeck-bus-feed"
  cd "$root/flightdeck-bus-feed"
  cat > src/bus.rs <<'RS'
pub const FEED_URL: &str = "https://feeds.example.com/bus";
pub const REFRESH_MINUTES: u32 = 5;

pub fn stop_departures(stop: &str) -> Vec<String> {
    let url = format!("{FEED_URL}/stops/{stop}");
    fetch_lines(&url)
}
RS
  sed -i '' 's/^mod routes;/mod bus;\nmod routes;/; s/^pub use routes::/pub use bus::stop_departures;\npub use routes::/' src/lib.rs
  jj new -m "test: cover bus stop lookups"
  mkdir -p tests
  cat > tests/bus.rs <<'RS'
use flightdeck::stop_departures;

#[test]
fn unknown_stops_have_no_departures() {
    assert!(stop_departures("nowhere").is_empty());
}
RS
  jj st >/dev/null
)

# A finished agent leaves an empty, undescribed checkout above its work, which Overview flags.
(
  export JJ_TIMESTAMP="$(ago 40M)"
  jj workspace add --name cli-docs -r main -m "docs: document the plan command's flags" "$root/flightdeck-cli-docs"
  cd "$root/flightdeck-cli-docs"
  sed -i '' 's/^    cargo run -- plan --from Lisbon --to Porto$/    cargo run -- plan --from Lisbon --to Porto --via Coimbra --depart 08:30\n\n`--via` adds a stop, and `--depart` sets the earliest departure./' README.md
  jj new
)

(
  export JJ_TIMESTAMP="$(ago 25M)"
  jj workspace add --name fares -r main -m "feat: add fare tables" "$fares"
  cd "$fares"
  cat > src/fares.rs <<'RS'
use crate::Route;

pub const CURRENCY: &str = "EUR";

pub struct FareTable {
    pub cents_per_minute: u32,
    pub transfer_cents: u32,
}

impl FareTable {
    pub const RAIL: FareTable = FareTable { cents_per_minute: 9, transfer_cents: 150 };
    pub const FERRY: FareTable = FareTable { cents_per_minute: 14, transfer_cents: 0 };
}

pub fn estimate_fare(legs: &[(FareTable, Route)]) -> u32 {
    legs.iter()
        .map(|(table, route)| table.cents_per_minute * route.duration_minutes + table.transfer_cents)
        .sum()
}
RS
  sed -i '' 's/^mod routes;/mod fares;\nmod routes;/' src/lib.rs
  jj st >/dev/null

  # The rounding rewrites an existing line so the agent's note sits on a word-level diff.
  export JJ_TIMESTAMP="$(ago 6M)"
  jj new -m "feat: round fare estimates up to 10 cents"
  sed -i '' 's/^pub struct FareTable {/#[derive(Clone, Copy, Debug)]\npub struct FareTable {/; s/| table.cents_per_minute/| round_up_to_ten(table.cents_per_minute/; s/+ table.transfer_cents)$/+ table.transfer_cents))/' src/fares.rs
  cat >> src/fares.rs <<'RS'

fn round_up_to_ten(cents: u32) -> u32 {
    cents.div_ceil(10) * 10
}
RS
  sed -i '' 's/^pub use routes::/pub use fares::{estimate_fare, FareTable};\npub use routes::/' src/lib.rs
  mkdir -p tests
  cat > tests/fares.rs <<'RS'
use flightdeck::{estimate_fare, FareTable, Route};

fn leg(minutes: u32) -> Route {
    Route {
        origin: "Lisbon".into(),
        destination: "Porto".into(),
        duration_minutes: minutes,
        transfer_minutes: 0,
        available: true,
    }
}

#[test]
fn rounds_each_leg_up_to_ten_cents() {
    assert_eq!(estimate_fare(&[(FareTable::RAIL, leg(165))]), 1640);
}
RS
  jj st >/dev/null
)

# The agent's triage pass: straightforward files and groups marked, one note per concrete risk (see the review-triage skill).
line_of() {
  grep -n -m 1 -F "$2" "$1" | cut -d: -f1
}
mark() {
  local workspace="$1"
  shift
  "$jayjay" review mark --repo "$workspace" --expected-commit "$(jj -R "$workspace" log --no-graph -r @ -T commit_id)" "$@"
}

mark "$repo" --file README.md
mark "$repo" --file tests/routes.rs
"$jayjay" review add-note --repo "$repo" --file src/stations.rs --line "$(line_of src/stations.rs 'pub fn validate')" \
  -m "Nothing declares mod stations; in lib.rs, so validate never compiles and plan_route never calls it. Declare the module and call validate before planning, or drop the file."

mark "$fares" --file src/lib.rs
mark "$fares" --file tests/fares.rs
mark "$fares" --file src/fares.rs --line "$(line_of "$fares/src/fares.rs" '#[derive(Clone, Copy, Debug)]')"
"$jayjay" review add-note --repo "$fares" --file src/fares.rs --line "$(line_of "$fares/src/fares.rs" '.map(|(table, route)|')" \
  -m "Rounding each leg up to 10 cents before summing charges multi-leg journeys up to 9 cents extra per leg. Confirm the operator rounds per leg; otherwise round the total once."

# The workspace lanes are new rows, so the Overview capture looks them up here like the DAG rows.
jj log --no-graph -r 'all() ~ root()' \
  -T 'if(divergent, commit_id.short(12), change_id.short(12)) ++ "\t" ++ description.first_line() ++ "\n"' > "$root/rows.tsv"
echo "Built demo in $root"
