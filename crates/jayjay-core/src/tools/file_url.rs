use std::path::{Component, Path, PathBuf};

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

/// Return a `file://` URL for an existing non-directory file inside `repo_path`.
pub fn repo_file_url(repo_path: &str, file_path: &str) -> Option<String> {
    repo_file_path(repo_path, file_path)
        .as_deref()
        .map(file_url_from_path)
}

/// Canonical path of an existing file inside `repo_path`; parent components are fine when the canonical result stays inside the root.
pub fn repo_file_path(repo_path: &str, file_path: &str) -> Option<PathBuf> {
    let relative = Path::new(file_path);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, Component::RootDir | Component::Prefix(_)))
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
    let path = url_path(path.to_string_lossy().replace('\\', "/"));
    format!("file://{}", utf8_percent_encode(&path, FILE_URL_PATH_SET))
}

#[cfg(not(windows))]
fn url_path(path: String) -> String {
    path
}

/// Drive paths gain a leading `/`; UNC paths drop their `//` so the server becomes the URL host.
#[cfg(windows)]
fn url_path(path: String) -> String {
    let path = if let Some(rest) = path.strip_prefix("//?/UNC/") {
        format!("//{rest}")
    } else if let Some(rest) = path.strip_prefix("//?/") {
        rest.to_owned()
    } else {
        path
    };
    if let Some(unc) = path.strip_prefix("//") {
        unc.to_owned()
    } else if path.as_bytes().get(1) == Some(&b':') {
        format!("/{path}")
    } else {
        path
    }
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

    #[cfg(windows)]
    #[test]
    fn windows_file_urls_support_verbatim_drive_and_network_paths() {
        assert_eq!(
            file_url_from_path(Path::new(r"\\?\C:\名 repo\a#%.html")),
            "file:///C:/%E5%90%8D%20repo/a%23%25.html"
        );
        assert_eq!(
            file_url_from_path(Path::new(r"\\server\share\page name.html")),
            "file://server/share/page%20name.html"
        );
        assert_eq!(
            file_url_from_path(Path::new(r"\\?\UNC\server\share\page name.html")),
            "file://server/share/page%20name.html"
        );
    }

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
        let escaping = format!(
            "../{}",
            outside.path().file_name().unwrap().to_str().unwrap()
        );
        assert_eq!(repo_file_url(tmp.path().to_str().unwrap(), &escaping), None);
    }

    #[test]
    fn repo_file_path_keeps_contained_parent_components() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("assets").join("logo.png");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::create_dir(tmp.path().join("docs")).unwrap();
        std::fs::write(&file, b"png").unwrap();

        assert_eq!(
            repo_file_path(tmp.path().to_str().unwrap(), "docs/../assets/logo.png"),
            Some(tmp.path().canonicalize().unwrap().join("assets/logo.png"))
        );
    }
}
