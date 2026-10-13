use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use tempfile::TempDir;

use crate::cmd::{configure_test_user, init_colocated, run_jj_in};
use crate::template::copy_of;

static TEMPLATE: OnceLock<TempDir> = OnceLock::new();

/// Eight changes on top of `good-base`. `change-5` is the first that contains `bug.txt`.
pub struct BisectStack {
    _tmp: TempDir,
    pub path: PathBuf,
}

impl BisectStack {
    pub fn build() -> Self {
        let tmp = copy_of(&TEMPLATE, build_repo);
        let path = tmp.path().join("repo");
        Self { _tmp: tmp, path }
    }

    pub const GOOD: &'static str = "good-base";
    pub const FIRST_BAD: &'static str = "change-5";
    pub const BAD_END: &'static str = "change-8";
    /// The change immediately before [`Self::FIRST_BAD`].
    pub const BEFORE_BAD: &'static str = "change-4";
}

fn build_repo(path: &Path) {
    init_colocated(path);
    configure_test_user(path);
    fs::write(path.join("base.txt"), "base\n").expect("write base");
    run_jj_in(path, &["describe", "-m", BisectStack::GOOD]);
    for number in 1..=8 {
        run_jj_in(path, &["new", "-m", &format!("change-{number}")]);
        fs::write(
            path.join(format!("c{number}.txt")),
            format!("content {number}\n"),
        )
        .expect("write change file");
        if number == 5 {
            fs::write(path.join("bug.txt"), "bug\n").expect("write bug marker");
        }
    }
    // The last `jj new` is this change, so snapshot its files into the commit the tests bisect.
    run_jj_in(path, &["st"]);
}
