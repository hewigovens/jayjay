use std::collections::HashMap;
use std::path::Path;

use jj_lib::config::ConfigGetResultExt as _;
use jj_lib::git::{self, GitImportOptions};
use jj_lib::ref_name::RemoteNameBuf;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::revset::{self, RevsetDiagnostics};
use jj_lib::settings::UserSettings;
use jj_lib::str_util::StringMatcher;
use jj_lib::workspace::Workspace;

use super::config::ConfigEnv;
use super::support::block_on_result;
use crate::types::*;

const WORKTREE_INIT_ERROR: &str = "Cannot create a colocated jj repo inside a Git worktree.\n\
Hint: Run `jj git init` in the main Git repository instead, or use `jj workspace add` to create additional jj workspaces.";

const BARE_REPO_INIT_ERROR: &str =
    "Cannot create a colocated jj repo inside a bare Git repository.";

pub fn init_jj_git_repo(path: &Path) -> CoreResult<()> {
    init_jj_git_repo_with_env(path, &ConfigEnv::from_environment())
}

fn init_jj_git_repo_with_env(path: &Path, env: &ConfigEnv) -> CoreResult<()> {
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| CoreError::Internal {
            message: format!("jj git init: Failed to create workspace: {e}"),
        })?;
    }

    let settings = env.settings_for_new_workspace(path)?;

    if let Ok(git_repo) = gix::open(path) {
        if git_repo.is_bare() {
            return Err(CoreError::Internal {
                message: BARE_REPO_INIT_ERROR.to_owned(),
            });
        }
        if git_repo.git_dir() != git_repo.common_dir() {
            return Err(CoreError::Internal {
                message: WORKTREE_INIT_ERROR.to_owned(),
            });
        }
        init_external(path, git_repo.git_dir(), &settings)?;
    } else if path.join(".git").exists() {
        init_external(path, &path.join(".git"), &settings)?;
    } else {
        let object_hash = object_hash_from_settings(&settings)?;
        block_on_result(
            "jj git init",
            Workspace::init_colocated_git(&settings, path, object_hash),
        )?;
    }
    write_jj_gitignore(path)?;
    Ok(())
}

fn init_external(
    workspace_root: &Path,
    git_repo_path: &Path,
    settings: &UserSettings,
) -> CoreResult<()> {
    let import_options = GitImportOptions {
        abandon_unreachable_commits: false,
        record_synthetic_predecessors: false,
        remote_auto_track_bookmarks: remote_auto_track_bookmarks(settings)?,
    };
    let (mut workspace, repo) = block_on_result(
        "jj git init",
        Workspace::init_external_git(settings, workspace_root, git_repo_path),
    )?;
    let mut tx = repo.start_transaction();
    block_on_result(
        "import git refs",
        git::import_refs(tx.repo_mut(), &import_options),
    )?;
    if let Some(git_head) =
        block_on_result("import git head", git::import_head_commit(tx.repo_mut()))?
    {
        block_on_result(
            "check out git head",
            tx.repo_mut()
                .check_out(workspace.workspace_name().to_owned(), &git_head),
        )?;
    }
    block_on_result("rebase descendants", tx.repo_mut().rebase_descendants())?;
    if !tx.repo().has_changes() {
        return Ok(());
    }
    git::export_refs(tx.repo_mut())
        .map_err(|e| Error::internal(format!("export git refs: {e}")))?;
    let repo = block_on_result("import git refs", tx.commit("import git refs"))?;
    sync_working_copy(&mut workspace, &repo)?;
    Ok(())
}

fn sync_working_copy(workspace: &mut Workspace, repo: &ReadonlyRepo) -> CoreResult<()> {
    let wc_commit_id = repo
        .view()
        .get_wc_commit_id(workspace.workspace_name())
        .ok_or_else(|| Error::internal("workspace has no working-copy commit"))?;
    let wc_commit = repo
        .store()
        .get_commit(wc_commit_id)
        .map_err(|e| Error::internal(format!("load working-copy commit: {e}")))?;
    let mut locked_ws =
        block_on_result("lock working copy", workspace.start_working_copy_mutation())?;
    block_on_result(
        "reset working copy",
        locked_ws.locked_wc().recover(&wc_commit),
    )?;
    block_on_result(
        "finish working copy",
        locked_ws.finish(repo.op_id().clone()),
    )?;
    Ok(())
}

fn write_jj_gitignore(workspace_root: &Path) -> CoreResult<()> {
    std::fs::write(workspace_root.join(".jj").join(".gitignore"), "/*\n")
        .map_err(|e| Error::internal(format!("Failed to write .jj/.gitignore file: {e}")))
}

fn remote_auto_track_bookmarks(
    settings: &UserSettings,
) -> CoreResult<HashMap<RemoteNameBuf, StringMatcher>> {
    let remotes = settings.remote_settings().map_err(Error::internal)?;
    let mut matchers = HashMap::new();
    for (name, remote) in remotes {
        let Some(text) = remote.auto_track_bookmarks else {
            continue;
        };
        let mut diagnostics = RevsetDiagnostics::default();
        let expr = revset::parse_string_expression(&mut diagnostics, &text).map_err(|err| {
            Error::internal(format!(
                "remotes.{}.auto-track-bookmarks: {err}",
                name.as_str()
            ))
        })?;
        matchers.insert(name, expr.to_matcher());
    }
    Ok(matchers)
}

fn object_hash_from_settings(settings: &UserSettings) -> CoreResult<gix::hash::Kind> {
    match settings.get_string("git.object-hash").optional() {
        Ok(None) => Ok(gix::hash::Kind::Sha1),
        Ok(Some(value)) => match value.as_str() {
            "sha1" => Ok(gix::hash::Kind::Sha1),
            "sha256" => Ok(gix::hash::Kind::Sha256),
            other => Err(Error::internal(format!(
                "Invalid type or value for git.object-hash: {other}"
            ))),
        },
        Err(error) => Err(Error::internal(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use crate::Repo;
    use jj_test::run_git;

    #[test]
    fn initializes_jj_git_repo() {
        let tmp = tempfile::tempdir().expect("tempdir");
        init_jj_git_repo(tmp.path()).expect("init repo");

        assert!(tmp.path().join(".jj").exists());
        assert!(tmp.path().join(".git").exists());
        assert_eq!(
            fs::read_to_string(tmp.path().join(".jj").join(".gitignore")).expect("gitignore"),
            "/*\n"
        );
        Repo::open(tmp.path()).expect("open initialized repo");
    }

    #[test]
    fn initializes_over_existing_git_repo() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path();
        git_repo_with_commit(path);

        init_jj_git_repo(path).expect("init over git");
        assert!(path.join(".jj").exists());

        let repo = Repo::open(path).expect("open initialized repo");
        let changes = repo.log("all()").expect("log");
        assert!(
            changes
                .iter()
                .any(|change| change.description.contains("init")),
            "imported git history: {changes:?}"
        );

        let working_copy = repo.show("@").expect("show working copy");
        assert!(
            working_copy.info.is_empty,
            "working copy should match imported Git HEAD"
        );
        let parent = repo.show("@-").expect("show parent");
        assert!(
            parent.info.description.contains("init"),
            "working copy parent: {}",
            parent.info.description
        );
    }

    #[test]
    fn refresh_after_init_keeps_tracked_gitignored_paths() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path();
        git_repo_with_commit(path);
        fs::write(path.join("secret.txt"), "secret\n").expect("secret");
        run_git(path, &["add", "secret.txt"]);
        fs::write(path.join(".gitignore"), "secret.txt\n").expect("gitignore");
        run_git(path, &["add", ".gitignore"]);
        run_git(
            path,
            &[
                "-c",
                "user.name=Test User",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-m",
                "track ignored secret",
            ],
        );

        init_jj_git_repo(path).expect("init over git");
        let repo = Repo::open(path).expect("open initialized repo");
        repo.refresh_working_copy().expect("refresh working copy");

        let working_copy = repo.show("@").expect("show working copy");
        assert!(
            working_copy.info.is_empty,
            "refresh should not delete tracked gitignored paths: {working_copy:?}"
        );
        assert!(path.join("secret.txt").exists());
    }

    #[test]
    fn refresh_after_init_preserves_uncommitted_git_deletions() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path();
        git_repo_with_commit(path);
        fs::remove_file(path.join("README.md")).expect("delete tracked file");

        init_jj_git_repo(path).expect("init over git");
        assert!(
            !path.join("README.md").exists(),
            "init must not restore the deletion"
        );

        let repo = Repo::open(path).expect("open initialized repo");
        repo.refresh_working_copy().expect("refresh working copy");
        assert!(!path.join("README.md").exists());
        let working_copy = repo.show("@").expect("show working copy");
        assert!(
            working_copy
                .diff
                .iter()
                .any(|hunk| hunk.path == "README.md"),
            "refresh should keep the uncommitted deletion: {working_copy:?}"
        );
    }

    #[test]
    fn invalid_auto_track_expression_does_not_create_jj_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path();
        let home = tmp.path().join("home");
        let config_dir = tmp.path().join("config");
        fs::create_dir(&home).expect("home");
        fs::create_dir_all(config_dir.join("jj")).expect("config");
        git_repo_with_commit(path);
        fs::write(
            config_dir.join("jj").join("config.toml"),
            "[remotes.origin]\nauto-track-bookmarks = \"(\"\n",
        )
        .expect("write remotes config");

        init_jj_git_repo_with_env(
            path,
            &ConfigEnv::new(
                Some(home),
                Some(config_dir),
                None,
                "test-host".to_owned(),
                HashMap::new(),
            ),
        )
        .expect_err("invalid auto-track expression");
        assert!(!path.join(".jj").exists());
    }

    #[test]
    fn initializes_over_separate_git_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("work");
        let git_dir = tmp.path().join("gitdir");
        fs::create_dir(&path).expect("work");
        run_git(
            &path,
            &[
                "init",
                "--separate-git-dir",
                git_dir.to_str().expect("git dir"),
            ],
        );
        fs::write(path.join("README.md"), "hello\n").expect("write readme");
        run_git(&path, &["add", "."]);
        run_git(
            &path,
            &[
                "-c",
                "user.name=Test User",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-m",
                "init",
            ],
        );

        init_jj_git_repo(&path).expect("init over separate-git-dir");
        assert!(path.join(".jj").exists());
        Repo::open(&path).expect("open initialized repo");
    }

    #[test]
    fn tracks_remote_bookmarks_from_user_auto_track_settings() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let origin = tmp.path().join("origin.git");
        let local = tmp.path().join("local");
        let home = tmp.path().join("home");
        let config_dir = tmp.path().join("config");
        fs::create_dir(&origin).expect("origin");
        fs::create_dir(&local).expect("local");
        fs::create_dir(&home).expect("home");
        fs::create_dir_all(config_dir.join("jj")).expect("config");
        run_git(&origin, &["init", "--bare", "-b", "main"]);
        git_repo_with_commit(&local);
        run_git(
            &local,
            &[
                "remote",
                "add",
                "origin",
                origin.to_str().expect("origin path"),
            ],
        );
        run_git(&local, &["push", "-u", "origin", "HEAD"]);
        fs::write(
            config_dir.join("jj").join("config.toml"),
            "[remotes.origin]\nauto-track-bookmarks = \"glob:*\"\n",
        )
        .expect("write remotes config");

        init_jj_git_repo_with_env(
            &local,
            &ConfigEnv::new(
                Some(home),
                Some(config_dir),
                None,
                "test-host".to_owned(),
                HashMap::new(),
            ),
        )
        .expect("init over git with remotes");

        let repo = Repo::open(&local).expect("open initialized repo");
        let bookmarks = repo.list_bookmarks().expect("bookmarks");
        let bookmark = bookmarks
            .iter()
            .find(|bookmark| {
                bookmark
                    .available_remotes
                    .iter()
                    .any(|remote| remote == "origin")
            })
            .unwrap_or_else(|| panic!("imported origin bookmark: {bookmarks:?}"));
        assert!(
            bookmark
                .tracked_remotes
                .iter()
                .any(|remote| remote == "origin"),
            "auto-track should mark origin as tracked: {bookmark:?}"
        );
    }

    #[test]
    fn initializes_with_configured_sha256_object_hash() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("repo");
        let home = tmp.path().join("home");
        let config_dir = tmp.path().join("config");
        fs::create_dir(&path).expect("repo");
        fs::create_dir(&home).expect("home");
        fs::create_dir_all(config_dir.join("jj")).expect("config");
        fs::write(
            config_dir.join("jj").join("config.toml"),
            "git.object-hash = \"sha256\"\n",
        )
        .expect("write object-hash");

        init_jj_git_repo_with_env(
            &path,
            &ConfigEnv::new(
                Some(home),
                Some(config_dir),
                None,
                "test-host".to_owned(),
                HashMap::new(),
            ),
        )
        .expect("init sha256 repo");
        assert_eq!(
            String::from_utf8_lossy(&run_git(&path, &["rev-parse", "--show-object-format"]).stdout)
                .trim(),
            "sha256"
        );
    }

    #[test]
    fn rejects_invalid_object_hash_without_creating_jj_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("repo");
        let home = tmp.path().join("home");
        let config_dir = tmp.path().join("config");
        fs::create_dir(&path).expect("repo");
        fs::create_dir(&home).expect("home");
        fs::create_dir_all(config_dir.join("jj")).expect("config");
        fs::write(
            config_dir.join("jj").join("config.toml"),
            "git.object-hash = \"sha512\"\n",
        )
        .expect("write invalid object-hash");

        let err = init_jj_git_repo_with_env(
            &path,
            &ConfigEnv::new(
                Some(home),
                Some(config_dir),
                None,
                "test-host".to_owned(),
                HashMap::new(),
            ),
        )
        .expect_err("invalid object-hash");
        let CoreError::Internal { message } = err else {
            panic!("unexpected error kind");
        };
        assert!(message.contains("git.object-hash"));
        assert!(message.contains("sha512"));
        assert!(!path.join(".jj").exists());
        assert!(!path.join(".git").exists());
    }

    #[test]
    fn rejects_bare_git_repo_without_nesting() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("bare.git");
        fs::create_dir(&path).expect("bare");
        run_git(&path, &["init", "--bare", "-b", "main"]);

        let err = init_jj_git_repo(&path).expect_err("init bare repo should fail");
        let CoreError::Internal { message } = err else {
            panic!("unexpected error kind");
        };
        assert!(message.contains("bare Git repository"));
        assert!(!path.join(".jj").exists());
        assert!(!path.join(".git").exists());
    }

    #[test]
    fn preserves_git_worktree_init_error() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let main = tmp.path().join("main");
        let worktree = tmp.path().join("worktree");
        fs::create_dir(&main).expect("create main repo");
        git_repo_with_commit(&main);
        run_git(
            &main,
            &["worktree", "add", worktree.to_str().expect("worktree path")],
        );

        let err = init_jj_git_repo(&worktree).expect_err("init worktree should fail");
        let CoreError::Internal { message } = err else {
            panic!("unexpected error kind");
        };
        assert!(message.contains("Cannot create a colocated jj repo inside a Git worktree"));
        assert!(message.contains("Run `jj git init` in the main Git repository"));
        assert!(!worktree.join(".jj").exists());
    }

    fn git_repo_with_commit(path: &Path) {
        run_git(path, &["init"]);
        fs::write(path.join("README.md"), "hello\n").expect("write readme");
        run_git(path, &["add", "."]);
        run_git(
            path,
            &[
                "-c",
                "user.name=Test User",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-m",
                "init",
            ],
        );
    }
}
