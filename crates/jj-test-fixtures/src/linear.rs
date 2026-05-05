use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::cmd::{configure_test_user, init_colocated, run_jj_in};

/// Linear history: 3 describing changes ending in a working copy with two new
/// files. Mirrors the same-shaped fixture in `shell/justfile:ui-test-setup`
/// so behavior parity with the SwiftUI UI tests is intentional.
pub struct LinearFixture {
    _tmp: TempDir,
    pub path: PathBuf,
}

impl LinearFixture {
    pub fn build() -> Self {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let path = tmp.path().join("repo");

        init_colocated(&path);
        configure_test_user(&path);

        write(&path, "README.md", "# Sample project\n");
        run_jj_in(&path, &["describe", "-m", "initial"]);

        run_jj_in(&path, &["new", "-m", "add hello"]);
        write(&path, "hello.txt", "hello\n");

        run_jj_in(&path, &["new", "-m", "add feature"]);
        write(&path, "feature.txt", "feature\n");
        run_jj_in(&path, &["bookmark", "create", "main", "-r", "@"]);

        run_jj_in(&path, &["new"]);
        write(&path, "wip1.txt", "wip 1\n");
        write(&path, "wip2.txt", "wip 2\n");

        Self { _tmp: tmp, path }
    }
}

fn write(repo: &Path, rel: &str, contents: &str) {
    fs::write(repo.join(rel), contents).unwrap_or_else(|err| panic!("write {rel}: {err}"));
}
