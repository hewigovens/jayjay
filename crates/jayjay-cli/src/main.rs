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

    // If app is already running, use URL scheme to open in existing instance
    if is_app_running() {
        if let Some(path) = &repo_path {
            let encoded = urlencoding::encode(path.to_str().unwrap_or(""));
            let url = format!("jayjay://open?path={encoded}");
            let _ = Command::new("open").arg(&url).status();
        } else {
            // Just activate the app
            let _ = Command::new("open").arg("-a").arg("JayJay").status();
        }
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

fn is_app_running() -> bool {
    Command::new("pgrep")
        .args(["-x", "JayJay"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
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
    // Try to find the .app bundle by resolving our own exe path
    if let Ok(raw_exe) = env::current_exe() {
        // Try multiple resolution strategies
        for exe in resolve_exe(&raw_exe) {
            if let Some(app) = walk_up_to_app(&exe) {
                return Some(app);
            }
            // Check sibling
            if let Some(dir) = exe.parent() {
                let sibling = dir.join("JayJay.app");
                if sibling.exists() {
                    return Some(sibling);
                }
            }
        }
    }

    // Well-known locations
    for path in ["/Applications/JayJay.app", "~/Applications/JayJay.app"] {
        let expanded = if let Some(rest) = path.strip_prefix("~/") {
            if let Ok(home) = env::var("HOME") {
                PathBuf::from(home).join(rest)
            } else {
                continue;
            }
        } else {
            PathBuf::from(path)
        };
        if expanded.exists() {
            return Some(expanded);
        }
    }

    None
}

fn resolve_exe(raw: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    // 1. canonicalize (follows symlinks + resolves ..)
    if let Ok(resolved) = raw.canonicalize() {
        results.push(resolved);
    }
    // 2. read_link chain (manual symlink resolution)
    if let Ok(target) = std::fs::read_link(raw) {
        let abs = if target.is_absolute() {
            target.clone()
        } else {
            raw.parent().unwrap_or(Path::new("/")).join(&target)
        };
        if let Ok(resolved) = abs.canonicalize() {
            if !results.contains(&resolved) {
                results.push(resolved);
            }
        } else if !results.contains(&abs) {
            results.push(abs);
        }
    }
    // 3. raw path itself
    if !results.contains(&raw.to_path_buf()) {
        results.push(raw.to_path_buf());
    }
    results
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
