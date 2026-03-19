use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let repo_path = resolve_repo_path(&args);

    let app_path = find_app();
    let Some(app) = app_path else {
        eprintln!("error: JayJay.app not found");
        eprintln!("Build it first: just build");
        std::process::exit(1);
    };

    let mut cmd = Command::new("open");
    cmd.arg("-n").arg(&app);
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

fn resolve_repo_path(args: &[String]) -> Option<PathBuf> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo" | "-r" => {
                return iter.next().map(|p| canonicalize(p));
            }
            "--" => {
                return iter.next().map(|p| canonicalize(p));
            }
            s if s.starts_with("--repo=") => {
                return Some(canonicalize(&s["--repo=".len()..]));
            }
            s if s.starts_with('-') => continue,
            _ => return Some(canonicalize(arg)),
        }
    }

    // Default: current directory if it has a .jj folder
    let cwd = env::current_dir().ok()?;
    if cwd.join(".jj").is_dir() {
        Some(cwd)
    } else {
        None
    }
}

fn canonicalize(path: &str) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_owned()
    } else {
        env::current_dir()
            .map(|cwd| cwd.join(p))
            .unwrap_or_else(|_| p.to_owned())
    }
    .canonicalize()
    .unwrap_or_else(|_| PathBuf::from(path))
}

fn find_app() -> Option<PathBuf> {
    // 1. Next to the CLI binary
    if let Ok(exe) = env::current_exe() {
        let sibling = exe.parent()?.join("JayJay.app");
        if sibling.exists() {
            return Some(sibling);
        }
    }

    // 2. /Applications
    let global = PathBuf::from("/Applications/JayJay.app");
    if global.exists() {
        return Some(global);
    }

    // 3. ~/Applications
    if let Ok(home) = env::var("HOME") {
        let user = PathBuf::from(home).join("Applications/JayJay.app");
        if user.exists() {
            return Some(user);
        }
    }

    None
}
