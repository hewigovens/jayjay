use std::collections::HashSet;
#[cfg(unix)]
use std::ffi::CStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
#[cfg(unix)]
use std::time::{Duration, Instant};

use crate::types::*;

pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

pub(crate) fn xdg_config_home() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|home| home.join(".config")))
}

pub(crate) fn git_excludes_file_path(
    excludes_file: Option<impl AsRef<[u8]>>,
    workspace_root: &Path,
) -> Option<PathBuf> {
    if let Some(value) = excludes_file {
        let path = std::str::from_utf8(value.as_ref())
            .ok()
            .map(jj_lib::file_util::expand_home_path)?;
        return Some(if path.is_absolute() {
            path
        } else {
            workspace_root.join(path)
        });
    }

    xdg_config_home().map(|path| path.join("git").join("ignore"))
}

/// Find a CLI binary. macOS app bundles don't inherit shell PATH.
pub(crate) fn find_binary(name: &str) -> String {
    find_existing_binary(name).unwrap_or_else(|| name.to_string())
}

/// Build a subprocess with the same CLI environment JayJay uses to resolve tools.
pub(crate) fn command(binary: &str) -> Command {
    let program = if Path::new(binary).components().count() == 1 {
        find_binary(binary)
    } else {
        binary.to_string()
    };
    let mut command = Command::new(program);
    apply_command_environment(&mut command);
    command
}

fn apply_command_environment(command: &mut Command) {
    if let Some(path) = command_path() {
        command.env("PATH", path);
    }
    if let Some(sock) = ssh_auth_sock() {
        command.env("SSH_AUTH_SOCK", sock);
    }
}

fn command_path() -> Option<String> {
    let login_shell_path = cached_login_shell_path().as_ref().cloned();
    let inherited_path = std::env::var("PATH").ok();
    join_path_entries(path_entries_from_values(
        [login_shell_path, inherited_path],
        home_dir(),
    ))
}

/// Resolve a CLI binary from PATH, login-shell PATH, and common fallback paths.
pub fn find_existing_binary(name: &str) -> Option<String> {
    let inherited_path = std::env::var("PATH").ok();
    find_existing_candidate(binary_candidates_from_paths(
        name,
        [inherited_path, None],
        home_dir(),
    ))
    .or_else(|| {
        let shell_path = cached_login_shell_path().as_ref().cloned();
        find_existing_candidate(binary_candidates_from_paths(name, [shell_path, None], None))
    })
}

fn find_existing_candidate(candidates: Vec<PathBuf>) -> Option<String> {
    for path in candidates {
        if is_executable_file(&path) {
            return Some(path.to_string_lossy().into_owned());
        }
    }
    None
}

/// A regular file with an execute bit on unix; any regular file elsewhere. Shared with the shell CLI installers so their notion of "executable" cannot drift from detection's.
#[cfg(unix)]
pub fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata(path)
        .map(|meta| meta.is_file() && meta.mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
pub fn is_executable_file(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|meta| meta.is_file())
        .unwrap_or(false)
}

fn binary_candidates_from_paths(
    name: &str,
    path_values: [Option<String>; 2],
    home: Option<PathBuf>,
) -> Vec<PathBuf> {
    path_entries_from_values(path_values, home)
        .into_iter()
        .map(|entry| entry.join(name))
        .collect()
}

fn path_entries_from_values(
    path_values: [Option<String>; 2],
    home: Option<PathBuf>,
) -> Vec<PathBuf> {
    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    let mut push_entry = |entry: PathBuf| {
        if entry.is_absolute() && seen.insert(entry.clone()) {
            entries.push(entry);
        }
    };

    for path in path_values.into_iter().flatten() {
        // Skip relative PATH entries so a repo-local `jj`/`gh` can't shadow a system binary.
        for entry in std::env::split_paths(&path) {
            push_entry(entry);
        }
    }
    if let Some(home) = home {
        push_entry(home.join(".local").join("bin"));
        push_entry(home.join(".cargo").join("bin"));
    }
    for entry in [
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
        PathBuf::from("/usr/sbin"),
        PathBuf::from("/sbin"),
    ] {
        push_entry(entry);
    }
    entries
}

fn join_path_entries(entries: Vec<PathBuf>) -> Option<String> {
    std::env::join_paths(entries)
        .ok()
        .map(|path| path.to_string_lossy().into_owned())
        .filter(|path| !path.is_empty())
}

fn ssh_auth_sock() -> Option<String> {
    std::env::var("SSH_AUTH_SOCK")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(super::platform::launchctl_ssh_auth_sock)
}

pub fn login_shell_path() -> Option<String> {
    cached_login_shell_path().clone()
}

fn cached_login_shell_path() -> &'static Option<String> {
    static PATH: OnceLock<Option<String>> = OnceLock::new();
    PATH.get_or_init(resolve_login_shell_path)
}

fn resolve_login_shell_path() -> Option<String> {
    #[cfg(unix)]
    {
        let shell = login_shell();
        let command = if shell.ends_with("fish") {
            "string join : -- $PATH"
        } else {
            "printf %s \"$PATH\""
        };
        run_shell_path_command(&shell, command)
    }
    #[cfg(not(unix))]
    {
        None
    }
}

#[cfg(unix)]
pub fn login_shell() -> String {
    login_shell_from_passwd()
        .or_else(|| {
            std::env::var("SHELL")
                .ok()
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| "/bin/zsh".to_string())
}

#[cfg(not(unix))]
pub fn login_shell() -> String {
    String::new()
}

#[cfg(unix)]
fn login_shell_from_passwd() -> Option<String> {
    use std::mem::MaybeUninit;

    // getpwuid_r is thread-safe; getpwuid returns a shared static and races other callers.
    let buf_size = match unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) } {
        n if n > 0 => n as usize,
        _ => 16 * 1024,
    };
    let mut buf = vec![0u8; buf_size];
    let mut passwd = MaybeUninit::<libc::passwd>::uninit();
    let mut result: *mut libc::passwd = std::ptr::null_mut();

    // SAFETY: getpwuid_r writes the passwd record into `passwd` and string fields into `buf`.
    let rc = unsafe {
        libc::getpwuid_r(
            libc::getuid(),
            passwd.as_mut_ptr(),
            buf.as_mut_ptr() as *mut libc::c_char,
            buf.len(),
            &mut result,
        )
    };
    if rc != 0 || result.is_null() {
        return None;
    }
    // SAFETY: getpwuid_r returned 0 and a non-null result, so `passwd` is initialized.
    let passwd = unsafe { passwd.assume_init() };
    if passwd.pw_shell.is_null() {
        return None;
    }
    // SAFETY: pw_shell points into `buf` and is NUL-terminated by getpwuid_r.
    unsafe { CStr::from_ptr(passwd.pw_shell) }
        .to_str()
        .ok()
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(unix)]
fn run_shell_path_command(shell: &str, command: &str) -> Option<String> {
    let mut child = Command::new(shell)
        .args(["-l", "-c", command])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait().ok()? {
            Some(status) if status.success() => {
                let output = child.wait_with_output().ok()?;
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return (!path.is_empty()).then_some(path);
            }
            Some(_) => return None,
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            None => std::thread::sleep(Duration::from_millis(20)),
        }
    }
}

pub fn jj_binary() -> String {
    find_binary("jj")
}

pub(crate) fn gh_binary() -> String {
    find_binary("gh")
}

pub(crate) fn glab_binary() -> String {
    find_binary("glab")
}

fn check_cli(binary: &str) -> CliStatus {
    let resolved = find_binary(binary);
    let is_fallback = resolved == binary;
    if is_fallback {
        match std::process::Command::new(binary).arg("version").output() {
            Ok(output) if output.status.success() => {
                let version = extract_version(&String::from_utf8_lossy(&output.stdout));
                return CliStatus {
                    is_installed: true,
                    version,
                    path: binary.to_string(),
                };
            }
            _ => {
                return CliStatus {
                    is_installed: false,
                    version: String::new(),
                    path: String::new(),
                };
            }
        }
    }
    let version = std::process::Command::new(&resolved)
        .arg("version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| extract_version(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default();
    CliStatus {
        is_installed: true,
        version,
        path: resolved,
    }
}

/// Extract a semver-like token from verbose version output (e.g., "gh version 2.89.0 (2026-03-26)" → "2.89.0").
fn extract_version(raw: &str) -> String {
    raw.split_whitespace()
        .find(|w| w.chars().next().is_some_and(|c| c.is_ascii_digit()) && w.contains('.'))
        .unwrap_or(raw.trim())
        .to_string()
}

pub fn check_jj_environment() -> CliStatus {
    check_cli("jj")
}

pub fn check_gh_environment() -> CliStatus {
    check_cli("gh")
}

pub fn check_glab_environment() -> CliStatus {
    check_cli("glab")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests below use Unix-style PATH (`:` separator, absolute paths starting with `/`),
    // so they rely on Unix semantics from `std::env::split_paths` / `Path::is_absolute`.
    #[cfg(unix)]
    #[test]
    fn binary_candidates_include_login_shell_path_entries() {
        let candidates = binary_candidates_from_paths(
            "jj",
            [
                Some("/usr/bin:/bin".to_string()),
                Some(
                    "/etc/profiles/per-user/alice/bin:/nix/var/nix/profiles/default/bin"
                        .to_string(),
                ),
            ],
            Some(PathBuf::from("/Users/alice")),
        );

        assert_eq!(candidates[0], PathBuf::from("/usr/bin/jj"));
        assert!(candidates.contains(&PathBuf::from("/etc/profiles/per-user/alice/bin/jj")));
        assert!(candidates.contains(&PathBuf::from("/nix/var/nix/profiles/default/bin/jj")));
        assert!(candidates.contains(&PathBuf::from("/Users/alice/.local/bin/jj")));
        assert!(candidates.contains(&PathBuf::from("/Users/alice/.cargo/bin/jj")));
    }

    #[cfg(unix)]
    #[test]
    fn command_path_prefers_login_shell_path_and_skips_relative_entries() {
        let path = join_path_entries(path_entries_from_values(
            [
                Some("/opt/homebrew/bin:bin:/usr/local/bin".to_string()),
                Some("/usr/bin:.:/opt/homebrew/bin".to_string()),
            ],
            Some(PathBuf::from("/Users/alice")),
        ))
        .expect("joined PATH");
        let entries = std::env::split_paths(&path).collect::<Vec<_>>();

        assert_eq!(entries[0], PathBuf::from("/opt/homebrew/bin"));
        assert!(entries.contains(&PathBuf::from("/Users/alice/.local/bin")));
        assert!(entries.contains(&PathBuf::from("/Users/alice/.cargo/bin")));
        assert!(!entries.iter().any(|p| !p.is_absolute()));
        let homebrew_bin = Path::new("/opt/homebrew/bin");
        assert_eq!(
            entries
                .iter()
                .filter(|p| p.as_path() == homebrew_bin)
                .count(),
            1
        );
    }

    #[cfg(unix)]
    #[test]
    fn binary_candidates_skip_relative_path_entries() {
        // Relative PATH entries like "." or "bin" must not become candidate paths,
        // otherwise a repo-local jj/gh could shadow a trusted system binary.
        let candidates = binary_candidates_from_paths(
            "jj",
            [Some(".:bin:/usr/bin:./tools".to_string()), None],
            None,
        );

        assert!(candidates.contains(&PathBuf::from("/usr/bin/jj")));
        assert!(!candidates.iter().any(|p| !p.is_absolute()));
    }

    #[cfg(unix)]
    #[test]
    fn find_existing_candidate_requires_executable_bit() {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let non_exec = dir.path().join("jj-nonexec");
        let exec = dir.path().join("jj-exec");

        std::fs::File::create(&non_exec)
            .unwrap()
            .write_all(b"#!/bin/sh\n")
            .unwrap();
        std::fs::set_permissions(&non_exec, std::fs::Permissions::from_mode(0o644)).unwrap();

        std::fs::File::create(&exec)
            .unwrap()
            .write_all(b"#!/bin/sh\n")
            .unwrap();
        std::fs::set_permissions(&exec, std::fs::Permissions::from_mode(0o755)).unwrap();

        // Non-executable comes first; resolver must skip it and keep looking.
        let resolved = find_existing_candidate(vec![non_exec.clone(), exec.clone()]);
        assert_eq!(resolved, Some(exec.to_string_lossy().into_owned()));
    }
}
