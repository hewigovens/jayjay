use gpui::{
    Context, Div, InteractiveElement, MouseButton, MouseMoveEvent, MouseUpEvent, Styled, div, rgb,
};

use super::super::RepoWindow;
use crate::app::actions::{
    CopyDiffSelection, DiffEditCollapseAll, DiffEditExpandAll, ForgetStaleBookmarks,
    GitFetchOrigin, GitPushDefault, NewWorkspace, OpenAbout, OpenBookmarkManager,
    OpenCommandPalette, OpenFind, OpenOperationLog, OpenRemoteRepository, OpenRepoInEditor,
    OpenRepoInTerminal, OpenSettings, Refresh, SaveFileEditor, SaveNoteComposer,
    ShowRepoInFileManager,
};
use crate::app::theme::Theme;
use crate::platform::append_menu_bar;
use crate::ui::text_area::Newline;
use crate::windows::command_palette::CommandPalette;
use crate::windows::repo_list::RepoListWindow;
use crate::windows::settings::{SettingsSection, SettingsView};

impl RepoWindow {
    pub(super) fn render_root(&self, t: &Theme, cx: &mut Context<Self>) -> Div {
        let mut root = div()
            .track_focus(&self.focus_handle)
            .key_context("RepoWindow")
            .on_action(cx.listener(|_, _: &OpenSettings, _, cx| SettingsView::open(cx)))
            .on_action(cx.listener(|_, _: &OpenAbout, _, cx| {
                SettingsView::open_section(SettingsSection::About, cx);
            }))
            .on_action(cx.listener(|view, _: &OpenCommandPalette, _, cx| {
                let repo_path = view.vm.read(cx).repo_path.clone();
                CommandPalette::open(repo_path, Some(cx.entity()), cx);
            }))
            .on_action(cx.listener(|view, _: &OpenFind, _, cx| view.open_find(cx)))
            .on_action(cx.listener(|view, _: &OpenBookmarkManager, _, cx| {
                view.open_bookmark_manager(cx);
            }))
            .on_action(cx.listener(|view, _: &NewWorkspace, _, cx| {
                view.open_create_workspace(cx);
            }))
            .on_action(cx.listener(|view, _: &OpenOperationLog, _, cx| {
                view.open_operation_log(cx);
            }))
            .on_action(cx.listener(|view, _: &OpenRepoInEditor, _, cx| {
                view.open_repo_in_editor(cx);
            }))
            .on_action(cx.listener(|view, _: &OpenRepoInTerminal, _, cx| {
                view.open_repo_in_terminal(cx);
            }))
            .on_action(cx.listener(|view, _: &ShowRepoInFileManager, _, cx| {
                view.show_repo_in_file_manager(cx);
            }))
            .on_action(cx.listener(|view, _: &OpenRemoteRepository, _, cx| {
                view.open_remote_repository(cx);
            }))
            .on_action(cx.listener(|view, _: &GitFetchOrigin, _, cx| {
                view.git_fetch_origin(cx);
            }))
            .on_action(cx.listener(|view, _: &GitPushDefault, _, cx| {
                view.git_push_default(cx);
            }))
            .on_action(cx.listener(|view, _: &ForgetStaleBookmarks, _, cx| {
                view.forget_stale_bookmarks(cx);
            }))
            .on_action(
                cx.listener(|view, _: &CopyDiffSelection, _, cx| view.copy_diff_selection(cx)),
            )
            .on_action(
                cx.listener(|view, _: &crate::app::actions::Dismiss, window, cx| {
                    if !cx.stop_active_drag(window) {
                        view.dismiss_overlay(cx);
                    }
                }),
            )
            .on_action(
                cx.listener(|view, _: &crate::app::actions::CloseWindow, window, cx| {
                    if !view.dismiss_overlay(cx) {
                        RepoListWindow::open_if_last_repo_window(cx);
                        window.remove_window();
                    }
                }),
            )
            .on_action(cx.listener(|view, _: &Refresh, _, cx| {
                let vm = view.vm.clone();
                vm.update(cx, |vm, cx| vm.refresh(false, cx));
            }))
            // Scoped to the note composer's own "NoteComposer" key context (see `render/overlays.rs`), so this is a no-op whenever it can't actually fire.
            .on_action(cx.listener(|view, _: &SaveNoteComposer, _, cx| {
                view.submit_text_modal(cx);
            }))
            .on_action(cx.listener(|view, _: &Newline, _, cx| {
                if view
                    .text_modal
                    .as_ref()
                    .is_some_and(|modal| modal.action.submits_on_enter())
                {
                    view.submit_text_modal(cx);
                }
            }))
            .on_action(cx.listener(|view, _: &SaveFileEditor, _, cx| {
                view.save_file_editor(cx);
            }))
            .on_action(cx.listener(
                |view, _: &crate::app::actions::SubmitStackedPr, _, cx| {
                    view.submit_stacked_pr(cx);
                },
            ))
            .on_action(cx.listener(|view, _: &DiffEditExpandAll, _, cx| {
                if view.diff_edit_active() {
                    view.expand_all_diff_edit(cx);
                }
            }))
            .on_action(cx.listener(|view, _: &DiffEditCollapseAll, _, cx| {
                if view.diff_edit_active() {
                    view.collapse_all_diff_edit(cx);
                }
            }))
            .on_key_down(cx.listener(|view, ev: &gpui::KeyDownEvent, window, cx| {
                // An open context menu is modal, as a native menu would be.
                if view.context_menu.is_some() {
                    return;
                }
                if view.handle_bookmark_picker_key(ev, cx)
                    || view.handle_repo_switcher_key(ev, cx)
                {
                    return;
                }
                if view.handle_stacked_pr_key(ev, cx) {
                    return;
                }
                if view.is_text_input_focused(window, cx) {
                    return;
                }
                if view.handle_find_key(ev, cx) {
                    return;
                }
                view.handle_nav_key(ev, cx);
            }))
            .on_mouse_move(cx.listener(|view, ev: &MouseMoveEvent, _w, cx| {
                if view.layout.drag.is_some() {
                    view.drag_to(f32::from(ev.position.x), f32::from(ev.position.y), cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|view, _: &MouseUpEvent, _w, cx| {
                    view.end_drag(cx);
                }),
            )
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg));

        if let Some(state) = self.stacked_pr.as_ref() {
            root = root.key_context(if state.active_input.is_some() {
                "StackedPrPanel StackedPrInput"
            } else {
                "StackedPrPanel"
            });
        }

        append_menu_bar(root, t, cx)
    }
}
