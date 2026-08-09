use std::sync::Arc;

use gpui::{App, Context};
use jayjay_core::ChangeInfo;

use super::RepoWindow;
use crate::repo::revset;
use crate::ui::context_menu::{ContextAction, ContextMenuItem};
use crate::ui::icons::glyph;

pub enum ChangeAction {
    Edit { rev: String },
    Squash { rev: String, into: Option<String> },
    Rebase { rev: String, dest: String },
    Merge { parents: Vec<String> },
    Duplicate { rev: String },
    Absorb { rev: String },
    Revert { rev: String },
}

impl RepoWindow {
    pub fn build_change_menu(&self, change: &ChangeInfo, cx: &App) -> Vec<ContextMenuItem> {
        let rev = revset::change_revision(change);
        let can_squash_into_parent = {
            let vm = self.vm.read(cx);
            change.parents.first().is_some_and(|parent_id| {
                vm.graph
                    .changes
                    .iter()
                    .find(|parent| parent.commit_id.id == *parent_id)
                    .is_none_or(|parent| !parent.is_immutable)
            })
        };
        let mut items = vec![ContextMenuItem::new(
            "New change on top",
            glyph::PLUS_CIRCLE,
            ContextAction::NewChangeOnTop(rev.clone().into()),
        )];
        if !change.is_immutable {
            items.push(ContextMenuItem::new(
                "Edit (modify this change)",
                glyph::PENCIL_CIRCLE,
                change_action(ChangeAction::Edit { rev: rev.clone() }),
            ));
            if can_squash_into_parent {
                items.push(ContextMenuItem::new(
                    "Squash into parent",
                    glyph::ARROW_DOWN,
                    change_action(ChangeAction::Squash {
                        rev: rev.clone(),
                        into: None,
                    }),
                ));
            }
            if !change.is_working_copy {
                items.push(ContextMenuItem::new(
                    "Move changes to working copy",
                    glyph::ARROW_DOWN,
                    change_action(ChangeAction::Squash {
                        rev: rev.clone(),
                        into: Some("@".to_owned()),
                    }),
                ));
            }
        }
        items.extend([
            ContextMenuItem::new(
                "Copy Change ID",
                glyph::COPY,
                ContextAction::CopyText(change.change_id.id.clone().into()),
            ),
            ContextMenuItem::new(
                "Copy Commit ID",
                glyph::COPY,
                ContextAction::CopyText(change.commit_id.id.clone().into()),
            ),
            ContextMenuItem::new(
                "Show History (evolog)",
                glyph::ARROW_CLOCKWISE,
                ContextAction::OpenEvologFor(rev.clone().into()),
            ),
            ContextMenuItem::new(
                "Create bookmark…",
                glyph::BOOKMARK,
                ContextAction::CreateBookmark(rev.clone().into()),
            ),
        ]);
        let (bookmark_diff, selected_rev) = {
            let vm = self.vm.read(cx);
            let selected = vm.selected_change();
            (
                selected.and_then(|base| revset::bookmark_diff_request(base, change)),
                selected
                    .filter(|selected| selected.change_id.id != change.change_id.id)
                    .map(|selected| (revset::change_revision(selected), selected.is_immutable)),
            )
        };
        if let Some(request) = bookmark_diff {
            items.push(ContextMenuItem::new(
                "Show Bookmark Diff",
                glyph::ARROWS_LEFT_RIGHT,
                ContextAction::ShowBookmarkDiff(request),
            ));
        }
        if let Some((selected_rev, selected_immutable)) = selected_rev {
            // Rebase and squash rewrite the selected change; the menu target is only a destination.
            if !selected_immutable {
                items.push(ContextMenuItem::new(
                    "Rebase selected onto this",
                    glyph::ARROW_UP,
                    change_action(ChangeAction::Rebase {
                        rev: selected_rev.clone(),
                        dest: rev.clone(),
                    }),
                ));
                if !change.is_immutable {
                    items.push(ContextMenuItem::new(
                        "Squash selected into this",
                        glyph::ARROW_DOWN,
                        change_action(ChangeAction::Squash {
                            rev: selected_rev.clone(),
                            into: Some(rev.clone()),
                        }),
                    ));
                }
            }
            items.push(ContextMenuItem::new(
                "Merge with selected",
                glyph::GIT_MERGE,
                change_action(ChangeAction::Merge {
                    parents: vec![selected_rev, rev.clone()],
                }),
            ));
        }
        items.push(ContextMenuItem::new(
            "Duplicate",
            glyph::COPY,
            change_action(ChangeAction::Duplicate { rev: rev.clone() }),
        ));
        if !change.is_immutable {
            items.push(ContextMenuItem::new(
                "Absorb into ancestors",
                glyph::ARROW_DOWN,
                change_action(ChangeAction::Absorb { rev: rev.clone() }),
            ));
        }
        items.push(ContextMenuItem::new(
            "Revert change",
            glyph::ARROW_CLOCKWISE,
            change_action(ChangeAction::Revert { rev: rev.clone() }),
        ));
        if !change.is_immutable {
            items.push(ContextMenuItem::new(
                "Stacked Pull Requests…",
                glyph::GIT_BRANCH,
                ContextAction::OpenStackedPr(rev.clone().into()),
            ));
            let label = if change.is_divergent {
                "Abandon (resolve divergence)"
            } else {
                "Abandon"
            };
            items.push(ContextMenuItem::new(
                label,
                glyph::X_CIRCLE,
                ContextAction::AbandonChange(rev.into()),
            ));
        }
        items
    }

    pub(crate) fn run_change_action(&mut self, action: Arc<ChangeAction>, cx: &mut Context<Self>) {
        let task = match action.as_ref() {
            ChangeAction::Edit { rev } => {
                self.vm.update(cx, |vm, cx| vm.edit_change(rev.clone(), cx))
            }
            ChangeAction::Squash { rev, into } => self
                .vm
                .update(cx, |vm, cx| vm.squash_change(rev.clone(), into.clone(), cx)),
            ChangeAction::Rebase { rev, dest } => self
                .vm
                .update(cx, |vm, cx| vm.rebase_change(rev.clone(), dest.clone(), cx)),
            ChangeAction::Merge { parents } => self
                .vm
                .update(cx, |vm, cx| vm.merge_changes(parents.clone(), cx)),
            ChangeAction::Duplicate { rev } => self
                .vm
                .update(cx, |vm, cx| vm.duplicate_change(rev.clone(), cx)),
            ChangeAction::Absorb { rev } => self
                .vm
                .update(cx, |vm, cx| vm.absorb_change(rev.clone(), cx)),
            ChangeAction::Revert { rev } => self
                .vm
                .update(cx, |vm, cx| vm.revert_change(rev.clone(), cx)),
        };
        task.detach();
    }
}

pub(super) fn change_action(action: ChangeAction) -> ContextAction {
    ContextAction::Change(Arc::new(action))
}
