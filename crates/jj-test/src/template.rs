use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use tempfile::TempDir;

/// Builds a fixture repo once per test process and hands out directory copies: every `jj` process the builders spawn costs tens of milliseconds on Linux and hundreds on Windows, while a copy of a tiny repo costs almost nothing. jj notices the copied config-id on first use and gives the copy its own repo config, so `--repo` settings carry over.
pub(crate) fn copy_of(template: &'static OnceLock<TempDir>, build: fn(&Path)) -> TempDir {
    let template = template.get_or_init(|| {
        let tmp = tempfile::tempdir().expect("create template tempdir");
        build(&tmp.path().join("repo"));
        tmp
    });
    let tmp = tempfile::tempdir().expect("create tempdir");
    copy_dir(&template.path().join("repo"), &tmp.path().join("repo"));
    tmp
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir(to).unwrap_or_else(|err| panic!("create {}: {err}", to.display()));
    let entries = fs::read_dir(from).unwrap_or_else(|err| panic!("read {}: {err}", from.display()));
    for entry in entries {
        let entry = entry.expect("read fixture entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("fixture entry type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target)
                .unwrap_or_else(|err| panic!("copy {}: {err}", target.display()));
        }
    }
}
