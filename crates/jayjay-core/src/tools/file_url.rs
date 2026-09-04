use std::path::{Component, Path, PathBuf};

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

/// Return a `file://` URL for an existing non-directory file inside `repo_path`.
pub fn repo_file_url(repo_path: &str, file_path: &str) -> Option<String> {
    existing_repo_file_path(repo_path, file_path)
        .as_deref()
        .map(file_url_from_path)
}

fn existing_repo_file_path(repo_path: &str, file_path: &str) -> Option<PathBuf> {
    let relative = Path::new(file_path);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return None;
    }

    let repo = Path::new(repo_path).canonicalize().ok()?;
    let candidate = repo.join(relative).canonicalize().ok()?;
    if !candidate.starts_with(&repo) || !candidate.is_file() {
        return None;
    }
    Some(candidate)
}

pub(super) fn file_url_from_path(path: &Path) -> String {
    let mut path = path.to_string_lossy().replace('\\', "/");
    if cfg!(windows) && path.as_bytes().get(1) == Some(&b':') {
        path.insert(0, '/');
    }
    format!("file://{}", utf8_percent_encode(&path, FILE_URL_PATH_SET))
}

const FILE_URL_PATH_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'[')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_file_url_opens_existing_repo_file() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("docs").join("index page#v%.html");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "<html></html>").unwrap();

        let url = repo_file_url(tmp.path().to_str().unwrap(), "docs/index page#v%.html")
            .expect("html file should be openable");

        assert!(url.starts_with("file:///"), "{url}");
        assert!(url.ends_with("/docs/index%20page%23v%25.html"), "{url}");
    }

    #[test]
    fn repo_file_url_rejects_dirs_missing_files_and_escapes() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("docs")).unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();

        assert_eq!(repo_file_url(tmp.path().to_str().unwrap(), "docs"), None);
        assert_eq!(
            repo_file_url(tmp.path().to_str().unwrap(), "missing.html"),
            None
        );
        assert_eq!(
            repo_file_url(
                tmp.path().to_str().unwrap(),
                outside.path().to_str().unwrap()
            ),
            None
        );
        assert_eq!(
            repo_file_url(tmp.path().to_str().unwrap(), "../outside.html"),
            None
        );
    }
}
