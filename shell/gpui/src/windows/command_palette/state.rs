use gpui::{App, Entity, FocusHandle, Focusable, SharedString, Subscription};

use crate::repo::window::RepoWindow;
use crate::ui::input::{CaretBlink, LineEdit};

pub struct CommandPalette {
    pub(super) query: LineEdit,
    pub(super) selected: usize,
    pub(super) focus_handle: FocusHandle,
    pub(super) repo_path: SharedString,
    pub(super) repo_window: Option<Entity<RepoWindow>>,
    pub(super) output: CommandOutput,
    pub(super) history: Vec<String>,
    pub(super) history_index: Option<usize>,
    pub(super) caret: CaretBlink,
    pub(super) focus_subscriptions: Vec<Subscription>,
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
