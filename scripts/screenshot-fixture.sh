#!/usr/bin/env bash
# Builds the "flightdeck" demo repository behind the public screenshots (docs/imgs), captured by
# shell/mac/Tests/JayJayUITests/Screenshots/ReleaseScreenshots.swift; see agents/release.md.
#
# Usage: screenshot-fixture.sh [root]   (default /tmp/jayjay-screenshots)
set -euo pipefail

root="${1:-/tmp/jayjay-screenshots}"
repo="$root/flightdeck"
export JJ_USER="JayJay CI" JJ_EMAIL="ci@jayjay.local"

rm -rf "$root"
mkdir -p "$root"
jj git init --colocate "$repo"
cd "$repo"
git remote add origin https://github.com/jayjay-demo/flightdeck.git

write_routes() {
  local sort_key="$1" describe="$2"
  cat > src/routes.rs <<RS
use crate::{Journey, Route, RouteError};

pub fn plan_route(journey: &Journey) -> Result<Route, RouteError> {
    let mut candidates = journey.available_routes();
    candidates.retain(|route| route.is_available());
    candidates.sort_by_key($sort_key);

    candidates.into_iter().next()
        .ok_or(RouteError::NoAvailableRoute)
}

pub fn describe_route(route: &Route) -> String {
$describe
}

pub fn estimated_minutes(route: &Route) -> u32 {
    route.duration_minutes + route.transfer_minutes
}
RS
}

mkdir -p src tests
cat > Cargo.toml <<'TOML'
[package]
name = "flightdeck"
version = "0.4.0"
edition = "2024"

[dependencies]
thiserror = "2"
TOML
cat > README.md <<'MD'
# flightdeck

Plans multi-leg journeys across rail, bus, and ferry timetables.

## Usage

    cargo run -- plan --from Lisbon --to Porto
MD
cat > src/lib.rs <<'RS'
mod routes;

pub use routes::{describe_route, estimated_minutes, plan_route};

pub struct Journey {
    pub origin: String,
    pub destination: String,
    routes: Vec<Route>,
}

#[derive(Clone)]
pub struct Route {
    pub origin: String,
    pub destination: String,
    pub duration_minutes: u32,
    pub transfer_minutes: u32,
    pub available: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum RouteError {
    #[error("no route is available for this journey")]
    NoAvailableRoute,
}
RS
write_routes '|route| route.duration_minutes' '    format!("{} → {}", route.origin, route.destination)'
jj describe -m "feat: add journey planning"
jj bookmark create main -r @

# Side branches come first so the feature stack and working copy stay at the top of the graph; the sibling workspace adds a name@ chip and a second picker row.
jj workspace add --name route-experiment -r main "$root/flightdeck-route-experiment" >/dev/null
# An empty, described change gives the command palette a blank backdrop to blur.
jj describe -r route-experiment@ -m "wip: try weighted routing" >/dev/null

# Two describes of one change from the same operation leave it divergent.
jj new main -m "experiment: score routes by comfort"
echo 'pub fn comfort_score(transfers: u32) -> u32 { 100 - transfers * 15 }' > src/comfort.rs
base_op=$(jj op log --no-graph -n1 -T 'id.short(16)')
jj describe -m "experiment: score routes by comfort (weighted)"
jj --at-op "$base_op" describe -m "experiment: score routes by comfort (linear)" >/dev/null 2>&1

# The timetable edits collide on the same lines, so the merge change is a two-sided conflict.
jj new main -m "feat: load timetables from the rail feed"
cat > src/timetable.rs <<'RS'
pub const FEED_URL: &str = "https://feeds.example.com/rail";
pub const REFRESH_MINUTES: u32 = 15;

pub fn departures(station: &str) -> Vec<String> {
    let url = format!("{FEED_URL}/{station}");
    fetch_lines(&url)
}
RS
jj bookmark create rail-feed -r @
jj new main -m "feat: load timetables from the ferry feed"
cat > src/timetable.rs <<'RS'
pub const FEED_URL: &str = "https://feeds.example.com/ferry";
pub const REFRESH_MINUTES: u32 = 30;

pub fn departures(station: &str) -> Vec<String> {
    let url = format!("{FEED_URL}/{station}?mode=ferry");
    fetch_lines(&url)
}
RS
jj new rail-feed @ -m "merge rail and ferry timetables"

jj new main -m "test: cover route endpoints"

cat > tests/routes.rs <<'RS'
use flightdeck::{describe_route, Route};

fn lisbon_to_porto() -> Route {
    Route {
        origin: "Lisbon".into(),
        destination: "Porto".into(),
        duration_minutes: 165,
        transfer_minutes: 0,
        available: true,
    }
}

#[test]
fn describes_both_endpoints() {
    assert_eq!(describe_route(&lisbon_to_porto()), "Lisbon → Porto");
}
RS

jj new -m "feat: cache recently planned routes"
cat > src/cache.rs <<'RS'
use std::collections::HashMap;

use crate::Route;

#[derive(Default)]
pub struct RouteCache {
    entries: HashMap<(String, String), Route>,
}

impl RouteCache {
    pub fn get(&self, origin: &str, destination: &str) -> Option<&Route> {
        self.entries.get(&(origin.to_owned(), destination.to_owned()))
    }

    pub fn insert(&mut self, route: Route) {
        let key = (route.origin.clone(), route.destination.clone());
        self.entries.insert(key, route);
    }
}
RS
sed -i '' 's/^mod routes;/mod cache;\nmod routes;/' src/lib.rs
jj bookmark create route-cache -r @

jj new -m "feat: prefer the fastest available route"
write_routes '|route| route.duration_minutes + route.transfer_minutes' '    format!("{} → {}", route.origin, route.destination)'
jj bookmark create faster-routes -r @

jj new -m "feat: include transfer time in route estimates"
write_routes '|route| estimated_minutes(route)' '    format!(
        "{} → {} · {} min",
        route.origin,
        route.destination,
        estimated_minutes(route),
    )'
jj st >/dev/null
sed -i '' 's/        self.entries.insert(key, route);/        self.entries.insert(key, route);\n        self.evict_stale();/' src/cache.rs
cat >> src/cache.rs <<'RS'

impl RouteCache {
    const CAPACITY: usize = 64;

    fn evict_stale(&mut self) {
        while self.entries.len() > Self::CAPACITY {
            let Some(key) = self.entries.keys().next().cloned() else {
                break;
            };
            self.entries.remove(&key);
        }
    }
}
RS
jj st >/dev/null
sed -i '' 's/const CAPACITY: usize = 64;/const CAPACITY: usize = 128;/' src/cache.rs
jj st >/dev/null
feature=$(jj log --no-graph -r @ -T 'change_id.short(12)')

# The working copy sits on the feature stack with review-ready edits.
jj new "$feature"
sed -i '' 's/    candidates.retain(|route| route.is_available());/    candidates.retain(|route| route.is_available() \&\& route.origin != route.destination);/' src/routes.rs
cat >> tests/routes.rs <<'RS'

#[test]
fn describes_estimated_minutes() {
    let route = lisbon_to_porto();
    assert_eq!(describe_route(&route), "Lisbon → Porto · 165 min");
}
RS
cat > src/stations.rs <<'RS'
use crate::{Journey, RouteError};

pub fn validate(journey: &Journey) -> Result<(), RouteError> {
    if journey.origin == journey.destination {
        return Err(RouteError::NoAvailableRoute);
    }
    Ok(())
}
RS
cat >> README.md <<'MD'

Estimates include transfer time, and recently planned routes are cached.
MD
jj st >/dev/null

printf '{"repositories":["%s"]}\n' "$repo" > "$root/repositories.json"
# The app reads this instead of the machine's own jj config, so Settings > Jujutsu shows demo values.
cat > "$root/jj-config.toml" <<'TOML'
[user]
name = "JayJay CI"
email = "ci@jayjay.local"

[operation]
hostname = "flightdeck-ci"
username = "jayjay"

[ui]
default-command = "log"
diff-formatter = ":git"

[revsets]
log = "present(@) | ancestors(immutable_heads().., 2) | trunk()"
TOML
# DAG rows expose their selection revision, not their text, so the capture tests look rows up by subject here.
jj log --no-graph -r 'all() ~ root()' \
  -T 'if(divergent, commit_id.short(12), change_id.short(12)) ++ "\t" ++ description.first_line() ++ "\n"' > "$root/rows.tsv"
echo "Built $repo"
