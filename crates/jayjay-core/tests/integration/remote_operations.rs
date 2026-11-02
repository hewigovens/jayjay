#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::thread;
use std::time::{Duration, Instant};

use std::path::{Path, PathBuf};

use jayjay_core::{JayError, Repo};
use jj_test::env::ScopedEnv;
use jj_test::{
    find_bookmark, first_change, git_stdout, init_jj_repo, repo_with_remote, run_git, run_jj_in,
    scoped_env,
};
use tempfile::TempDir;

#[test]
fn canceling_the_sync_token_interrupts_a_fetch() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let git_started = temp_dir.path().join("git-started");
    let fake_git = temp_dir.path().join("git");
    fs::write(
        &fake_git,
        format!("#!/bin/sh\ntouch '{}'\nsleep 30\n", git_started.display()),
    )
    .expect("write fake git");
    fs::set_permissions(&fake_git, fs::Permissions::from_mode(0o755)).expect("chmod fake git");
    run_jj_in(
        &repo_path,
        &[
            "git",
            "remote",
            "add",
            "origin",
            "https://example.invalid/repo.git",
        ],
    );
    run_jj_in(
        &repo_path,
        &[
            "config",
            "set",
            "--repo",
            "git.executable-path",
            fake_git.to_str().expect("utf-8 path"),
        ],
    );
    let repo = Repo::open(&repo_path).expect("open repo");
    let sync = repo.sync_token();

    let outcome = thread::scope(|scope| {
        let fetch = scope.spawn(|| repo.git_fetch(&sync));
        let deadline = Instant::now() + Duration::from_secs(10);
        while !git_started.exists() {
            assert!(Instant::now() < deadline, "fetch never reached git");
            thread::sleep(Duration::from_millis(20));
        }
        sync.cancel();
        fetch.join().expect("fetch thread")
    });

    let error = outcome.expect_err("fetch should be canceled");
    assert!(matches!(error, JayError::Canceled), "{error}");
}

fn upload_title(repo: &Repo) -> Option<String> {
    repo.repository_context().expect("context").upload_title
}

fn set_repo_config(repo_path: &Path, key: &str, value: &str) {
    run_jj_in(repo_path, &["config", "set", "--repo", key, value]);
}

#[test]
fn pulling_a_bookmark_tracks_it_on_the_remote_it_came_from() {
    let (dir, repo_path, bare) = repo_with_remote("upstream");
    let main = git_stdout(&bare, &["rev-parse", "refs/heads/main"]);
    run_git(&bare, &["update-ref", "refs/heads/feature", main.trim()]);
    let repo = Repo::open(&repo_path).expect("open repo");
    repo.git_pull_bookmark("feature", &repo.sync_token())
        .expect("pull feature");
    assert_eq!(
        find_bookmark(&repo, "feature").tracked_remotes,
        ["upstream"]
    );

    let fork = dir.path().join("fork.git");
    run_git(dir.path(), &["init", "--bare", fork.to_str().unwrap()]);
    run_jj_in(
        &repo_path,
        &["git", "remote", "add", "fork", fork.to_str().unwrap()],
    );
    for (key, remote) in [("git.fetch", "upstream"), ("git.push", "fork")] {
        run_jj_in(&repo_path, &["config", "set", "--repo", key, remote]);
    }
    run_git(
        &fork,
        &[
            "fetch",
            "-q",
            bare.to_str().unwrap(),
            "main:refs/heads/other",
        ],
    );
    run_jj_in(&repo_path, &["git", "fetch", "--remote", "fork"]);
    let tree = git_stdout(&bare, &["rev-parse", "main^{tree}"]);
    let upstream_other = git_stdout(
        &bare,
        &[
            "-c",
            "user.name=Upstream",
            "-c",
            "user.email=upstream@example.com",
            "commit-tree",
            tree.trim(),
            "-p",
            main.trim(),
            "-m",
            "upstream other",
        ],
    );
    run_git(
        &bare,
        &["update-ref", "refs/heads/other", upstream_other.trim()],
    );
    let repo = Repo::open(&repo_path).expect("reopen repo");
    repo.git_pull_bookmark("other", &repo.sync_token())
        .expect("pull other");
    let other = find_bookmark(&repo, "other");
    assert_eq!(other.tracked_remotes, ["upstream"]);
    assert_eq!(other.description.trim(), "upstream other");
}

#[test]
fn a_configured_sync_command_replaces_fetch_and_rebase() {
    let (_dir, repo_path, _bare) = repo_with_remote("origin");
    set_repo_config(
        &repo_path,
        "jayjay.sync-command",
        r#"["describe", "-m", "synced"]"#,
    );
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.git_fetch(&repo.sync_token()).expect("sync");

    assert_eq!(first_change(&repo, "@").description.trim(), "synced");
}

#[test]
fn a_configured_upload_command_runs_on_the_chosen_change() {
    let (_dir, repo_path, _bare) = repo_with_remote("origin");
    set_repo_config(
        &repo_path,
        "jayjay.upload-command",
        r#"["describe", "-r", "$revision", "-m", "uploaded"]"#,
    );
    run_jj_in(&repo_path, &["describe", "-m", "work"]);
    run_jj_in(&repo_path, &["new"]);
    let repo = Repo::open(&repo_path).expect("open repo");
    let work = first_change(&repo, "@-");

    assert_eq!(upload_title(&repo).as_deref(), Some("Upload Change"));
    repo.upload(&work.change_id.id, &repo.sync_token())
        .expect("upload");

    assert_eq!(first_change(&repo, "@-").description.trim(), "uploaded");
}

#[test]
fn a_gerrit_repo_uploads_to_the_branch_trunk_tracks() {
    let (_dir, repo_path, bare) = repo_with_remote("origin");
    assert_eq!(
        upload_title(&Repo::open(&repo_path).expect("open repo")),
        None
    );
    set_repo_config(&repo_path, "gerrit.default-remote", "origin");
    fs::write(repo_path.join("work.txt"), "work\n").expect("write work");
    run_jj_in(&repo_path, &["describe", "-m", "work"]);
    let repo = Repo::open(&repo_path).expect("open repo");
    assert_eq!(upload_title(&repo).as_deref(), Some("Upload to Gerrit"));

    repo.upload("@", &repo.sync_token()).expect("upload");

    let refs = git_stdout(&bare, &["for-each-ref", "--format=%(refname)", "refs/for"]);
    assert_eq!(refs.trim(), "refs/for/main");
}

#[test]
fn uploading_refuses_to_undo_a_newer_gerrit_patch_set() {
    let gerrit = FakeGerrit::with_uploaded_change();
    gerrit.edit_in_gerrit("edited in Gerrit\n", "work, edited");

    let error = gerrit.repo.upload("@", &gerrit.repo.sync_token());

    let Err(JayError::Refused { message }) = error else {
        panic!("upload replaced Gerrit's patch set: {error:?}");
    };
    assert!(message.contains("patch set 2"), "{message}");
}

#[test]
fn update_from_gerrit_takes_the_newer_patch_set_and_allows_the_next_upload() {
    let gerrit = FakeGerrit::with_uploaded_change();
    gerrit.edit_in_gerrit("edited in Gerrit\n", "work, edited");
    let repo = &gerrit.repo;

    let message = repo
        .update_from_gerrit("@", &repo.sync_token())
        .expect("update");

    assert!(message.contains("patch set 2"), "{message}");
    assert_eq!(
        fs::read_to_string(gerrit.path.join("work.txt")).unwrap(),
        "edited in Gerrit\n"
    );
    assert!(
        first_change(repo, "@")
            .description
            .starts_with("work, edited")
    );
    assert!(repo.log("divergent()").unwrap().is_empty());
    let again = repo
        .update_from_gerrit("@", &repo.sync_token())
        .expect("update again");
    assert!(again.contains("already matches"), "{again}");
    repo.upload("@", &repo.sync_token())
        .expect("upload after update");
}

#[test]
fn update_from_gerrit_keeps_what_the_change_inherits_from_its_local_parent() {
    let gerrit = FakeGerrit::with_uploaded_change();
    let repo = &gerrit.repo;
    run_jj_in(&gerrit.path, &["new", "-B", "@", "-m", "parent"]);
    fs::write(gerrit.path.join("parent.txt"), "uploaded\n").unwrap();
    run_jj_in(&gerrit.path, &["edit", "children(@)"]);
    repo.upload("@", &repo.sync_token())
        .expect("upload the stack");
    gerrit.edit_in_gerrit("edited in Gerrit\n", "work, edited");
    run_jj_in(&gerrit.path, &["edit", "@-"]);
    fs::write(gerrit.path.join("parent.txt"), "edited here\n").unwrap();
    run_jj_in(&gerrit.path, &["edit", "children(@)"]);

    let message = repo
        .update_from_gerrit("@", &repo.sync_token())
        .expect("update");

    assert!(message.contains("patch set 3"), "{message}");
    for (file, contents) in [
        ("work.txt", "edited in Gerrit\n"),
        ("parent.txt", "edited here\n"),
    ] {
        assert_eq!(
            fs::read_to_string(gerrit.path.join(file)).unwrap(),
            contents
        );
    }
    assert_eq!(first_change(repo, "@-").description.trim(), "parent");
}

#[test]
fn uploads_to_another_gerrit_server_are_not_mixed_up() {
    let gerrit = FakeGerrit::with_uploaded_change();
    let other = gerrit.dir.path().join("other.git");
    run_git(
        gerrit.dir.path(),
        &["init", "-q", "--bare", other.to_str().unwrap()],
    );
    run_jj_in(
        &gerrit.path,
        &[
            "git",
            "remote",
            "set-url",
            "origin",
            "--push",
            other.to_str().unwrap(),
        ],
    );

    let error = gerrit
        .repo
        .update_from_gerrit("@", &gerrit.repo.sync_token());

    let Err(JayError::Refused { message }) = error else {
        panic!("update used the other server's change: {error:?}");
    };
    assert!(message.contains("hasn’t been uploaded"), "{message}");
}

#[test]
fn update_from_gerrit_refuses_to_drop_local_edits() {
    let edits: [(&str, &str); 2] = [("edited here\n", "work"), ("work\n", "work, reworded here")];
    for (contents, message) in edits {
        let gerrit = FakeGerrit::with_uploaded_change();
        gerrit.edit_in_gerrit("edited in Gerrit\n", "work, edited");
        fs::write(gerrit.path.join("work.txt"), contents).unwrap();
        run_jj_in(&gerrit.path, &["describe", "-m", message]);

        let error = gerrit
            .repo
            .update_from_gerrit("@", &gerrit.repo.sync_token());

        assert!(matches!(error, Err(JayError::Refused { .. })), "{error:?}");
        assert_eq!(
            fs::read_to_string(gerrit.path.join("work.txt")).unwrap(),
            contents
        );
        assert_eq!(first_change(&gerrit.repo, "@").description.trim(), message);
    }
}

/// A bare remote, pushed to through a fetch mirror, whose post-receive hook answers pushes to `refs/for/*` like Gerrit: each becomes the next patch set of change 1, and the push output links it.
struct FakeGerrit {
    repo: Repo,
    path: PathBuf,
    bare: PathBuf,
    _uploads: ScopedEnv,
    dir: TempDir,
}

impl FakeGerrit {
    fn with_uploaded_change() -> Self {
        let (dir, path, bare) = repo_with_remote("origin");
        let mirror = dir.path().join("mirror.git");
        run_git(
            dir.path(),
            &[
                "clone",
                "-q",
                "--mirror",
                bare.to_str().unwrap(),
                mirror.to_str().unwrap(),
            ],
        );
        run_jj_in(
            &path,
            &[
                "git",
                "remote",
                "set-url",
                "origin",
                mirror.to_str().unwrap(),
                "--push",
                bare.to_str().unwrap(),
            ],
        );
        let uploads = dir.path().join("gerrit_uploads.json");
        let uploads = scoped_env([("JAYJAY_GERRIT_UPLOADS_PATH", Some(uploads.as_os_str()))]);
        let hook = bare.join("hooks/post-receive");
        fs::write(
            &hook,
            "#!/bin/sh\n\
             while read old new ref; do\n\
               case \"$ref\" in refs/for/*)\n\
                 n=$(git for-each-ref --format=x refs/changes/01/1/ | wc -l)\n\
                 git update-ref \"refs/changes/01/1/$((n + 1))\" \"$new\"\n\
                 git update-ref -d \"$ref\"\n\
                 echo '  http://gerrit.test/c/demo/+/1 work' >&2;;\n\
               esac\n\
             done\n",
        )
        .unwrap();
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
        set_repo_config(&path, "gerrit.default-remote", "origin");
        fs::write(path.join("work.txt"), "work\n").unwrap();
        run_jj_in(&path, &["describe", "-m", "work"]);
        let repo = Repo::open(&path).expect("open repo");
        repo.upload("@", &repo.sync_token()).expect("upload");
        Self {
            repo,
            path,
            bare,
            _uploads: uploads,
            dir,
        }
    }

    /// An edit in Gerrit's web UI: a new patch set amended from the latest one.
    fn edit_in_gerrit(&self, contents: &str, message: &str) {
        let clone = self.dir.path().join("gerrit-edit");
        run_git(
            self.dir.path(),
            &[
                "clone",
                "-q",
                self.bare.to_str().unwrap(),
                clone.to_str().unwrap(),
            ],
        );
        let latest = git_stdout(
            &self.bare,
            &["for-each-ref", "--format=x", "refs/changes/01/1/"],
        )
        .lines()
        .count();
        run_git(
            &clone,
            &[
                "fetch",
                "-q",
                "origin",
                &format!("refs/changes/01/1/{latest}"),
            ],
        );
        run_git(&clone, &["checkout", "-q", "FETCH_HEAD"]);
        fs::write(clone.join("work.txt"), contents).unwrap();
        run_git(
            &clone,
            &[
                "-c",
                "user.name=Reviewer",
                "-c",
                "user.email=reviewer@example.com",
                "commit",
                "-q",
                "-a",
                "--amend",
                "-m",
                message,
            ],
        );
        run_git(
            &clone,
            &[
                "push",
                "-q",
                "origin",
                &format!("HEAD:refs/changes/01/1/{}", latest + 1),
            ],
        );
    }
}
