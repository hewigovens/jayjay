use super::path;

#[cfg(target_os = "macos")]
use super::app::launch;
#[cfg(not(target_os = "macos"))]
use super::gpui::launch;

pub(crate) fn launch_app(repo: Option<String>, path: Option<String>) {
    launch(path::repo_path(repo, path).as_deref());
}
