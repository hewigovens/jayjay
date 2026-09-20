use std::fs;
use std::path::PathBuf;

use jj_test::{init_jj_repo, run_git, run_jj_in};
use tempfile::TempDir;

use super::plan::{PrHeadRepo, PullRequestImportPlan, RemoteChoice, ResolvedPullRequest};
use crate::repo::Repo;
use crate::repo::hosted_repo::{HostedRepo, RepoHost};
use crate::types::{CoreError, CoreResult, PrState};

const MISSING_HEAD: &str = "1111111111111111111111111111111111111111";

struct Fixture {
    temp: TempDir,
    repo: Repo,
    fork: PathBuf,
    head: String,
}

impl Fixture {
    fn new() -> Self {
        let temp = init_jj_repo();
        let repo_path = temp.path().join("repo");
        run_jj_in(
            &repo_path,
            &[
                "git",
                "remote",
                "add",
                "origin",
                "https://github.com/owner/base.git",
            ],
        );
        let fork = temp.path().join("fork");
        fs::create_dir(&fork).expect("create fork dir");
        run_git(&fork, &["init"]);
        fs::write(fork.join("feature.txt"), "from the fork\n").expect("write fork file");
        run_git(&fork, &["add", "."]);
        run_git(
            &fork,
            &[
                "-c",
                "user.name=Fork",
                "-c",
                "user.email=fork@example.com",
                "commit",
                "-m",
                "fork change",
            ],
        );
        run_git(&fork, &["branch", "feat/x"]);
        let head = String::from_utf8(run_git(&fork, &["rev-parse", "HEAD"]).stdout)
            .expect("utf8 sha")
            .trim()
            .to_owned();
        let repo = Repo::open(&repo_path).expect("open repo");
        Self {
            temp,
            repo,
            fork,
            head,
        }
    }

    fn path(&self, name: &str) -> String {
        self.temp.path().join(name).to_string_lossy().into_owned()
    }

    fn plan(&self, head_commit_id: &str, remote_exists: bool) -> PullRequestImportPlan {
        let (name, url) = ("alice".to_owned(), self.fork.to_string_lossy().into_owned());
        PullRequestImportPlan {
            base: HostedRepo {
                host: RepoHost::GitHub,
                owner: "owner".into(),
                repo: "base".into(),
            },
            resolved: ResolvedPullRequest {
                number: 1,
                title: "fork change".into(),
                state: PrState::Open,
                url: "https://github.com/owner/base/pull/1".into(),
                head_branch: "feat/x".into(),
                head_commit_id: head_commit_id.to_owned(),
                head: PrHeadRepo::Fork(HostedRepo {
                    host: RepoHost::GitHub,
                    owner: name.clone(),
                    repo: "base".into(),
                }),
            },
            remote: if remote_exists {
                RemoteChoice::Reuse { name, url }
            } else {
                RemoteChoice::Add { name, url }
            },
        }
    }

    fn import(&self, plan: &PullRequestImportPlan, workspace: &str) -> CoreResult<String> {
        self.repo.import_resolved(
            plan,
            workspace,
            &self.path(workspace),
            &self.repo.sync_token(),
        )
    }

    fn remotes(&self) -> String {
        let list = run_jj_in(&self.temp.path().join("repo"), &["git", "remote", "list"]);
        String::from_utf8(list.stdout).expect("utf8 remotes")
    }

    fn parent_of(&self, workspace: &str) -> String {
        let log = run_jj_in(
            &self.temp.path().join(workspace),
            &["log", "--no-graph", "-r", "@-", "-T", "commit_id"],
        );
        String::from_utf8(log.stdout)
            .expect("utf8 log")
            .trim()
            .to_owned()
    }
}

#[test]
fn import_creates_the_workspace_at_the_pr_head_and_reuses_the_remote_next_time() {
    let fixture = Fixture::new();

    let created = fixture
        .import(&fixture.plan(&fixture.head, false), "pr-1")
        .expect("import");
    assert_eq!(created, fixture.path("pr-1"));
    assert_eq!(fixture.parent_of("pr-1"), fixture.head);
    assert!(fixture.temp.path().join("pr-1/feature.txt").exists());

    let created = fixture
        .repo
        .import_resolved(
            &fixture.plan(&fixture.head, true),
            "pr-1-again",
            "pr-1-again",
            &fixture.repo.sync_token(),
        )
        .expect("import again with a relative location");
    let canonical_root = crate::repo::support::canonicalize(fixture.temp.path());
    assert_eq!(created, canonical_root.join("pr-1-again").to_string_lossy());
    assert_eq!(fixture.parent_of("pr-1-again"), fixture.head);
    let remotes = fixture.remotes();
    assert_eq!(remotes.matches("alice").count(), 1, "{remotes}");
}

#[test]
fn failed_fetch_reports_and_keeps_the_added_remote() {
    let mut fixture = Fixture::new();
    fixture.fork = fixture.temp.path().join("missing-fork");

    let error = fixture
        .import(&fixture.plan(MISSING_HEAD, false), "pr-1")
        .expect_err("fetch from a missing remote must fail");
    assert!(
        error.to_string().contains("Remote 'alice' was added"),
        "{error}"
    );
    assert!(fixture.remotes().contains("alice"));

    let error = fixture
        .import(&fixture.plan(MISSING_HEAD, true), "pr-1")
        .expect_err("retry still fails to fetch");
    assert!(!error.to_string().contains("was added"), "{error}");
}

#[cfg(unix)]
#[test]
fn canceling_a_hung_fetch_reports_canceled_even_after_adding_the_remote() {
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Duration, Instant};

    let fixture = Fixture::new();
    let git_started = fixture.temp.path().join("git-started");
    let fake_git = fixture.temp.path().join("git");
    fs::write(
        &fake_git,
        format!("#!/bin/sh\ntouch '{}'\nsleep 30\n", git_started.display()),
    )
    .expect("write fake git");
    fs::set_permissions(&fake_git, fs::Permissions::from_mode(0o755)).expect("chmod fake git");
    run_jj_in(
        &fixture.temp.path().join("repo"),
        &[
            "config",
            "set",
            "--repo",
            "git.executable-path",
            &fake_git.to_string_lossy(),
        ],
    );
    let repo = Repo::open(&fixture.temp.path().join("repo")).expect("reopen repo");
    let sync = repo.sync_token();
    let plan = fixture.plan(MISSING_HEAD, false);
    let dest = fixture.path("pr-1");

    let outcome = std::thread::scope(|scope| {
        let import = scope.spawn(|| repo.import_resolved(&plan, "pr-1", &dest, &sync));
        let deadline = Instant::now() + Duration::from_secs(10);
        while !git_started.exists() {
            assert!(Instant::now() < deadline, "fetch never reached git");
            std::thread::sleep(Duration::from_millis(20));
        }
        sync.cancel();
        import.join().expect("import thread")
    });

    let error = outcome.expect_err("import should be canceled");
    assert!(matches!(error, CoreError::Canceled), "{error}");
}

#[test]
fn import_rejects_a_head_the_fetched_branch_no_longer_has() {
    let fixture = Fixture::new();
    fixture
        .import(&fixture.plan(&fixture.head, false), "pr-0")
        .expect("an earlier import leaves the head local");
    run_git(
        &fixture.fork,
        &[
            "-c",
            "user.name=Fork",
            "-c",
            "user.email=fork@example.com",
            "commit",
            "--amend",
            "--allow-empty",
            "-m",
            "amended",
        ],
    );
    run_git(&fixture.fork, &["branch", "-f", "feat/x"]);
    let stale = fixture.head.clone();

    for (head, branch) in [
        (stale.as_str(), "feat/x"),
        (MISSING_HEAD, "feat/x"),
        (MISSING_HEAD, "deleted"),
    ] {
        let mut plan = fixture.plan(head, true);
        plan.resolved.head_branch = branch.into();
        let error = fixture
            .import(&plan, "pr-1")
            .expect_err("a head missing from the fetched branch must fail");
        assert!(
            error.to_string().contains("no longer contains"),
            "{branch} {head}: {error}"
        );
        assert!(
            !fixture.temp.path().join("pr-1").exists(),
            "{branch} {head}"
        );
    }
}

#[test]
fn a_resolved_pull_request_needs_the_asked_number_and_a_full_head() {
    let fixture = Fixture::new();
    let resolved = |head: &str| fixture.plan(head, false).resolved;

    assert!(resolved(&fixture.head).checked(1).is_some());
    assert!(resolved(&fixture.head).checked(2).is_none());
    assert!(resolved("abc123").checked(1).is_none());
    assert!(resolved(&fixture.head).has_head(&fixture.head.to_uppercase()));
    assert!(!resolved(&fixture.head).has_head(MISSING_HEAD));
}

#[test]
fn import_rejects_a_taken_name_or_occupied_destination_before_adding_the_remote() {
    let fixture = Fixture::new();
    let plan = fixture.plan(&fixture.head, false);
    fs::create_dir(fixture.path("occupied")).expect("create occupied dir");
    fs::write(fixture.temp.path().join("occupied/keep.txt"), "keep").expect("write occupied file");

    for (name, dest) in [
        ("default", fixture.path("pr-1")),
        ("pr-1", fixture.path("occupied")),
        ("pr-1", "occupied".to_owned()),
        ("bad/name", fixture.path("pr-1")),
    ] {
        fixture
            .repo
            .import_resolved(&plan, name, &dest, &fixture.repo.sync_token())
            .expect_err("the workspace must be rejected");
        assert!(!fixture.remotes().contains("alice"), "{name}");
    }
}

#[test]
fn preview_finds_the_workspace_already_built_on_the_pull_request() {
    let fixture = Fixture::new();
    let plan = fixture.plan(&fixture.head, false);
    let existing = |repo: &Repo, plan: &PullRequestImportPlan| {
        repo.existing_pull_request_workspace(plan)
            .expect("lookup")
            .map(|workspace| workspace.name)
    };
    assert_eq!(existing(&fixture.repo, &plan), None);

    fixture.import(&plan, "pr-1").expect("import");
    run_jj_in(
        &fixture.temp.path().join("pr-1"),
        &["commit", "-m", "review fixup"],
    );
    fixture
        .import(&fixture.plan(&fixture.head, true), "pr-1-again")
        .expect("import again");
    fs::remove_dir_all(fixture.temp.path().join("pr-1")).expect("delete the first checkout");
    let repo = Repo::open(&fixture.temp.path().join("repo")).expect("reopen repo");

    let mut moved = fixture.plan(MISSING_HEAD, true);
    assert_eq!(existing(&repo, &plan).as_deref(), Some("pr-1-again"));
    assert_eq!(existing(&repo, &moved).as_deref(), Some("pr-1-again"));
    moved.resolved.head_branch = "other".into();
    assert_eq!(existing(&repo, &moved), None);
}

#[test]
fn workspace_suggestion_skips_taken_names_and_existing_dirs() {
    let fixture = Fixture::new();
    let suggest = || {
        fixture
            .repo
            .suggest_workspace_destination(7)
            .expect("suggest")
    };

    let canonical_root = crate::repo::support::canonicalize(fixture.temp.path());
    assert_eq!(
        suggest(),
        (
            "pr-7".to_owned(),
            canonical_root.join("pr-7").to_string_lossy().into_owned()
        )
    );

    fixture
        .import(&fixture.plan(&fixture.head, false), "pr-7")
        .expect("import");
    assert_eq!(suggest().0, "pr-7-2");

    fs::create_dir(fixture.path("pr-7-2")).expect("create colliding dir");
    assert_eq!(suggest().0, "pr-7-3");
}
