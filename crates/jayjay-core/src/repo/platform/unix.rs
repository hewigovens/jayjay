use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// A regular file with an execute bit. Shared with the shell CLI installers so their notion of "executable" cannot drift from detection's.
pub fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata(path)
        .map(|meta| meta.is_file() && meta.mode() & 0o111 != 0)
        .unwrap_or(false)
}

pub fn executable_names(name: &str) -> Vec<String> {
    vec![name.to_owned()]
}

pub fn system_path_entries() -> Vec<PathBuf> {
    [
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/usr/bin",
        "/bin",
        "/usr/sbin",
        "/sbin",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect()
}

pub fn command(program: OsString) -> Command {
    Command::new(program)
}
