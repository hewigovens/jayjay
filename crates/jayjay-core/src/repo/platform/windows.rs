use std::ffi::{OsStr, OsString};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

/// PATHEXT can also list document types, which Command cannot launch.
const EXECUTABLE_EXTENSIONS: [&str; 4] = ["com", "exe", "bat", "cmd"];

pub fn launchctl_ssh_auth_sock() -> Option<String> {
    None
}

fn system_root() -> PathBuf {
    std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
}

pub fn system32() -> PathBuf {
    system_root().join("System32")
}

pub fn home_dir() -> Option<PathBuf> {
    home_dir_from_env(|name| std::env::var_os(name))
}

fn home_dir_from_env(env: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    let nonempty = |name| env(name).filter(|value| !value.is_empty());
    nonempty("USERPROFILE")
        .or_else(|| {
            let mut drive = nonempty("HOMEDRIVE")?;
            drive.push(nonempty("HOMEPATH")?);
            Some(drive)
        })
        .or_else(|| nonempty("HOME"))
        .map(PathBuf::from)
}

pub fn is_executable_file(path: &Path) -> bool {
    // App execution aliases such as wt.exe cannot be opened through, but they launch.
    has_executable_extension(path)
        && std::fs::metadata(path)
            .or_else(|_| std::fs::symlink_metadata(path))
            .map(|meta| meta.is_file())
            .unwrap_or(false)
}

fn has_executable_extension(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(is_executable_extension)
}

fn is_executable_extension(ext: &str) -> bool {
    EXECUTABLE_EXTENSIONS
        .iter()
        .any(|allowed| ext.eq_ignore_ascii_case(allowed))
}

pub fn executable_names(name: &str) -> Vec<String> {
    if has_executable_extension(Path::new(name)) {
        return vec![name.to_owned()];
    }
    let pathext = std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_owned());
    pathext
        .split(';')
        .filter_map(|ext| ext.strip_prefix('.'))
        .filter(|ext| is_executable_extension(ext))
        .map(|ext| format!("{name}.{ext}"))
        .collect()
}

pub fn system_path_entries() -> Vec<PathBuf> {
    vec![system32(), system_root()]
}

/// A bare name would let Command search the inherited PATH, relative entries included.
pub fn command(program: OsString) -> Command {
    let program = if Path::new(&program).components().count() == 1 {
        system32().join(program).into_os_string()
    } else {
        program
    };
    let mut command = Command::new(program);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_home_precedes_unix_home_and_supports_legacy_fallback() {
        let values = [
            ("USERPROFILE", r"C:\Users\名 user"),
            ("HOMEDRIVE", "D:"),
            ("HOMEPATH", r"\Legacy"),
            ("HOME", "C:/unix"),
        ];
        let lookup = |name: &str| {
            values
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| OsString::from(value))
        };
        assert_eq!(home_dir_from_env(lookup), Some(PathBuf::from(values[0].1)));
        assert_eq!(
            home_dir_from_env(|name| (name != "USERPROFILE").then(|| lookup(name)).flatten()),
            Some(PathBuf::from(r"D:\Legacy"))
        );
        assert_eq!(
            home_dir_from_env(|name| (name == "HOME").then(|| lookup(name)).flatten()),
            Some(PathBuf::from("C:/unix"))
        );
    }
}
