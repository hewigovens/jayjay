use std::path::PathBuf;

use gpui::{App, AppContext, Context, Entity, Focusable, KeyDownEvent, Window};

use jayjay_core::{Error, PullRequestImportPreview, PullRequestImportWorkspace, SyncToken};

use super::RepoWindow;
use crate::ui::text_area::TextArea;

pub(crate) struct PrImportState {
    pub(crate) url_input: Entity<TextArea>,
    pub(crate) name_input: Entity<TextArea>,
    pub(crate) dest_input: Entity<TextArea>,
    pub(crate) preview: Option<PullRequestImportPreview>,
    pub(crate) import_again: bool,
    pub(crate) resolving: bool,
    pub(crate) importing: bool,
    pub(crate) error: Option<String>,
    sync: Option<SyncToken>,
    focus_pending: Option<Entity<TextArea>>,
}

impl PrImportState {
    fn new(cx: &mut Context<RepoWindow>) -> Self {
        let url_input = cx.new(|cx| TextArea::new("", "Pull request URL", false, 32., cx));
        let name_input = cx.new(|cx| TextArea::new("", "Workspace name", false, 32., cx));
        let dest_input = cx.new(|cx| TextArea::new("", "Location", false, 32., cx));
        for input in [&url_input, &name_input, &dest_input] {
            TextArea::subscribe_updates(input, cx);
        }
        Self {
            focus_pending: Some(url_input.clone()),
            url_input,
            name_input,
            dest_input,
            preview: None,
            import_again: false,
            resolving: false,
            importing: false,
            error: None,
            sync: None,
        }
    }

    pub(crate) fn take_focus(&mut self, window: &mut Window, cx: &mut App) {
        if let Some(input) = self.focus_pending.take() {
            window.focus(&input.read(cx).focus_handle(cx), cx);
        }
    }

    fn inputs(&self) -> [&Entity<TextArea>; 3] {
        [&self.url_input, &self.name_input, &self.dest_input]
    }

    pub(crate) fn is_focused(&self, window: &Window, cx: &App) -> bool {
        self.inputs()
            .iter()
            .any(|input| input.read(cx).focus_handle(cx).is_focused(window))
    }

    pub(crate) fn existing_workspace(&self) -> Option<&PullRequestImportWorkspace> {
        if self.import_again {
            return None;
        }
        self.preview.as_ref()?.existing_workspace.as_ref()
    }

    pub(crate) fn can_submit(&self, cx: &App) -> bool {
        if self.resolving || self.importing {
            return false;
        }
        if self.preview.is_none() {
            return !self.url_input.read(cx).text().trim().is_empty();
        }
        if self.existing_workspace().is_some() {
            return true;
        }
        let name = self.name_input.read(cx).text();
        jayjay_core::is_valid_workspace_name(name.trim())
            && !self.dest_input.read(cx).text().trim().is_empty()
    }

    fn begin(&mut self, importing: bool, sync: SyncToken, cx: &mut Context<RepoWindow>) {
        self.resolving = !importing;
        self.importing = importing;
        self.sync = Some(sync);
        self.lock_inputs(true, cx);
    }

    fn finish(&mut self, cx: &mut Context<RepoWindow>) {
        self.resolving = false;
        self.importing = false;
        self.sync = None;
        self.lock_inputs(false, cx);
    }

    fn lock_inputs(&self, locked: bool, cx: &mut Context<RepoWindow>) {
        for input in self.inputs() {
            input.update(cx, |input, cx| input.set_read_only(locked, cx));
        }
        cx.notify();
    }

    fn visible_inputs(&self) -> Vec<&Entity<TextArea>> {
        match &self.preview {
            None => vec![&self.url_input],
            Some(_) if self.existing_workspace().is_some() => Vec::new(),
            Some(_) => vec![&self.name_input, &self.dest_input],
        }
    }

    fn focus_next_input(&self, backward: bool, window: &mut Window, cx: &mut App) {
        let inputs = self.visible_inputs();
        if inputs.is_empty() {
            return;
        }
        let current = inputs
            .iter()
            .position(|input| input.read(cx).focus_handle(cx).is_focused(window));
        let next = match (current, backward) {
            (Some(ix), false) => (ix + 1) % inputs.len(),
            (Some(ix), true) => (ix + inputs.len() - 1) % inputs.len(),
            (None, _) => 0,
        };
        window.focus(&inputs[next].read(cx).focus_handle(cx), cx);
    }

    /// Seeds the workspace fields only on the first preview, so a post-failure refresh keeps the user's edits.
    fn apply_preview(&mut self, preview: PullRequestImportPreview, cx: &mut Context<RepoWindow>) {
        if self.preview.is_none() {
            self.name_input.update(cx, |input, cx| {
                input.set_text(preview.workspace.name.clone(), cx)
            });
            self.dest_input.update(cx, |input, cx| {
                input.set_text(preview.workspace.dest.clone(), cx)
            });
            self.focus_pending = Some(self.name_input.clone());
            self.import_again = false;
        }
        self.preview = Some(preview);
    }
}

impl RepoWindow {
    pub fn open_pr_import(&mut self, cx: &mut Context<Self>) {
        if self.vm.read(cx).repo.is_none() {
            self.show_toast("Repository is not open", cx);
            return;
        }
        self.pr_import = Some(PrImportState::new(cx));
        cx.notify();
    }

    /// The repo-wide Tab cycle is suspended under modals, so the sheet routes Tab between its own fields.
    pub(crate) fn handle_pr_import_tab(
        &mut self,
        ev: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(state) = self.pr_import.as_ref() else {
            return false;
        };
        let modifiers = &ev.keystroke.modifiers;
        if ev.keystroke.key != "tab" || modifiers.platform || modifiers.alt || modifiers.control {
            return false;
        }
        state.focus_next_input(modifiers.shift, window, cx);
        true
    }

    pub fn has_pr_import_modal(&self) -> bool {
        self.pr_import.is_some()
    }

    pub fn pr_import_error(&self) -> Option<&str> {
        self.pr_import
            .as_ref()
            .and_then(|state| state.error.as_deref())
    }

    pub fn pr_import_url_input(&self) -> Option<Entity<TextArea>> {
        self.pr_import.as_ref().map(|state| state.url_input.clone())
    }

    pub fn pr_import_workspace_fields(&self, cx: &App) -> Option<(String, String)> {
        let state = self.pr_import.as_ref()?;
        Some((
            state.name_input.read(cx).text(),
            state.dest_input.read(cx).text(),
        ))
    }

    pub fn pr_import_show_preview_for_test(
        &mut self,
        preview: PullRequestImportPreview,
        cx: &mut Context<Self>,
    ) {
        if let Some(state) = self.pr_import.as_mut() {
            state.apply_preview(preview, cx);
            cx.notify();
        }
    }

    pub fn submit_pr_import(&mut self, cx: &mut Context<Self>) {
        let Some(state) = self.pr_import.as_ref() else {
            return;
        };
        if !state.can_submit(cx) {
            return;
        }
        if state.preview.is_none() {
            self.resolve_pr_import(false, cx);
            return;
        }
        if let Some(workspace) = state.existing_workspace().cloned() {
            self.pr_import = None;
            cx.notify();
            cx.defer(move |cx| super::open_repo_window(PathBuf::from(workspace.dest), cx));
            return;
        }
        self.import_pr_import(cx);
    }

    /// A running import keeps the sheet open until the canceled task reports back.
    pub fn cancel_pr_import(&mut self, cx: &mut Context<Self>) {
        let Some(state) = self.pr_import.as_ref() else {
            return;
        };
        if let Some(sync) = state.sync.as_ref() {
            sync.cancel();
        }
        if !state.importing {
            self.pr_import = None;
            cx.notify();
        }
    }

    /// A refresh after a failed import keeps that import's error, like SwiftUI's `refreshPreview`.
    fn resolve_pr_import(&mut self, refresh: bool, cx: &mut Context<Self>) {
        let Some(repo) = self.vm.read(cx).repo.clone() else {
            return;
        };
        let Some(state) = self.pr_import.as_mut() else {
            return;
        };
        let url = state.url_input.read(cx).text().trim().to_owned();
        let sync = repo.sync_token();
        if !refresh {
            state.error = None;
        }
        state.begin(false, sync.clone(), cx);
        let task =
            cx.background_spawn(async move { repo.pull_request_import_preview(&url, &sync) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |view, cx| {
                let Some(state) = view.pr_import.as_mut() else {
                    return;
                };
                state.finish(cx);
                match result {
                    Ok(preview) => state.apply_preview(preview, cx),
                    Err(error) if !refresh => state.error = Some(error.to_string()),
                    Err(_) => {}
                }
            });
        })
        .detach();
    }

    fn import_pr_import(&mut self, cx: &mut Context<Self>) {
        let Some(repo) = self.vm.read(cx).repo.clone() else {
            return;
        };
        let Some(state) = self.pr_import.as_mut() else {
            return;
        };
        let Some(head) = state
            .preview
            .as_ref()
            .map(|preview| preview.head_commit_id.clone())
        else {
            return;
        };
        let url = state.url_input.read(cx).text().trim().to_owned();
        let name = state.name_input.read(cx).text().trim().to_owned();
        let dest = state.dest_input.read(cx).text().trim().to_owned();
        let sync = repo.sync_token();
        state.error = None;
        state.begin(true, sync.clone(), cx);
        let task = cx.background_spawn(async move {
            repo.pull_request_import(&url, &head, &name, &dest, &sync)
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |view, cx| {
                // A canceled import may already have added the remote.
                if matches!(result, Ok(_) | Err(Error::Canceled)) {
                    view.vm.update(cx, |vm, cx| vm.refresh(false, cx));
                }
                match result {
                    Ok(created) => {
                        view.pr_import = None;
                        cx.notify();
                        // Opening reads every repo window, including this one while it is still being updated.
                        cx.defer(move |cx| super::open_repo_window(PathBuf::from(created), cx));
                    }
                    Err(Error::Canceled) => {
                        view.pr_import = None;
                        cx.notify();
                    }
                    Err(error) => {
                        if let Some(state) = view.pr_import.as_mut() {
                            state.finish(cx);
                            state.error = Some(error.to_string());
                        }
                        // The PR or repository may have changed under the preview; show the current head.
                        view.resolve_pr_import(true, cx);
                    }
                }
            });
        })
        .detach();
    }
}
