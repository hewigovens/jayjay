use std::sync::Arc;

use gpui::{App, Context};
use jayjay_core::{ChangeInfo, InsertPosition, MutationEffect};

use super::RepoWindow;
use super::confirmation::{Confirmation, ConfirmedAction};
use crate::repo::revset;
use crate::ui::context_menu::{ContextAction, ContextMenuItem};
use crate::ui::icons::glyph;

pub enum ChangeAction {
    Edit { rev: String },
    Insert { rev: String, at: InsertPosition },
    Squash { rev: String, into: Option<String> },
    Rebase { rev: String, dest: String },
    RebaseMany { revs: Vec<String>, dest: String },
    Merge { parents: Vec<String> },
    SquashMany { revs: Vec<String> },
    AbandonMany { revs: Vec<String> },
    Duplicate { rev: String },
    Absorb { rev: String },
    Revert { rev: String },
}

impl RepoWindow {
    pub fn build_change_menu(&self, change: &ChangeInfo, cx: &App) -> Vec<ContextMenuItem> {
        let (target_ix, has_multiple_selection, target_is_selected) = {
            let vm = self.vm.read(cx);
            let target_ix = vm
                .graph
                .changes
                .iter()
                .position(|candidate| candidate.commit_id == change.commit_id);
            (
                target_ix,
                vm.has_multiple_change_selection(),
                target_ix.is_some_and(|ix| vm.is_change_selected(ix)),
            )
        };
        if has_multiple_selection && target_is_selected {
            return self.build_multi_change_menu(cx);
        }

        let mut items = self.build_single_change_menu(change, cx);
        if has_multiple_selection {
            let vm = self.vm.read(cx);
            let revisions = vm.selected_revisions();
            let count = revisions.len();
            let enabled = target_ix.is_some_and(|ix| vm.can_rebase_selected_changes_onto(ix));
            items.push(
                ContextMenuItem::new(
                    format!("Rebase {count} selected onto this"),
                    glyph::ARROW_UP,
                    change_action(ChangeAction::RebaseMany {
                        revs: revisions,
                        dest: revset::change_revision(change),
                    }),
                )
                .with_enabled(enabled),
            );
        }
        items
    }

    fn build_multi_change_menu(&self, cx: &App) -> Vec<ContextMenuItem> {
        let vm = self.vm.read(cx);
        let revisions = vm.selected_revisions();
        let count = revisions.len();
        vec![
            ContextMenuItem::new(
                format!("Merge {count} selected"),
                glyph::GIT_MERGE,
                change_action(ChangeAction::Merge {
                    parents: revisions.clone(),
                }),
            )
            .with_enabled(vm.can_merge_selected_changes()),
            ContextMenuItem::new(
                format!("Squash {count} selected…"),
                glyph::ARROW_DOWN,
                change_action(ChangeAction::SquashMany {
                    revs: revisions.clone(),
                }),
            )
            .with_enabled(vm.can_squash_selected_changes()),
            ContextMenuItem::separator(),
            ContextMenuItem::new(
                format!("Abandon {count} selected…"),
                glyph::X_CIRCLE,
                change_action(ChangeAction::AbandonMany { revs: revisions }),
            )
            .with_enabled(vm.can_abandon_selected_changes()),
        ]
    }

    fn build_single_change_menu(&self, change: &ChangeInfo, cx: &App) -> Vec<ContextMenuItem> {
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
        let (bookmark_diff, selected_rev) = {
            let vm = self.vm.read(cx);
            let selected = if vm.has_multiple_change_selection() {
                None
            } else {
                vm.selected_change()
            };
            (
                selected.and_then(|base| revset::bookmark_diff_request(base, change)),
                selected
                    .filter(|selected| selected.change_id.id != change.change_id.id)
                    .map(|selected| {
                        (
                            revset::change_revision(selected),
                            selected.is_immutable,
                            vm.can_merge_selected_change_with(change),
                        )
                    }),
            )
        };

        let mut items = Vec::new();
        if change.new_change.on_top {
            items.push(ContextMenuItem::new(
                "New change on top",
                glyph::PLUS_CIRCLE,
                ContextAction::NewChangeOnTop(rev.clone().into()),
            ));
        }
        if change.new_change.before {
            items.push(ContextMenuItem::new(
                "New change before",
                glyph::ARROW_DOWN,
                change_action(ChangeAction::Insert {
                    rev: rev.clone(),
                    at: InsertPosition::Before,
                }),
            ));
        }
        if change.new_change.after {
            items.push(ContextMenuItem::new(
                "New change after",
                glyph::ARROW_UP,
                change_action(ChangeAction::Insert {
                    rev: rev.clone(),
                    at: InsertPosition::After,
                }),
            ));
        }
        items.push(ContextMenuItem::separator());
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

        if bookmark_diff.is_some() || selected_rev.is_some() {
            items.push(ContextMenuItem::separator());
        }
        if let Some(request) = bookmark_diff {
            items.push(ContextMenuItem::new(
                "Diff Bookmark",
                glyph::ARROWS_LEFT_RIGHT,
                ContextAction::ShowBookmarkDiff(request),
            ));
        }
        if let Some((selected_rev, selected_immutable, can_merge)) = selected_rev {
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
            items.push(
                ContextMenuItem::new(
                    "Merge with selected",
                    glyph::GIT_MERGE,
                    change_action(ChangeAction::Merge {
                        parents: vec![selected_rev, rev.clone()],
                    }),
                )
                .with_enabled(can_merge),
            );
        }

        items.push(ContextMenuItem::separator());
        items.push(ContextMenuItem::new(
            "Create bookmark here...",
            glyph::BOOKMARK,
            ContextAction::CreateBookmark(rev.clone().into()),
        ));
        if !change.is_immutable {
            items.push(ContextMenuItem::new(
                "Create / Update Stacked PRs…",
                glyph::GIT_BRANCH,
                ContextAction::OpenStackedPr(rev.clone().into()),
            ));
        }
        items.push(ContextMenuItem::new(
            "Show evolution…",
            glyph::ARROW_CLOCKWISE,
            ContextAction::OpenEvologFor(rev.clone().into()),
        ));

        items.extend([
            ContextMenuItem::separator(),
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
            ContextMenuItem::separator(),
        ]);

        let mut more_actions = vec![ContextMenuItem::new(
            "Duplicate",
            glyph::COPY,
            change_action(ChangeAction::Duplicate { rev: rev.clone() }),
        )];
        if !change.is_immutable {
            more_actions.push(ContextMenuItem::new(
                "Absorb into ancestors",
                glyph::ARROW_DOWN,
                change_action(ChangeAction::Absorb { rev: rev.clone() }),
            ));
        }
        more_actions.push(ContextMenuItem::new(
            "Revert change",
            glyph::ARROW_CLOCKWISE,
            change_action(ChangeAction::Revert { rev: rev.clone() }),
        ));
        items.push(ContextMenuItem::submenu(
            "More Actions",
            glyph::DOT,
            more_actions,
        ));

        if !change.is_immutable {
            items.push(ContextMenuItem::separator());
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
            ChangeAction::Insert { rev, at } => self
                .vm
                .update(cx, |vm, cx| vm.insert_change(rev.clone(), *at, cx)),
            ChangeAction::Squash { rev, into } => self
                .vm
                .update(cx, |vm, cx| vm.squash_change(rev.clone(), into.clone(), cx)),
            ChangeAction::Rebase { rev, dest } => self
                .vm
                .update(cx, |vm, cx| vm.rebase_change(rev.clone(), dest.clone(), cx)),
            ChangeAction::RebaseMany { revs, dest } => self.vm.update(cx, |vm, cx| {
                vm.rebase_changes(revs.clone(), dest.clone(), cx)
            }),
            ChangeAction::Merge { parents } => self
                .vm
                .update(cx, |vm, cx| vm.merge_changes(parents.clone(), cx)),
            ChangeAction::SquashMany { revs } => {
                let count = revs.len();
                self.request_confirmation(
                    Confirmation {
                        title: format!("Squash {count} Changes?").into(),
                        message: "This combines the selected linear range into its oldest change and abandons the other selected changes. You can undo it with jj op restore.".into(),
                        confirm_label: "Squash".into(),
                        action: ConfirmedAction::SquashChanges { revs: revs.clone() },
                    },
                    cx,
                );
                return;
            }
            ChangeAction::AbandonMany { revs } => {
                let count = revs.len();
                self.request_confirmation(
                    Confirmation {
                        title: format!("Abandon {count} Changes?").into(),
                        message: "This removes the selected changes and reparents their descendants. You can undo it with jj op restore.".into(),
                        confirm_label: format!("Abandon {count}").into(),
                        action: ConfirmedAction::AbandonChanges { revs: revs.clone() },
                    },
                    cx,
                );
                return;
            }
            ChangeAction::Duplicate { rev } => self
                .vm
                .update(cx, |vm, cx| vm.duplicate_change(rev.clone(), cx)),
            ChangeAction::Absorb { rev } => {
                let task = self
                    .vm
                    .update(cx, |vm, cx| vm.absorb_change(rev.clone(), cx));
                cx.spawn(async move |this, cx| {
                    if let Ok(MutationEffect::Unchanged) = task.await {
                        let _ = this.update(cx, move |view, cx| {
                            view.show_toast(
                                "Nothing to absorb. Use Squash into parent instead.",
                                cx,
                            );
                        });
                    }
                })
                .detach();
                return;
            }
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
