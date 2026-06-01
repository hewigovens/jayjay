use gpui::{App, Entity, FocusHandle, Focusable, SharedString};

use crate::repo::window::RepoWindow;

pub struct CommandPalette {
    pub(super) query: String,
    pub(super) selected: usize,
    pub(super) focus_handle: FocusHandle,
    pub(super) repo_path: SharedString,
    pub(super) repo_window: Option<Entity<RepoWindow>>,
    pub(super) output: CommandOutput,
    pub(super) history: Vec<String>,
    pub(super) history_index: Option<usize>,
}

#[derive(Clone)]
pub(super) enum CommandOutput {
    Idle,
    Running {
        display: String,
    },
    Done {
        display: String,
        output: String,
        exit_code: i32,
    },
}

impl CommandOutput {
    pub(super) fn is_success(&self) -> bool {
        matches!(self, Self::Done { exit_code: 0, .. })
    }
}

impl Focusable for CommandPalette {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
