use std::env;
use std::path::{Path, PathBuf};

pub(super) fn repo_path(repo: Option<String>, path: Option<String>) -> Option<PathBuf> {
    repo.map(|p| canonicalize(&p))
        .or_else(|| path.map(|p| canonicalize(&p)))
        .or_else(|| {
            let cwd = env::current_dir().ok()?;
            cwd.join(".jj").is_dir().then_some(cwd)
        })
}

pub(crate) fn canonicalize(path: &str) -> PathBuf {
    let path = Path::new(path);
    let abs = if path.is_absolute() {
        path.to_owned()
    } else {
        env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_owned())
    };
    abs.canonicalize().unwrap_or(abs)
}
