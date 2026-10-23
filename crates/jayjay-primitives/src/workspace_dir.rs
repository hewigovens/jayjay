use std::path::Path;

/// Nearest directory at or above the canonical `dir` that holds `.jj`, as jj discovers its workspace.
pub fn enclosing_workspace_dir(dir: &Path) -> Option<&Path> {
    if !dir.is_dir() {
        return None;
    }
    dir.ancestors()
        .find(|ancestor| ancestor.join(".jj").is_dir())
}

#[cfg(test)]
mod tests {
    use super::enclosing_workspace_dir;
    use std::fs;

    #[test]
    fn finds_the_nearest_workspace_directory() {
        let temp = tempfile::tempdir().expect("temp dir");
        let outer = temp.path().join("outer");
        let inner = outer.join("vendor/inner");
        let plain = temp.path().join("plain");
        for dir in [
            outer.join(".jj"),
            inner.join(".jj"),
            inner.join("src"),
            plain.clone(),
        ] {
            fs::create_dir_all(dir).expect("create dir");
        }
        let file = inner.join("src/lib.rs");
        fs::write(&file, "").expect("write file");

        assert_eq!(
            enclosing_workspace_dir(&inner.join("src")),
            Some(inner.as_path())
        );
        assert_eq!(
            enclosing_workspace_dir(&outer.join("vendor")),
            Some(outer.as_path())
        );
        assert_eq!(enclosing_workspace_dir(&plain), None);
        assert_eq!(enclosing_workspace_dir(&file), None);
    }
}
