use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;

/// Native GUI for Jujutsu version control
#[derive(Parser)]
#[command(name = "jayjay", version, about)]
#[command(disable_version_flag = true)]
struct Cli {
    /// Path to a jj repository (default: current directory if it contains .jj)
    path: Option<String>,

    /// Open repository at PATH
    #[arg(short, long)]
    repo: Option<String>,

    /// Print version
    #[arg(short = 'v', long = "version")]
    show_version: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.show_version {
        println!("jayjay {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let repo_path = cli
        .repo
        .map(|p| canonicalize(&p))
        .or_else(|| cli.path.map(|p| canonicalize(&p)))
        .or_else(|| {
            let cwd = env::current_dir().ok()?;
            cwd.join(".jj").is_dir().then_some(cwd)
        });

    if let Some(bundle) = running_app_bundle() {
        open_in_running(repo_path.as_deref(), &bundle);
        return;
    }

    // App not running — launch it
    let Some(app) = find_app() else {
        eprintln!("error: JayJay.app not found");
        eprintln!("Install it to /Applications or build with: just build");
        std::process::exit(1);
    };

    let mut cmd = Command::new("open");
    cmd.arg("-a").arg(&app);
    if let Some(path) = &repo_path {
        cmd.arg("--args").arg("--repo").arg(path);
    }

    match cmd.status() {
        Ok(status) if status.success() => {}
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("error: failed to launch JayJay: {e}");
            std::process::exit(1);
        }
    }
}

/// Bundle of a running JayJay instance, resolved from its executable path. `None` if not running.
fn running_app_bundle() -> Option<PathBuf> {
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

/// Hand the open request to the running instance, pinned to its bundle so that app
/// receives it rather than the LaunchServices default for the jayjay:// scheme.
fn open_in_running(repo_path: Option<&Path>, bundle: &Path) {
    let mut cmd = Command::new("open");
    cmd.arg("-a").arg(bundle);
    if let Some(path) = repo_path {
        cmd.arg(repo_url(path));
    }
    let _ = cmd.status();
}

fn repo_url(path: &Path) -> String {
    let encoded = urlencoding::encode(path.to_str().unwrap_or(""));
    format!("jayjay://open?path={encoded}")
}

fn canonicalize(path: &str) -> PathBuf {
    let p = Path::new(path);
    let abs = if p.is_absolute() {
        p.to_owned()
    } else {
        env::current_dir()
            .map(|cwd| cwd.join(p))
            .unwrap_or_else(|_| p.to_owned())
    };
    abs.canonicalize().unwrap_or(abs)
}

fn find_app() -> Option<PathBuf> {
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
    well_known_app()
}

fn well_known_app() -> Option<PathBuf> {
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

/// Find the nearest ancestor with `.app` extension (without checking existence).
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
    use std::path::PathBuf;

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
