use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use tempfile::TempDir;

use crate::cmd::{configure_test_user, init_colocated, run_jj_in};
use crate::template::copy_of;

static TEMPLATE: OnceLock<TempDir> = OnceLock::new();

/// Linear history: 3 describing changes ending in a working copy with two new
/// files. Mirrors the same-shaped fixture in `shell/justfile:ui-test-setup`
/// so behavior parity with the SwiftUI UI tests is intentional.
pub struct LinearFixture {
    _tmp: TempDir,
    pub path: PathBuf,
}

impl LinearFixture {
    pub fn build() -> Self {
        let tmp = copy_of(&TEMPLATE, build_repo);
        let path = tmp.path().join("repo");
        Self { _tmp: tmp, path }
    }

    pub fn add_tracked_working_copy_edits(&self) {
        fs::write(
            self.path.join("README.md"),
            "# Sample project\nEdited in GPUI test\n",
        )
        .expect("write README.md");
        fs::write(
            self.path.join("feature.txt"),
            "feature\nEdited in GPUI test\n",
        )
        .expect("write feature.txt");
        run_jj_in(&self.path, &["st"]);
    }

    pub fn add_multiline_working_copy_edit(&self) {
        fs::write(
            self.path.join("feature.txt"),
            "second\nthird\nfourth\nfeature\n",
        )
        .expect("write feature.txt");
        run_jj_in(&self.path, &["st"]);
    }

    pub fn add_conflict_marker_working_copy_edit(&self) {
        fs::write(
            self.path.join("feature.txt"),
            "<<<<<<< Conflict\none line\n>>>>>>> Conflict ends\nfeature\n",
        )
        .expect("write feature.txt");
        run_jj_in(&self.path, &["st"]);
    }

    pub fn remove_tracked_working_copy_file(&self, path: &str) {
        fs::remove_file(self.path.join(path)).expect("remove tracked file");
        run_jj_in(&self.path, &["st"]);
    }
}

fn build_repo(path: &Path) {
    init_colocated(path);
    configure_test_user(path);

    write(path, "README.md", "# Sample project\n");
    run_jj_in(path, &["describe", "-m", "initial"]);

    run_jj_in(path, &["new", "-m", "add hello"]);
    write(path, "hello.txt", "hello\n");

    run_jj_in(path, &["new", "-m", "add feature"]);
    write(path, "feature.txt", "feature\n");
    run_jj_in(path, &["bookmark", "create", "main", "-r", "@"]);

    run_jj_in(path, &["new"]);
    write(path, "wip1.txt", "wip 1\n");
    write(path, "wip2.txt", "wip 2\n");
}

fn write(repo: &Path, rel: &str, contents: &str) {
    fs::write(repo.join(rel), contents).unwrap_or_else(|err| panic!("write {rel}: {err}"));
}
