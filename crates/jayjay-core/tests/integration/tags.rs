use jayjay_core::Repo;
use jj_test::{LinearFixture, git_stdout, run_git, run_jj_in};

#[test]
fn created_tag_exports_to_git_pushes_and_its_deletion_pushes() {
    let fixture = LinearFixture::build();
    let remote = fixture.add_bare_origin();
    let repo_path = &fixture.path;
    let bare_path = remote.path();

    let repo = Repo::open(repo_path).expect("open repo");
    repo.create_tag("v1.0", "@-").expect("create tag");
    let tagged = repo.log("@-").expect("log tagged change");
    assert_eq!(tagged[0].tags, ["v1.0"]);
    let tags = repo.list_tags().expect("list tags");
    assert_eq!(tags[0].name, "v1.0");
    assert!(tags[0].tracked_remotes.is_empty(), "not pushed yet");
    assert_eq!(
        git_stdout(repo_path, &["rev-parse", "refs/tags/v1.0"]),
        tagged[0].commit_id.id
    );
    assert!(
        repo.create_tag("v1.0", "@").is_err(),
        "an existing tag must not move"
    );
    assert!(repo.create_tag("bad name", "@-").is_err());

    repo.git_push_tag("v1.0", &repo.sync_token())
        .expect("push tag");
    assert_eq!(
        repo.list_tags().expect("list tags")[0].tracked_remotes,
        ["origin"]
    );
    assert_eq!(
        git_stdout(bare_path, &["rev-parse", "refs/tags/v1.0"]),
        tagged[0].commit_id.id
    );

    repo.delete_tag("v1.0").expect("delete tag");
    assert!(repo.log("@-").expect("log after delete")[0].tags.is_empty());
    assert!(
        repo.create_tag("v1.0", "@").is_err(),
        "pending remote deletion reserves the name"
    );
    repo.git_push_tag("v1.0", &repo.sync_token())
        .expect("push tag deletion");
    repo.create_tag("v1.0", "@")
        .expect("reuse name after remote deletion");
    assert!(git_stdout(bare_path, &["tag", "--list"]).is_empty());
}

#[test]
fn tag_deletion_pushes_to_the_remotes_that_track_it() {
    let fixture = LinearFixture::build();
    let origin = fixture.add_bare_origin();
    let upstream = fixture.add_bare_remote("upstream");
    let repo = Repo::open(&fixture.path).unwrap();
    repo.create_tag("v2.0", "@-").unwrap();
    run_jj_in(
        &fixture.path,
        &["git", "push", "--remote", "upstream", "--tag", "v2.0"],
    );
    let repo = Repo::open(&fixture.path).unwrap();
    assert_eq!(repo.list_tags().unwrap()[0].tracked_remotes, ["upstream"]);

    repo.delete_tag_and_push("v2.0", &repo.sync_token())
        .unwrap();
    assert!(git_stdout(upstream.path(), &["tag", "--list"]).is_empty());
    assert!(git_stdout(origin.path(), &["tag", "--list"]).is_empty());
    assert!(repo.list_tags().unwrap().is_empty());
}

#[test]
fn failed_tag_deletion_push_restores_the_tag_for_retry() {
    let fixture = LinearFixture::build();
    let remote = fixture.add_bare_origin();
    let repo = Repo::open(&fixture.path).unwrap();
    repo.create_tag("release", "@-").unwrap();
    repo.git_push_tag("release", &repo.sync_token()).unwrap();
    let target = git_stdout(&fixture.path, &["rev-parse", "refs/tags/release"]);
    let unavailable = remote.path().join("missing.git");
    run_git(
        &fixture.path,
        &["remote", "set-url", "origin", unavailable.to_str().unwrap()],
    );
    assert!(
        repo.delete_tag_and_push("release", &repo.sync_token())
            .is_err()
    );
    assert_eq!(
        git_stdout(&fixture.path, &["rev-parse", "refs/tags/release"]),
        target
    );
    assert_eq!(repo.log("tags()").unwrap()[0].tags, ["release"]);
    assert_eq!(
        git_stdout(remote.path(), &["rev-parse", "refs/tags/release"]),
        target
    );
    run_git(
        &fixture.path,
        &[
            "remote",
            "set-url",
            "origin",
            remote.path().to_str().unwrap(),
        ],
    );
    repo.delete_tag_and_push("release", &repo.sync_token())
        .unwrap();
    assert!(git_stdout(&fixture.path, &["tag", "--list"]).is_empty());
    assert!(git_stdout(remote.path(), &["tag", "--list"]).is_empty());
}

#[test]
fn tag_creation_rejects_a_rewritten_rendered_commit() {
    let fixture = LinearFixture::build();
    let repo = Repo::open(&fixture.path).unwrap();
    let rendered = repo.log("@").unwrap()[0].commit_id.id.clone();
    repo.describe("@", "rewritten after opening the tag dialog")
        .unwrap();
    let error = repo.create_tag("release", &rendered).unwrap_err();
    assert!(error.to_string().contains("reselect"), "{error}");
    assert!(repo.log("tags()").unwrap().is_empty());
    let current = repo.log("@").unwrap()[0].commit_id.id.clone();
    repo.create_tag("release", &current).unwrap();
    assert_eq!(
        git_stdout(&fixture.path, &["rev-parse", "refs/tags/release"]),
        current
    );
}
