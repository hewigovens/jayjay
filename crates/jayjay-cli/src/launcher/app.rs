use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

pub(super) fn running_app_bundle() -> Option<PathBuf> {
    let out = Command::new("ps")
        .args(["-A", "-o", "comm="])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .filter(|exe| exe.ends_with("/MacOS/JayJay"))
        .find_map(|exe| walk_up_to_app_match(Path::new(exe)))
}

/// Pinned to the running instance's bundle so it receives the open request rather than the LaunchServices default for the jayjay:// scheme.
pub(super) fn open_running(repo_path: Option<&Path>, bundle: &Path) {
    let mut cmd = Command::new("open");
    cmd.arg("-a").arg(bundle);
    if let Some(path) = repo_path {
        cmd.arg(repo_url(path));
    }
    let _ = cmd.status();
}

pub(super) fn open_app(app: &Path, repo_path: Option<&Path>) -> std::io::Result<ExitStatus> {
    let mut cmd = Command::new("open");
    cmd.arg("-a").arg(app);
    if let Some(path) = repo_path {
        cmd.arg("--args").arg("--repo").arg(path);
    }
    cmd.status()
}

pub(super) fn find_app() -> Option<PathBuf> {
    // Our exe may be a symlink (e.g. ~/.local/bin/jayjay); resolve it, then walk up to the bundle.
    if let Ok(exe) = env::current_exe() {
        let exe = exe.canonicalize().unwrap_or(exe);
        if let Some(app) = walk_up_to_app(&exe) {
            return Some(app);
        }
        let sibling = exe.parent().map(|dir| dir.join("JayJay.app"));
        if let Some(app) = sibling.filter(|p| p.exists()) {
            return Some(app);
        }
    }
    installed_app()
}

fn repo_url(path: &Path) -> String {
    let encoded = urlencoding::encode(path.to_str().unwrap_or(""));
    format!("jayjay://open?path={encoded}")
}

fn installed_app() -> Option<PathBuf> {
    let user_apps = env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join("Applications/JayJay.app"));
    [Some(PathBuf::from("/Applications/JayJay.app")), user_apps]
        .into_iter()
        .flatten()
        .find(|p| p.exists())
}

fn walk_up_to_app(exe: &Path) -> Option<PathBuf> {
    walk_up_to_app_match(exe).filter(|p| p.exists())
}

fn walk_up_to_app_match(exe: &Path) -> Option<PathBuf> {
    let mut path = exe.parent()?;
    loop {
        if path.extension().is_some_and(|e| e == "app") {
            return Some(path.to_owned());
        }
        path = path.parent()?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walk_up_to_app() {
        let path = PathBuf::from("/Applications/JayJay.app/Contents/MacOS/jayjay-cli");
        let result = walk_up_to_app_match(&path);
        assert_eq!(result, Some(PathBuf::from("/Applications/JayJay.app")));
    }

    #[test]
    fn test_repo_url_encodes_path() {
        let url = repo_url(Path::new("/Users/me/my repo"));
        assert_eq!(url, "jayjay://open?path=%2FUsers%2Fme%2Fmy%20repo");
    }

    #[test]
    fn test_walk_up_no_app() {
        let path = PathBuf::from("/usr/local/bin/jayjay");
        let result = walk_up_to_app_match(&path);
        assert!(result.is_none());
    }

    #[test]
    fn test_walk_up_nested_app() {
        let path = PathBuf::from(
            "/Users/user/workspace/build/JayJay.app/Contents/Resources/bin/jayjay-cli",
        );
        let result = walk_up_to_app_match(&path);
        assert_eq!(
            result,
            Some(PathBuf::from("/Users/user/workspace/build/JayJay.app"))
        );
    }
}
