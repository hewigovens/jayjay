use std::path::{Path, PathBuf};

use crate::types::*;

pub(crate) fn home_dir() -> Option<PathBuf> {
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
    config: &gix::config::File,
    workspace_root: &Path,
) -> Option<PathBuf> {
    if let Some(value) = config.string("core.excludesFile") {
        let path = std::str::from_utf8(&value)
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

pub(crate) fn find_existing_binary(name: &str) -> Option<String> {
    let mut candidates = Vec::new();
    if let Some(home) = home_dir() {
        candidates.push(home.join(".local").join("bin").join(name));
        candidates.push(home.join(".cargo").join("bin").join(name));
    }
    candidates.extend([
        PathBuf::from(format!("/opt/homebrew/bin/{name}")),
        PathBuf::from(format!("/usr/local/bin/{name}")),
        PathBuf::from(format!("/usr/bin/{name}")),
    ]);

    for path in candidates {
        if path.exists() {
            return Some(path.to_string_lossy().into_owned());
        }
    }
    None
}

pub(crate) fn jj_binary() -> String {
    find_binary("jj")
}

pub(crate) fn gh_binary() -> String {
    find_binary("gh")
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
