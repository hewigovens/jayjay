//! Thin wrapper over `jayjay_core::tools` — pulls the user's tool config
//! out of `AppConfig` and forwards. Real launcher logic lives in core so
//! both shells share it.

use gpui::App;
use std::path::{Component, Path, PathBuf};

use crate::app::config;
use crate::platform::reveal_path;

pub use jayjay_core::tools::{EDITOR_OPTIONS, TERMINAL_OPTIONS};

pub fn editor_title(cx: &App) -> &'static str {
    let cfg = config::current(cx);
    EDITOR_OPTIONS
        .iter()
        .find_map(|(id, label)| (*id == cfg.tools.external_editor.as_str()).then_some(*label))
        .unwrap_or("Editor")
}

pub fn open_in_editor_label(cx: &App) -> String {
    format!("Open in {}", editor_title(cx))
}

pub fn terminal_title(cx: &App) -> &'static str {
    let cfg = config::current(cx);
    TERMINAL_OPTIONS
        .iter()
        .find_map(|(id, label)| (*id == cfg.tools.terminal.as_str()).then_some(*label))
        .unwrap_or("Terminal")
}

pub fn open_in_terminal_label(cx: &App) -> String {
    format!("Open in {}", terminal_title(cx))
}

pub fn open_in_editor(repo_path: &str, file_path: &str, cx: &App) -> bool {
    jayjay_core::open_in_editor(repo_path, file_path, &cfg(cx))
}

pub fn open_in_terminal(repo_path: &str, cx: &App) -> bool {
    jayjay_core::open_in_terminal(repo_path, None, &cfg(cx))
}

pub fn show_in_file_manager(repo_path: &str, file_path: Option<&str>) -> bool {
    reveal_path(&file_viewer_selection_path(repo_path, file_path))
}

fn cfg(cx: &App) -> jayjay_core::ToolsConfig {
    let app_cfg = config::current(cx);
    jayjay_core::ToolsConfig {
        external_editor: app_cfg.tools.external_editor.clone(),
        custom_editor_command: app_cfg.tools.custom_editor_command.clone(),
        terminal: app_cfg.tools.terminal.clone(),
        custom_terminal_command: app_cfg.tools.custom_terminal_command.clone(),
    }
}

fn file_viewer_selection_path(repo_path: &str, file_path: Option<&str>) -> PathBuf {
    let repo = Path::new(repo_path);
    let Some(path) = file_path.filter(|path| !path.is_empty()) else {
        return repo.to_path_buf();
    };
    let relative = Path::new(path);
    if !is_safe_relative_path(relative) {
        return repo.to_path_buf();
    }
    let mut candidate = repo.join(relative);
    while candidate.starts_with(repo) {
        if candidate.exists() {
            return candidate;
        }
        let Some(parent) = candidate.parent() else {
            break;
        };
        if parent == candidate {
            break;
        }
        candidate = parent.to_path_buf();
    }
    repo.to_path_buf()
}

fn is_safe_relative_path(path: &Path) -> bool {
    path.components().all(|component| {
        !matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::file_viewer_selection_path;

    #[test]
    fn file_viewer_selection_path_uses_existing_file() {
        let repo = test_repo("existing");
        let file = repo.join("src/main.rs");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "fn main() {}\n").unwrap();

        assert_eq!(
            file_viewer_selection_path(repo.to_str().unwrap(), Some("src/main.rs")),
            file
        );
        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn file_viewer_selection_path_falls_back_to_existing_parent() {
        let repo = test_repo("parent");
        let dir = repo.join("src");
        fs::create_dir_all(&dir).unwrap();

        assert_eq!(
            file_viewer_selection_path(repo.to_str().unwrap(), Some("src/deleted.rs")),
            dir
        );
        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn file_viewer_selection_path_rejects_escaping_paths() {
        let repo = test_repo("escape");
        fs::create_dir_all(&repo).unwrap();

        assert_eq!(
            file_viewer_selection_path(repo.to_str().unwrap(), Some("../outside.rs")),
            repo
        );
        let _ = fs::remove_dir_all(repo);
    }

    fn test_repo(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("jayjay-gpui-tools-{name}-{}", std::process::id()))
    }
}
