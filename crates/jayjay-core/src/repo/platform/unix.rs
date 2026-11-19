use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// A regular file with an execute bit. Shared with the shell CLI installers so their notion of "executable" cannot drift from detection's.
pub fn is_executable_file(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|meta| meta.is_file() && meta.mode() & 0o111 != 0)
        .unwrap_or(false)
}

pub fn launch_target(path: PathBuf) -> PathBuf {
    path
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

pub fn hostname() -> OsString {
    let mut buffer = [0u8; 256];
    // SAFETY: the length passed bounds the writable buffer.
    if unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) } != 0 {
        return OsString::new();
    }
    let end = buffer
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(buffer.len());
    OsString::from_vec(buffer[..end].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostname_matches_uname() {
        let output = Command::new("uname").arg("-n").output().unwrap();
        assert!(output.status.success());
        assert_eq!(
            Some(hostname().as_encoded_bytes()),
            output.stdout.strip_suffix(b"\n")
        );
    }
}
