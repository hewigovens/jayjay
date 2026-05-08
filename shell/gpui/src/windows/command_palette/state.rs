use gpui::{App, FocusHandle, Focusable, SharedString};

pub struct CommandPalette {
    pub(super) query: String,
    pub(super) selected: usize,
    pub(super) focus_handle: FocusHandle,
    pub(super) repo_path: SharedString,
    pub(super) output: CommandOutput,
}

#[derive(Clone)]
pub(super) enum CommandOutput {
    Idle,
    Running {
        display: String,
    },
    Done {
        display: String,
        stdout: String,
        stderr: String,
        success: bool,
    },
}

impl Focusable for CommandPalette {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
