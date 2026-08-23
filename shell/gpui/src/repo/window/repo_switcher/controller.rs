use std::collections::HashSet;
use std::path::{Path, PathBuf};

use gpui::{AnyWindowHandle, Context, KeyDownEvent, Pixels, Point};

use super::model::{RepoSwitcherAction, RepoSwitcherState};
use super::sections::switcher_sections;
use crate::app::repositories;
use crate::repo::window::RepoWindow;
use crate::repo::window::picker::{PickerOutcome, PickerQuery, picker_actions};
use crate::ui::input::LineInput;
use crate::windows::repo_list::RepoListWindow;

impl RepoWindow {
    pub(crate) fn open_repo_switcher(
        &mut self,
        anchor: Point<Pixels>,
        current_window: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) {
        let current = self.vm.read(cx).repo_path.to_string();
        let (open, pinned) = repository_rows(current_window, &current, cx);
        #[cfg(not(target_os = "macos"))]
        {
            self.app_menu = None;
        }
        self.context_menu = None;
        self.close_bookmark_picker(cx);
        self.repo_switcher = Some(RepoSwitcherState {
            anchor,
            current,
            open,
            pinned,
            query: PickerQuery::new(),
        });
        LineInput::show_for_owner(self, cx, Self::repo_switcher_input);
        cx.notify();
    }

    fn repo_switcher_query(view: &mut Self) -> Option<&mut PickerQuery> {
        view.repo_switcher.as_mut().map(|state| &mut state.query)
    }

    fn repo_switcher_input(view: &mut Self) -> Option<&mut LineInput> {
        Self::repo_switcher_query(view).map(|query| &mut query.input)
    }

    pub(crate) fn close_repo_switcher(&mut self, cx: &mut Context<Self>) {
        if self.repo_switcher.is_some() {
            LineInput::hide_for_owner(self, cx, Self::repo_switcher_input);
            self.repo_switcher = None;
            cx.notify();
        }
    }

    pub(super) fn refresh_repo_switcher(
        &mut self,
        current_window: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) {
        let current = self.vm.read(cx).repo_path.to_string();
        let (open, pinned) = repository_rows(current_window, &current, cx);
        if let Some(state) = self.repo_switcher.as_mut() {
            state.current = current;
            state.open = open;
            state.pinned = pinned;
        }
        // The rows just moved under the selection, so a remembered index would activate a different row.
        let count = self.repo_switcher_actions(cx).len();
        if let Some(state) = self.repo_switcher.as_mut() {
            state.query.reset_selection_after_edit(count);
        }
        self.vm.update(cx, |vm, cx| vm.refresh_workspaces(cx));
        cx.notify();
    }

    pub(super) fn dispatch_repo_switcher(
        &mut self,
        action: RepoSwitcherAction,
        cx: &mut Context<Self>,
    ) {
        self.close_repo_switcher(cx);
        match action {
            RepoSwitcherAction::Activate(path) => {
                if path == self.vm.read(cx).repo_path.as_ref() {
                    return;
                }
                cx.defer(move |cx| {
                    crate::repo::window::open::activate_repo_window(Path::new(&path), cx);
                });
            }
            RepoSwitcherAction::Open(path) => {
                cx.defer(move |cx| {
                    crate::repo::window::open::open_repo_window(PathBuf::from(path), cx);
                });
            }
            RepoSwitcherAction::ShowRepositoryList => cx.defer(RepoListWindow::open),
            RepoSwitcherAction::OpenRepository => {
                cx.defer(crate::app::menus::prompt_open_repository);
            }
        }
    }

    pub(in crate::repo::window) fn handle_repo_switcher_key(
        &mut self,
        event: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(outcome) = self.drive_picker(
            event,
            Self::repo_switcher_query,
            Self::repo_switcher_input,
            |view, cx| view.repo_switcher_actions(cx),
            cx,
        ) else {
            return false;
        };
        match outcome {
            PickerOutcome::Handled => {}
            PickerOutcome::Dismiss => self.close_repo_switcher(cx),
            PickerOutcome::Activate(action) => self.dispatch_repo_switcher(action, cx),
        }
        true
    }

    fn repo_switcher_actions(&self, cx: &gpui::App) -> Vec<(RepoSwitcherAction, usize)> {
        let Some(state) = self.repo_switcher.as_ref() else {
            return Vec::new();
        };
        let workspaces = self.vm.read(cx).graph.workspaces.clone();
        picker_actions(&switcher_sections(state, &workspaces))
    }
}

fn repository_rows(
    current_window: AnyWindowHandle,
    current: &str,
    cx: &mut Context<RepoWindow>,
) -> (Vec<String>, Vec<String>) {
    let open = crate::repo::window::open::open_repo_paths(current_window, current, cx);
    let open_set = open.iter().collect::<HashSet<_>>();
    let pinned = repositories::current(cx)
        .into_iter()
        .filter(|path| !open_set.contains(path))
        .collect();
    (open, pinned)
}
