use gpui::Context;

use super::RepoViewModel;

/// What the Refresh button and ⌘R do: a stale working copy cannot be snapshotted, so they update it instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RefreshMode {
    Refresh,
    UpdateWorkspace,
}

impl RefreshMode {
    pub(crate) fn tooltip(self) -> &'static str {
        match self {
            RefreshMode::Refresh => "Refresh",
            RefreshMode::UpdateWorkspace => "Update Workspace — the working copy is stale",
        }
    }

    pub(crate) fn shows_badge(self) -> bool {
        self == RefreshMode::UpdateWorkspace
    }
}

impl RepoViewModel {
    pub(crate) fn refresh_mode(&self) -> RefreshMode {
        if self.working_copy_stale {
            RefreshMode::UpdateWorkspace
        } else {
            RefreshMode::Refresh
        }
    }

    /// Every manual Refresh activation (button, ⌘R, keyboard focus, palette) goes through here.
    pub(crate) fn run_refresh(&mut self, cx: &mut Context<Self>) {
        match self.refresh_mode() {
            RefreshMode::Refresh => self.refresh(false, cx),
            RefreshMode::UpdateWorkspace => self.update_stale_workspace(cx).detach(),
        }
    }
}
