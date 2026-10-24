use std::env;
use std::path::{Path, PathBuf};

use jayjay_primitives::enclosing_workspace_dir;

pub(super) fn repo_path(repo: Option<String>, path: Option<String>) -> Option<PathBuf> {
    let cwd = env::current_dir().ok();
    launch_path(repo.or(path).as_deref(), cwd.as_deref())
}

fn launch_path(explicit: Option<&str>, cwd: Option<&Path>) -> Option<PathBuf> {
    match explicit {
        Some(path) => {
            let path = canonicalize(path, cwd);
            Some(enclosing_workspace_dir(&path).map_or_else(|| path.clone(), Path::to_path_buf))
        }
        None => enclosing_workspace_dir(&dunce::canonicalize(cwd?).ok()?).map(Path::to_path_buf),
    }
}

fn canonicalize(path: &str, cwd: Option<&Path>) -> PathBuf {
    let path = Path::new(path);
    let abs = match cwd {
        Some(cwd) if !path.is_absolute() => cwd.join(path),
        _ => path.to_owned(),
    };
    dunce::canonicalize(&abs).unwrap_or(abs)
}

#[cfg(test)]
mod tests {
    use super::launch_path;
    use std::fs;

    #[test]
    fn launches_resolve_to_the_enclosing_workspace() {
        let temp = tempfile::tempdir().expect("temp dir");
        let base = dunce::canonicalize(temp.path()).expect("canonical temp dir");
        let repo = base.join("repo");
        let subfolder = repo.join("src/nested");
        let plain = base.join("plain");
        for dir in [repo.join(".jj"), subfolder.clone(), plain.clone()] {
            fs::create_dir_all(dir).expect("create dir");
        }

        let explicit = |path: &str, cwd: &std::path::Path| launch_path(Some(path), Some(cwd));
        assert_eq!(explicit("../repo/src/nested", &plain), Some(repo.clone()));
        assert_eq!(explicit("plain", &base), Some(plain.clone()));
        assert_eq!(launch_path(None, Some(&subfolder)), Some(repo.clone()));
        assert_eq!(launch_path(None, Some(&plain)), None);
        #[cfg(unix)]
        {
            let link = base.join("link");
            std::os::unix::fs::symlink(&subfolder, &link).expect("symlink");
            assert_eq!(explicit("link", &base), Some(repo));
        }
    }
}
