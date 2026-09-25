use std::fs;
use std::path::Path;

use jayjay_core::overview::overview_groups;
use jayjay_core::{OverviewBaseKind, OverviewLane, RebaseMode, Repo};
use jj_test::{init_jj_repo, run_jj_in};

fn describe_new(repo: &Path, parent: &str, message: &str) {
    run_jj_in(repo, &["new", "-m", message, parent]);
}

fn lane_by_head<'a>(lanes: &'a [OverviewLane], description: &str) -> &'a OverviewLane {
    lanes
        .iter()
        .find(|lane| lane.head().description == description)
        .unwrap_or_else(|| panic!("lane headed by {description}"))
}

/// Trunk `main` at the second commit, four lanes: two on main (one holding @), a three-change stack on the older
/// trunk commit with a sibling workspace checked out one below the head, and an empty checkout above one change.
fn build_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp_dir = init_jj_repo();
    let repo = temp_dir.path().join("repo");
    run_jj_in(&repo, &["bookmark", "create", "-r", "@", "old-main"]);
    describe_new(&repo, "@", "trunk: second commit");
    run_jj_in(&repo, &["bookmark", "create", "-r", "@", "main"]);
    run_jj_in(&repo, &["bookmark", "delete", "old-main"]);
    for (key, value) in [
        ("revset-aliases.'trunk()'", "main"),
        ("revset-aliases.'immutable_heads()'", "main"),
    ] {
        run_jj_in(&repo, &["config", "set", "--repo", key, value]);
    }

    describe_new(&repo, "main", "billing: helpers");
    describe_new(&repo, "@", "billing: validate currency");
    fs::write(repo.join("billing.txt"), "currency\n").expect("write billing edit");
    describe_new(&repo, "main", "search: normalize whitespace");
    run_jj_in(&repo, &["describe", "-m", "search: normalize whitespace"]);

    describe_new(&repo, "main-", "inventory: rollup refresh");
    describe_new(&repo, "@", "inventory: forecast engine");
    describe_new(&repo, "@", "inventory: nightly rollups");
    let planner = temp_dir.path().join("planner");
    run_jj_in(
        &repo,
        &[
            "workspace",
            "add",
            "--name",
            "planner",
            planner.to_str().expect("utf8"),
        ],
    );
    run_jj_in(
        &planner,
        &["edit", "subject(\"inventory: forecast engine\")"],
    );

    describe_new(&repo, "main", "notifications: chat integration");
    let bridge = temp_dir.path().join("bridge");
    run_jj_in(
        &repo,
        &[
            "workspace",
            "add",
            "--name",
            "bridge",
            "-r",
            "subject(\"notifications: chat integration\")",
            bridge.to_str().expect("utf8"),
        ],
    );

    run_jj_in(&repo, &["edit", "subject(\"billing: validate currency\")"]);
    (temp_dir, repo)
}

#[test]
fn overview_groups_mutable_changes_into_lanes_with_bases_and_workspaces() {
    let (_temp_dir, repo_path) = build_fixture();
    let repo = Repo::open(&repo_path).expect("open repo");

    let overview = repo.overview().expect("overview");
    assert_eq!(overview.workspace_count, 3);
    assert_eq!(overview.lanes.len(), 4, "{:#?}", overview.lanes);
    assert_eq!(
        overview
            .lanes
            .iter()
            .map(|lane| lane.changes.len())
            .sum::<usize>(),
        8
    );

    let billing = lane_by_head(&overview.lanes, "billing: validate currency");
    assert_eq!(billing.changes.len(), 2);
    assert_eq!(billing.base.kind, OverviewBaseKind::Trunk);
    assert_eq!(billing.base.bookmarks, ["main"]);
    assert_eq!(billing.workspaces.len(), 1);
    assert!(billing.workspaces[0].is_current);
    assert_eq!(billing.workspaces[0].changes_above, 0);
    assert!(billing.attention.is_empty());

    let search = lane_by_head(&overview.lanes, "search: normalize whitespace");
    assert_eq!(search.changes.len(), 1);
    assert!(search.workspaces.is_empty());

    let inventory = lane_by_head(&overview.lanes, "inventory: nightly rollups");
    assert_eq!(
        inventory
            .changes
            .iter()
            .map(|c| c.description.as_str())
            .collect::<Vec<_>>(),
        [
            "inventory: nightly rollups",
            "inventory: forecast engine",
            "inventory: rollup refresh"
        ]
    );
    assert_eq!(inventory.base.kind, OverviewBaseKind::OlderTrunk);
    assert_eq!(inventory.base.description, "initial change");
    assert_eq!(inventory.base.behind_trunk, 1);
    assert_eq!(billing.base.behind_trunk, 0);
    assert_eq!(inventory.workspaces.len(), 1);
    assert_eq!(inventory.workspaces[0].name, "planner");
    assert_eq!(inventory.workspaces[0].changes_above, 1);
    assert!(!inventory.workspaces[0].is_current);
    assert_eq!(
        overview
            .lanes
            .iter()
            .filter(|lane| lane.is_behind_trunk())
            .count(),
        1
    );

    let bridge = lane_by_head(&overview.lanes, "");
    assert!(bridge.head().is_empty);
    assert_eq!(
        bridge.changes[1].description,
        "notifications: chat integration"
    );
    assert_eq!(bridge.workspaces[0].name, "bridge");
    assert_eq!(
        bridge.attention,
        ["Empty, undescribed checkout above 1 change"]
    );

    let groups = overview_groups(&overview);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].base.kind, OverviewBaseKind::Trunk);
    assert_eq!(groups[1].base.kind, OverviewBaseKind::OlderTrunk);
    let billing_index = overview
        .lanes
        .iter()
        .position(|lane| std::ptr::eq(lane, billing))
        .expect("billing index") as u32;
    assert_eq!(
        groups[0].lanes[0], billing_index,
        "the current workspace leads the trunk group"
    );
}

#[test]
fn overview_splits_lanes_at_a_mutable_fork_point() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["bookmark", "create", "-r", "@", "main"]);
    for (key, value) in [
        ("revset-aliases.'trunk()'", "main"),
        ("revset-aliases.'immutable_heads()'", "main"),
    ] {
        run_jj_in(&repo_path, &["config", "set", "--repo", key, value]);
    }
    describe_new(&repo_path, "main", "shared base");
    describe_new(&repo_path, "@", "left");
    describe_new(&repo_path, "subject(\"shared base\")", "right");

    let repo = Repo::open(&repo_path).expect("open repo");
    let overview = repo.overview().expect("overview");

    assert_eq!(overview.lanes.len(), 3, "{:#?}", overview.lanes);
    let left = lane_by_head(&overview.lanes, "left");
    assert_eq!(left.base.kind, OverviewBaseKind::Mutable);
    assert_eq!(left.base.description, "shared base");
    let shared = lane_by_head(&overview.lanes, "shared base");
    assert_eq!(shared.base.kind, OverviewBaseKind::Trunk);
    let groups = overview_groups(&overview);
    assert_eq!(groups[0].base.kind, OverviewBaseKind::Trunk);
    assert_eq!(groups[1].base.kind, OverviewBaseKind::Mutable);
}

#[test]
fn rebasing_a_lane_root_onto_trunk_moves_the_lane_with_its_checkout() {
    let (_temp_dir, repo_path) = build_fixture();
    let repo = Repo::open(&repo_path).expect("open repo");
    let before = repo.overview().expect("overview");
    let inventory = lane_by_head(&before.lanes, "inventory: nightly rollups");
    let root = inventory.changes.last().expect("root").commit_id.id.clone();

    repo.rebase(&root, "trunk()", RebaseMode::Source)
        .expect("rebase lane root");

    let after = repo.overview().expect("overview after rebase");
    let inventory = lane_by_head(&after.lanes, "inventory: nightly rollups");
    assert_eq!(inventory.base.kind, OverviewBaseKind::Trunk);
    assert_eq!(inventory.changes.len(), 3);
    assert_eq!(inventory.workspaces[0].name, "planner");
    assert_eq!(inventory.workspaces[0].changes_above, 1);
    assert_eq!(
        after
            .lanes
            .iter()
            .filter(|lane| lane.is_behind_trunk())
            .count(),
        0
    );
}

#[test]
fn overview_reflects_operations_made_outside_the_handle() {
    let (_temp_dir, repo_path) = build_fixture();
    let repo = Repo::open(&repo_path).expect("open repo");
    assert_eq!(repo.overview().expect("overview").lanes.len(), 4);

    describe_new(&repo_path, "main", "cli: added elsewhere");

    let lanes = repo.overview().expect("overview after cli change").lanes;
    assert_eq!(lanes.len(), 5, "{lanes:#?}");
    assert!(
        lanes
            .iter()
            .any(|lane| lane.head().description == "cli: added elsewhere")
    );
}
