use crate::types::*;

/// Find a CLI binary. macOS app bundles don't inherit shell PATH.
pub(crate) fn find_binary(name: &str) -> String {
    let homebrew = format!("/opt/homebrew/bin/{name}");
    let usr_local = format!("/usr/local/bin/{name}");
    let usr = format!("/usr/bin/{name}");
    let candidates = [homebrew.as_str(), usr_local.as_str(), usr.as_str()];
    if let Ok(home) = std::env::var("HOME") {
        let cargo = format!("{home}/.cargo/bin/{name}");
        if std::path::Path::new(&cargo).exists() {
            return cargo;
        }
    }
    for path in candidates {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }
    name.to_string()
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
                return CliStatus { is_installed: true, version, path: binary.to_string() };
            }
            _ => return CliStatus { is_installed: false, version: String::new(), path: String::new() },
        }
    }
    let version = std::process::Command::new(&resolved)
        .arg("version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| extract_version(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default();
    CliStatus { is_installed: true, version, path: resolved }
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
