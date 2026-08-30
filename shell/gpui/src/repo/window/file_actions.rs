//! Direct file actions from the file column: split files from any mutable change, move historical files to @, or commit selected working-copy files.

use std::sync::Arc;

use gpui::{App, Context, Pixels, Point, SharedString};

use super::{RepoWindow, TextModalAction, TextModalCheckbox, TextModalState};
use crate::repo::revset;
use crate::ui::context_menu::ContextMenuItem;
use crate::ui::overlay::TextPrompt;

/// A selected revision and its file paths, shared by direct file actions and their confirmation UI.
pub struct SelectedFilesRequest {
    pub(super) rev: String,
    pub(super) paths: Vec<String>,
}

impl RepoWindow {
    pub(crate) fn open_file_context_menu(
        &mut self,
        clicked_path: &str,
        anchor: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let items = self.build_file_context_menu(clicked_path, cx);
        self.open_context_menu(anchor, items, cx);
    }

    /// Single-file rows get the inspection menu plus batch actions; a >1 selection gets batch actions only, mirroring SwiftUI's file context menu.
    pub fn build_file_context_menu(&self, clicked_path: &str, cx: &App) -> Vec<ContextMenuItem> {
        let paths = self.file_context_selection(clicked_path, cx);
        let mut items = if paths.len() <= 1 {
            Self::build_file_menu(clicked_path, cx)
        } else {
            Vec::new()
        };
        items.extend(self.batch_file_menu_items(&paths, cx));
        items
    }

    /// Split-to-new-change modal: title/checkbox/file-list mirror SwiftUI's `SplitSheetView` (`Split N files to new change`, "Parallel split" toggle, paths sorted for display). Shared by the file context menu's "Split ... to New Change" and the header's reviewed-files quick-split button.
    pub(crate) fn open_split_files_modal(
        &mut self,
        request: Arc<SelectedFilesRequest>,
        cx: &mut Context<Self>,
    ) {
        let count = request.paths.len();
        let noun = if count == 1 { "file" } else { "files" };
        let mut sorted_paths = request.paths.clone();
        sorted_paths.sort();
        self.text_modal = Some(TextModalState {
            prompt: TextPrompt::single_line(
                format!("Split {count} {noun} to new change"),
                SharedString::default(),
                "",
                "Description for split change",
                "Split",
                cx,
            ),
            action: TextModalAction::SplitFiles(request),
            context: None,
            checkbox: Some(TextModalCheckbox {
                label: "Parallel split".into(),
                checked: false,
            }),
            file_list: Some(sorted_paths.into_iter().map(SharedString::from).collect()),
        });
        cx.notify();
    }

    /// Header's quick-split button (SwiftUI: the file-column toolbar's branch icon) targets the files currently marked reviewed, not the row multi-selection.
    pub(crate) fn open_reviewed_files_split_modal(&mut self, cx: &mut Context<Self>) {
        let Some((rev, paths)) = self.reviewed_files_split_target(cx) else {
            return;
        };
        self.open_split_files_modal(Arc::new(SelectedFilesRequest { rev, paths }), cx);
    }

    fn reviewed_files_split_target(&self, cx: &App) -> Option<(String, Vec<String>)> {
        let vm = self.vm.read(cx);
        let change = vm.selected_change_for_file_ops()?;
        let rev = revset::change_revision(change);
        let change_id = change.change_id.id.clone();
        let files = vm.files.clone()?;
        let paths: Vec<String> = files
            .iter()
            .filter(|h| self.is_reviewed(&change_id, &h.path, &h.review_identity))
            .map(|h| h.path.clone())
            .collect();
        (!paths.is_empty()).then_some((rev, paths))
    }

    pub(crate) fn confirm_split_files(
        &mut self,
        request: Arc<SelectedFilesRequest>,
        message: String,
        parallel: bool,
        cx: &mut Context<Self>,
    ) {
        self.run_split_files(request, message, parallel, false, cx);
    }

    /// Commit the selected files with the commit-box message: core `split` on @ gives `jj commit FILESETS` semantics — the selected files become a finished change described by the message, the remainder stays as the working copy (with a fresh change id).
    pub(crate) fn commit_selected_files(
        &mut self,
        request: Arc<SelectedFilesRequest>,
        cx: &mut Context<Self>,
    ) {
        let Some(message) = self.commit_message_requiring_summary(cx) else {
            return;
        };
        self.run_split_files(request, message, false, true, cx);
    }

    fn run_split_files(
        &mut self,
        request: Arc<SelectedFilesRequest>,
        message: String,
        parallel: bool,
        clear_commit_inputs: bool,
        cx: &mut Context<Self>,
    ) {
        // SwiftUI parity: split-off paths leave the review session, unmarked on the pre-split change id (the remainder's marks go stale anyway once @ gets its fresh id).
        let review_change_id = self
            .vm
            .read(cx)
            .selected_change()
            .map(|c| c.change_id.id.clone());
        let paths = request.paths.clone();
        let task = self.vm.update(cx, |vm, cx| {
            vm.split_files(
                request.rev.clone(),
                request.paths.clone(),
                message,
                parallel,
                cx,
            )
        });
        cx.spawn(async move |this, cx| {
            if task.await.is_ok() {
                let _ = this.update(cx, |view, cx| {
                    if let Some(change_id) = review_change_id {
                        super::review::mutate(&view.review_store, |store| {
                            for path in &paths {
                                store.mark_unreviewed(&change_id, path);
                            }
                        });
                    }
                    if clear_commit_inputs {
                        view.clear_commit_box(cx);
                    }
                    cx.notify();
                });
            }
        })
        .detach();
    }
}
