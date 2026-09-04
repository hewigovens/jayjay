use std::path::{Path, PathBuf};

use crate::repo::{find_existing_binary, subprocess_command};

use super::super::super::file_url::file_url_from_path;
use super::super::super::launcher::EditorLaunch;

/// The `text/plain` handler's desktop entry as a launch for `path`.
pub fn default_text_editor(path: &str) -> Option<EditorLaunch> {
    let desktop_id = xdg_mime_default("text/plain")?;
    let entry = find_desktop_file(&desktop_id, &application_dirs())?;
    let (exec, in_terminal) = parse_desktop_entry(&std::fs::read_to_string(entry).ok()?)?;
    let mut argv = expand_exec(&exec, path)?;
    argv[0] = find_existing_binary(&argv[0])?;
    Some(EditorLaunch { argv, in_terminal })
}

fn xdg_mime_default(mime: &str) -> Option<String> {
    let binary = find_existing_binary("xdg-mime")?;
    let output = subprocess_command(&binary)
        .args(["query", "default", mime])
        .output()
        .ok()?;
    let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (output.status.success() && !value.is_empty()).then_some(value)
}

/// Absolute directories only: a relative XDG entry would resolve against the repository the app was launched in.
fn application_dirs() -> Vec<PathBuf> {
    let env_path = |name: &str| {
        std::env::var_os(name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    };
    let data_home = env_path("XDG_DATA_HOME")
        .or_else(|| env_path("HOME").map(|home| home.join(".local/share")));
    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|dirs| !dirs.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".to_owned());
    data_home
        .into_iter()
        .chain(data_dirs.split(':').map(PathBuf::from))
        .filter(|dir| dir.is_absolute())
        .map(|dir| dir.join("applications"))
        .collect()
}

/// Desktop IDs encode subdirectories as `-`, so `vendor-nvim.desktop` may live at `vendor/nvim.desktop`.
fn find_desktop_file(desktop_id: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    dirs.iter().find_map(|dir| {
        let direct = dir.join(desktop_id);
        if direct.is_file() {
            return Some(direct);
        }
        desktop_files(dir, 3).into_iter().find(|file| {
            file.strip_prefix(dir).is_ok_and(|relative| {
                let id: Vec<_> = relative.iter().map(|part| part.to_string_lossy()).collect();
                id.join("-") == desktop_id
            })
        })
    })
}

fn desktop_files(dir: &Path, depth: usize) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .flat_map(|entry| {
            let path = entry.path();
            match entry.file_type() {
                Ok(kind) if kind.is_dir() && depth > 0 => desktop_files(&path, depth - 1),
                Ok(kind)
                    if kind.is_file() && path.extension().is_some_and(|ext| ext == "desktop") =>
                {
                    vec![path]
                }
                _ => Vec::new(),
            }
        })
        .collect()
}

fn parse_desktop_entry(contents: &str) -> Option<(String, bool)> {
    let mut in_main_group = false;
    let mut exec = None;
    let mut terminal = false;
    for line in contents.lines().map(str::trim) {
        if line.starts_with('[') {
            in_main_group = line == "[Desktop Entry]";
        } else if !in_main_group {
            continue;
        } else if let Some(value) = line.strip_prefix("Exec=") {
            exec = Some(value.trim().to_owned());
        } else if let Some(value) = line.strip_prefix("Terminal=") {
            terminal = value.trim() == "true";
        }
    }
    exec.filter(|exec| !exec.is_empty())
        .map(|exec| (exec, terminal))
}

fn expand_exec(exec: &str, path: &str) -> Option<Vec<String>> {
    let mut argv = Vec::new();
    let mut has_target = false;
    for word in shell_words::split(exec).ok()? {
        match word.as_str() {
            "%f" | "%F" => {
                has_target = true;
                argv.push(path.to_owned());
            }
            "%u" | "%U" => {
                has_target = true;
                argv.push(file_url_from_path(Path::new(path)));
            }
            "%%" => argv.push("%".to_owned()),
            code if code.len() == 2 && code.starts_with('%') => {}
            _ => argv.push(word),
        }
    }
    if argv.is_empty() {
        return None;
    }
    if !has_target {
        argv.push(path.to_owned());
    }
    Some(argv)
}

#[cfg(test)]
mod tests {
    use super::{expand_exec, find_desktop_file, parse_desktop_entry};

    #[test]
    fn desktop_entry_reads_exec_and_terminal_from_the_main_group() {
        let nvim = "[Desktop Entry]\nExec=nvim %F\nTerminal=true\n";
        assert_eq!(
            parse_desktop_entry(nvim),
            Some(("nvim %F".to_owned(), true))
        );
        let gui = "[Desktop Entry]\nExec=editor --new-window %U\nTerminal=false\n[Desktop Action Console]\nExec=editor --console %F\nTerminal=true\n";
        assert_eq!(
            parse_desktop_entry(gui),
            Some(("editor --new-window %U".to_owned(), false))
        );
        assert_eq!(parse_desktop_entry("[Desktop Entry]\nName=Broken\n"), None);
    }

    #[test]
    fn exec_field_codes_expand_in_place() {
        let path = "/repo/a b.rs";
        assert_eq!(expand_exec("nvim %F", path).unwrap(), ["nvim", path]);
        assert_eq!(
            expand_exec("wrapper %f fixed", path).unwrap(),
            ["wrapper", path, "fixed"]
        );
        assert_eq!(
            expand_exec("browser %U", path).unwrap(),
            ["browser", "file:///repo/a%20b.rs"]
        );
        assert_eq!(
            expand_exec("editor --new-window %i", path).unwrap(),
            ["editor", "--new-window", path]
        );
        assert_eq!(expand_exec("", path), None);
    }

    #[test]
    fn desktop_ids_resolve_encoded_subdirectories_without_following_symlink_loops() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("vendor")).unwrap();
        std::fs::write(
            dir.path().join("vendor").join("nvim.desktop"),
            "[Desktop Entry]\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("gedit.desktop"), "[Desktop Entry]\n").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(".", dir.path().join("loop")).unwrap();
        let dirs = [dir.path().to_path_buf()];
        assert_eq!(
            find_desktop_file("gedit.desktop", &dirs),
            Some(dir.path().join("gedit.desktop"))
        );
        assert_eq!(
            find_desktop_file("vendor-nvim.desktop", &dirs),
            Some(dir.path().join("vendor").join("nvim.desktop"))
        );
        assert_eq!(find_desktop_file("missing.desktop", &dirs), None);
    }
}
