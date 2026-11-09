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

/// Scoop's shim reports only that its target could not start; launching the target itself yields the OS error.
pub fn launch_target(path: PathBuf) -> PathBuf {
    let is_exe = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"));
    if !is_exe {
        return path;
    }
    std::fs::read_to_string(path.with_extension("shim"))
        .ok()
        .and_then(|shim| scoop_shim_target(&shim))
        .unwrap_or(path)
}

/// A shim that adds arguments or environment, or expands variables, has to run as the shim.
fn scoop_shim_target(shim: &str) -> Option<PathBuf> {
    let mut lines = shim
        .trim_start_matches('\u{feff}')
        .lines()
        .filter(|line| !line.trim().is_empty());
    let target = lines.next()?.strip_prefix("path = ")?;
    if lines.next().is_some() || target.contains('%') {
        return None;
    }
    let target = PathBuf::from(target.trim().trim_matches('"'));
    (target.is_absolute() && has_executable_extension(&target)).then_some(target)
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
    fn launches_the_target_of_a_plain_scoop_shim() {
        let dir = tempfile::tempdir().unwrap();
        let shim = dir.path().join("jj.exe");
        let write = |text: &str| std::fs::write(dir.path().join("jj.shim"), text).unwrap();
        assert_eq!(launch_target(shim.clone()), shim);

        let target = r"C:\Users\名 user\scoop\apps\jj\current\jj.exe";
        write(&format!("\u{feff}path = \"{target}\"\r\n"));
        assert_eq!(launch_target(shim.clone()), PathBuf::from(target));
        assert_eq!(
            launch_target(dir.path().join("jj.cmd")),
            dir.path().join("jj.cmd")
        );

        for kept in [
            format!("path = \"{target}\"\r\nargs = --flag\r\n"),
            format!("path = \"{target}\"\r\nJJ_CONFIG = C:\\config.toml\r\n"),
            "path = \"%USERPROFILE%\\scoop\\apps\\jj\\current\\jj.exe\"\r\n".to_owned(),
            "path = \"apps\\jj\\current\\jj.exe\"\r\n".to_owned(),
            "path = \"C:\\scoop\\apps\\tool\\current\\tool.ps1\"\r\n".to_owned(),
        ] {
            write(&kept);
            assert_eq!(launch_target(shim.clone()), shim, "{kept}");
        }
    }

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
