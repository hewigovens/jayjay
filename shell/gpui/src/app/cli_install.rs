//! Linux-only `jayjay` entry point in `~/.local/bin`; macOS uses the SwiftUI app-bundle installer.

use std::env;
use std::io;
use std::path::{Path, PathBuf};

use jayjay_core::is_executable_file;

pub const CLI_NAME: &str = "jayjay";

/// Single gate for the Linux-only policy; `load_state`/`repair_broken_link` no-op elsewhere so callers need no cfg of their own.
pub fn supported() -> bool {
    cfg!(target_os = "linux")
}

/// What currently sits at `~/.local/bin/jayjay`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryStatus {
    NotInstalled,
    /// A live symlink this installer manages; safe to re-point or remove.
    Installed {
        target: PathBuf,
    },
    /// A managed symlink whose target no longer exists, e.g. a deleted or replaced AppImage; repairable.
    Broken {
        target: PathBuf,
    },
    /// A regular file (`target: None`) or a symlink to something we never installed; shown read-only and never removed, replaced, or re-pointed.
    Unmanaged {
        target: Option<PathBuf>,
    },
}

/// Snapshot the settings row renders from; recomputed after every install/remove.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliInstallState {
    pub bin_dir: PathBuf,
    pub status: EntryStatus,
    /// `Some` when `bin_dir` is missing from the login-shell PATH; the shell-appropriate fix-it line.
    pub path_hint: Option<String>,
    pub error: Option<String>,
}

impl CliInstallState {
    pub fn install_path(&self) -> PathBuf {
        self.bin_dir.join(CLI_NAME)
    }
}

/// `None` on unsupported platforms or when no home directory can be resolved.
pub fn load_state() -> Option<CliInstallState> {
    if !supported() {
        return None;
    }
    install_dir().map(|dir| state_for(&dir))
}

pub fn state_for(bin_dir: &Path) -> CliInstallState {
    let status = inspect_entry(&bin_dir.join(CLI_NAME));
    let path_hint = (!dir_on_path(bin_dir, &effective_path_var()))
        .then(|| path_hint_for_shell(&jayjay_core::login_shell(), bin_dir));
    CliInstallState {
        bin_dir: bin_dir.to_owned(),
        status,
        path_hint,
        error: None,
    }
}

pub fn perform_install(bin_dir: &Path) -> Result<(), String> {
    let target = resolve_target()?;
    install_symlink(bin_dir, &target)
        .map(|_| ())
        .map_err(|err| err.to_string())
}

pub fn perform_uninstall(bin_dir: &Path) -> Result<(), String> {
    remove_managed_entry(&bin_dir.join(CLI_NAME), &install_targets())
}

/// Uninstall mirrors install's no-clobber policy: only entries this installer manages are ever deleted.
fn remove_managed_entry(link: &Path, install_targets: &[PathBuf]) -> Result<(), String> {
    match inspect_entry_against(link, install_targets) {
        EntryStatus::Installed { .. } | EntryStatus::Broken { .. } => {
            std::fs::remove_file(link).map_err(|err| err.to_string())
        }
        EntryStatus::NotInstalled => Err(format!("nothing is installed at {}", link.display())),
        EntryStatus::Unmanaged { .. } => Err(format!(
            "{} was not installed by JayJay; remove it manually",
            link.display()
        )),
    }
}

/// Launch-time repair: re-point a dangling managed symlink (e.g. after an AppImage replacement) at the running install; intact and unmanaged links are never touched.
pub fn repair_broken_link() {
    if !supported() {
        return;
    }
    let Some(dir) = install_dir() else { return };
    if !matches!(
        inspect_entry(&dir.join(CLI_NAME)),
        EntryStatus::Broken { .. }
    ) {
        return;
    }
    if let Ok(target) = resolve_target() {
        let _ = install_symlink(&dir, &target);
    }
}

fn install_dir() -> Option<PathBuf> {
    Some(jayjay_core::home_dir()?.join(".local").join("bin"))
}

pub fn inspect_entry(link: &Path) -> EntryStatus {
    inspect_entry_against(link, &install_targets())
}

fn inspect_entry_against(link: &Path, install_targets: &[PathBuf]) -> EntryStatus {
    let Ok(meta) = link.symlink_metadata() else {
        return EntryStatus::NotInstalled;
    };
    if !meta.file_type().is_symlink() {
        return EntryStatus::Unmanaged { target: None };
    }
    let target = std::fs::read_link(link).unwrap_or_default();
    let resolved = if target.is_absolute() {
        target
    } else {
        link.parent()
            .map_or(target.clone(), |dir| dir.join(&target))
    };
    if !is_managed_target(&resolved, install_targets) {
        return EntryStatus::Unmanaged {
            target: Some(resolved),
        };
    }
    if resolved.is_file() {
        EntryStatus::Installed { target: resolved }
    } else {
        EntryStatus::Broken { target: resolved }
    }
}

/// Live targets need canonical identity (or a jayjay-named AppImage); dangling ones may match by distinctive basename so relocated installs stay repairable. Bare `jayjay` never matches — any user command could carry that name.
fn is_managed_target(target: &Path, install_targets: &[PathBuf]) -> bool {
    if is_jayjay_appimage(target) {
        return true;
    }
    if target.exists() {
        let canonical = target.canonicalize().unwrap_or_else(|_| target.to_owned());
        return install_targets.iter().any(|installed| {
            installed
                .canonicalize()
                .unwrap_or_else(|_| installed.clone())
                == canonical
        });
    }
    let Some(name) = target.file_name() else {
        return false;
    };
    install_targets
        .iter()
        .any(|installed| installed.file_name() == Some(name))
        || name == suffixed_launcher("jayjay-cli").as_str()
        || name == suffixed_launcher("jayjay-gpui").as_str()
}

/// Release images rename between versions, so name+extension is the relocation-repair fallback.
fn is_jayjay_appimage(target: &Path) -> bool {
    target
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("appimage"))
        && target
            .file_stem()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.to_ascii_lowercase().contains("jayjay"))
}

/// Every path `perform_install` could link for this running install; the basis for recognizing an entry as ours.
fn install_targets() -> Vec<PathBuf> {
    let mut targets: Vec<PathBuf> = appimage_env().into_iter().collect();
    if let Ok(exe) = env::current_exe() {
        let exe = exe.canonicalize().unwrap_or(exe);
        targets.extend(sibling_cli(&exe));
        targets.push(exe);
    }
    targets
}

/// The persistent path the installed symlink should point at.
pub fn resolve_target() -> Result<PathBuf, String> {
    resolve_target_from(appimage_env(), env::current_exe().ok())
}

fn appimage_env() -> Option<PathBuf> {
    env::var_os("APPIMAGE")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn resolve_target_from(appimage: Option<PathBuf>, exe: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(image) = appimage {
        // Link the persistent AppImage file, never the transient /tmp squashfs mount current_exe lives in; erroring beats installing a link that dies on unmount.
        if image.is_absolute() && image.is_file() {
            return Ok(image);
        }
        return Err(format!("AppImage not found at {}", image.display()));
    }
    let exe = exe.ok_or_else(|| "cannot resolve the running executable".to_owned())?;
    let exe = exe.canonicalize().unwrap_or(exe);
    Ok(sibling_cli(&exe).unwrap_or(exe))
}

/// Sibling lookup only; managed-entry detection deliberately excludes bare `jayjay`.
const PACKAGED_LAUNCHERS: [&str; 2] = ["jayjay-cli", CLI_NAME];

fn suffixed_launcher(name: &str) -> String {
    format!("{name}{}", env::consts::EXE_SUFFIX)
}

/// Prefer a packaged CLI launcher shipped beside the app binary; only regular executable files count, so our own installed symlink is never picked as a target.
fn sibling_cli(exe: &Path) -> Option<PathBuf> {
    let dir = exe.parent()?;
    PACKAGED_LAUNCHERS
        .iter()
        .map(|name| dir.join(suffixed_launcher(name)))
        .find(|candidate| {
            candidate.as_path() != exe
                && candidate
                    .symlink_metadata()
                    .is_ok_and(|meta| meta.file_type().is_file())
                && is_executable_file(candidate)
        })
}

pub fn install_symlink(bin_dir: &Path, target: &Path) -> io::Result<PathBuf> {
    let link = bin_dir.join(CLI_NAME);
    if link == *target {
        return Err(io::Error::other(
            "install target is the install path itself",
        ));
    }
    if !is_executable_file(target) {
        return Err(io::Error::other(format!(
            "not an executable file: {}",
            target.display()
        )));
    }
    std::fs::create_dir_all(bin_dir)?;
    // Only ever replace an entry we manage; a regular file or foreign symlink at the path is someone else's `jayjay` and must never be clobbered.
    let mut managed = install_targets();
    managed.push(target.to_owned());
    match inspect_entry_against(&link, &managed) {
        EntryStatus::NotInstalled => {}
        EntryStatus::Installed { .. } | EntryStatus::Broken { .. } => std::fs::remove_file(&link)?,
        EntryStatus::Unmanaged { .. } => {
            return Err(io::Error::other(format!(
                "an entry JayJay does not manage already exists at {}; remove it first",
                link.display()
            )));
        }
    }
    symlink(target, &link)?;
    Ok(link)
}

#[cfg(unix)]
use std::os::unix::fs::symlink;

#[cfg(not(unix))]
fn symlink(_target: &Path, _link: &Path) -> io::Result<()> {
    Err(io::Error::other(
        "CLI install is not supported on this platform yet",
    ))
}

fn effective_path_var() -> String {
    jayjay_core::login_shell_path()
        .or_else(|| env::var("PATH").ok())
        .unwrap_or_default()
}

pub fn dir_on_path(bin_dir: &Path, path_var: &str) -> bool {
    env::split_paths(path_var).any(|entry| entry == bin_dir)
}

pub fn path_hint_for_shell(shell: &str, bin_dir: &Path) -> String {
    let dir = bin_dir.display();
    if shell.ends_with("fish") {
        return format!(
            "{dir} is not on your PATH. Add to ~/.config/fish/config.fish: fish_add_path {dir}"
        );
    }
    let rc_file = if shell.ends_with("zsh") {
        "~/.zshrc"
    } else {
        "~/.bashrc"
    };
    format!("{dir} is not on your PATH. Add to {rc_file}: export PATH=\"{dir}:$PATH\"")
}

#[cfg(test)]
mod tests;
