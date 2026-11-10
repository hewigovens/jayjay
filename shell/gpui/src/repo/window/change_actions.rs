use std::sync::Arc;

use gpui::{App, Context};
use jayjay_core::compare;
use jayjay_core::dag::{ChangeCommand, MergeParentChoice, SelectionCommand};
use jayjay_core::{ChangeInfo, InsertPosition, MutationEffect, RebaseMode};

use super::RepoWindow;
use super::confirmation::{Confirmation, ConfirmedAction};
use super::dag_drag::DagDrag;
use super::rebase_subject::RebaseSubject;
use crate::ui::context_menu::{ContextAction, ContextMenuItem};
use crate::ui::icons::glyph;

pub enum ChangeAction {
    Edit { rev: String },
    Insert { rev: String, at: InsertPosition },
    Squash { rev: String, into: Option<String> },
    Rebase { rev: String, dest: String },
    RebaseOntoTrunk { rev: String },
    RebaseMany { revs: Vec<String>, dest: String },
    Merge { parents: Vec<String> },
    SquashMany { revs: Vec<String> },
    ParallelizeMany { revs: Vec<String> },
    AbandonMany { revs: Vec<String> },
    Duplicate { rev: String },
    Absorb { rev: String },
    Revert { rev: String },
    Fix { revs: Vec<String> },
    BisectFrom { commit_id: String },
    BisectBetween { first: String, second: String },
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
                    jayjay_i18n::menu::rebase_count_selected_onto_this(count),
                    glyph::ARROW_UP,
                    change_action(ChangeAction::RebaseMany {
                        revs: revisions,
                        dest: change.selection_revision().to_owned(),
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
        let can_merge = vm.can_merge_selected_changes();
        let changes: Vec<_> = if can_merge {
            vm.selected_change_indices()
                .into_iter()
                .map(|index| vm.graph.changes[index].clone())
                .collect()
        } else {
            Vec::new()
        };
        let bisect_ends: Vec<String> = if count == 2 {
            vm.selected_change_indices()
                .into_iter()
                .filter_map(|index| {
                    vm.graph
                        .changes
                        .get(index)
                        .map(|change| change.commit_id.id.clone())
                })
                .collect()
        } else {
            Vec::new()
        };
        let mut choices = vec![
            ContextMenuItem::new(jayjay_i18n::menu::first_parent(), "", ContextAction::Noop)
                .with_enabled(false),
        ];
        choices.extend(
            MergeParentChoice::for_selection(&changes)
                .into_iter()
                .map(|choice| {
                    ContextMenuItem::new(
                        choice.label,
                        glyph::GIT_MERGE,
                        change_action(ChangeAction::Merge {
                            parents: choice.parents,
                        }),
                    )
                }),
        );
        let mut items = vec![
            ContextMenuItem::submenu(
                jayjay_i18n::menu::merge_selected(count),
                glyph::GIT_MERGE,
                choices,
            )
            .with_enabled(can_merge),
            ContextMenuItem::new(
                jayjay_i18n::menu::squash_selected(count),
                glyph::ARROW_DOWN,
                change_action(ChangeAction::SquashMany {
                    revs: revisions.clone(),
                }),
            )
            .with_enabled(vm.can_squash_selected_changes()),
            ContextMenuItem::new(
                jayjay_i18n::menu::parallelize_selected(count),
                glyph::GIT_BRANCH,
                change_action(ChangeAction::ParallelizeMany {
                    revs: revisions.clone(),
                }),
            )
            .with_enabled(vm.can_parallelize_selected_changes()),
            ContextMenuItem::new(
                jayjay_i18n::menu::fix_selected(count),
                glyph::BRACES,
                change_action(ChangeAction::Fix {
                    revs: revisions.clone(),
                }),
            )
            .with_enabled(vm.selection_state().can_fix)
            .with_disabled_reason(vm.fix_unavailable_reason.clone()),
            ContextMenuItem::new(
                SelectionCommand::RebaseOntoTrunk.title(count),
                glyph::ARROW_UP,
                change_action(ChangeAction::RebaseMany {
                    revs: revisions.clone(),
                    dest: "trunk()".to_owned(),
                }),
            )
            .with_enabled(
                vm.selection_state()
                    .available_commands
                    .contains(&SelectionCommand::RebaseOntoTrunk),
            ),
            ContextMenuItem::separator(),
            ContextMenuItem::new(
                jayjay_i18n::menu::abandon_selected(count),
                glyph::X_CIRCLE,
                change_action(ChangeAction::AbandonMany { revs: revisions }),
            )
            .with_enabled(vm.can_abandon_selected_changes()),
        ];
        if let [first, second] = bisect_ends.as_slice() {
            items.insert(
                items.len().saturating_sub(1),
                ContextMenuItem::new(
                    jayjay_i18n::menu::bisect_between(),
                    glyph::ARROWS_LEFT_RIGHT,
                    change_action(ChangeAction::BisectBetween {
                        first: first.clone(),
                        second: second.clone(),
                    }),
                ),
            );
        }
        items
    }

    fn build_single_change_menu(&self, change: &ChangeInfo, cx: &App) -> Vec<ContextMenuItem> {
        let rev = change.selection_revision().to_owned();
        let available = |command: ChangeCommand| command.is_available(change);
        let (bookmark_diff, selected_rev, compare_with_selected, divergent_compares) = {
            let vm = self.vm.read(cx);
            let selected = if vm.has_multiple_change_selection() {
                None
            } else {
                vm.selected_change()
            };
            let selected_other =
                selected.filter(|selected| selected.change_id.id != change.change_id.id);
            (
                selected.and_then(|base| compare::BookmarkDiffRequest::between(base, change)),
                selected_other.map(|selected| {
                    (
                        selected.selection_revision().to_owned(),
                        selected.is_immutable,
                        vm.can_merge_selected_change_with(change),
                    )
                }),
                selected_other.map(|selected| compare::CompareState::between(selected, change)),
                divergent_compares(&vm.graph.changes, change),
            )
        };

        let mut items = Vec::new();
        if change.is_working_copy && self.vm.read(cx).working_copy_stale {
            items.push(ContextMenuItem::new(
                jayjay_i18n::menu::update_workspace(),
                glyph::WARNING,
                ContextAction::UpdateWorkspace,
            ));
            items.push(ContextMenuItem::separator());
        }
        if available(ChangeCommand::NewChange) {
            items.push(ContextMenuItem::new(
                ChangeCommand::NewChange.title(),
                glyph::PLUS,
                ContextAction::NewChangeOnTop(rev.clone().into()),
            ));
        }
        if available(ChangeCommand::NewChangeBefore) {
            items.push(ContextMenuItem::new(
                ChangeCommand::NewChangeBefore.title(),
                glyph::ARROW_DOWN,
                change_action(ChangeAction::Insert {
                    rev: rev.clone(),
                    at: InsertPosition::Before,
                }),
            ));
        }
        if available(ChangeCommand::NewChangeAfter) {
            items.push(ContextMenuItem::new(
                ChangeCommand::NewChangeAfter.title(),
                glyph::ARROW_UP,
                change_action(ChangeAction::Insert {
                    rev: rev.clone(),
                    at: InsertPosition::After,
                }),
            ));
        }
        items.push(ContextMenuItem::separator());
        if available(ChangeCommand::Edit) {
            items.push(ContextMenuItem::new(
                ChangeCommand::Edit.title(),
                glyph::PENCIL,
                change_action(ChangeAction::Edit { rev: rev.clone() }),
            ));
        }
        if available(ChangeCommand::Describe) {
            items.push(ContextMenuItem::new(
                ChangeCommand::Describe.title(),
                glyph::EDIT_CODE,
                ContextAction::EditDescription {
                    rev: rev.clone().into(),
                    description: change.description.clone().into(),
                },
            ));
        }
        if available(ChangeCommand::SquashIntoParent) {
            items.push(ContextMenuItem::new(
                ChangeCommand::SquashIntoParent.title(),
                glyph::ARROW_DOWN,
                change_action(ChangeAction::Squash {
                    rev: rev.clone(),
                    into: None,
                }),
            ));
        }
        if available(ChangeCommand::MoveChangesToWorkingCopy) {
            items.push(
                ContextMenuItem::new(
                    ChangeCommand::MoveChangesToWorkingCopy.title(),
                    glyph::ARROW_DOWN,
                    change_action(ChangeAction::Squash {
                        rev: rev.clone(),
                        into: Some("@".to_owned()),
                    }),
                )
                .with_shortcut('m'),
            );
        }
        if available(ChangeCommand::RebaseOntoTrunk) {
            items.push(ContextMenuItem::new(
                ChangeCommand::RebaseOntoTrunk.title(),
                glyph::ARROW_UP,
                change_action(ChangeAction::RebaseOntoTrunk { rev: rev.clone() }),
            ));
        }

        if change.is_divergent || bookmark_diff.is_some() || selected_rev.is_some() {
            items.push(ContextMenuItem::separator());
        }
        if change.is_divergent {
            items.push(ContextMenuItem::new(
                jayjay_i18n::converge::menu(),
                glyph::GIT_MERGE,
                ContextAction::ResolveDivergence(change.change_id.id.clone().into()),
            ));
        }
        if divergent_compares.len() == 1 {
            items.push(ContextMenuItem::new(
                jayjay_i18n::menu::compare_divergent_version(),
                glyph::ARROWS_LEFT_RIGHT,
                ContextAction::ShowCompare(divergent_compares[0].1.clone()),
            ));
        } else if !divergent_compares.is_empty() {
            items.push(ContextMenuItem::submenu(
                jayjay_i18n::menu::compare_divergent_versions(),
                glyph::ARROWS_LEFT_RIGHT,
                divergent_compares
                    .into_iter()
                    .map(|(label, compare)| {
                        ContextMenuItem::new(
                            label,
                            glyph::ARROWS_LEFT_RIGHT,
                            ContextAction::ShowCompare(compare),
                        )
                    })
                    .collect(),
            ));
        }
        if let Some(compare) = compare_with_selected {
            items.push(ContextMenuItem::new(
                jayjay_i18n::menu::compare_with_selected(),
                glyph::ARROWS_LEFT_RIGHT,
                ContextAction::ShowCompare(compare),
            ));
        }
        if let Some(request) = bookmark_diff {
            items.push(ContextMenuItem::new(
                jayjay_i18n::menu::diff_bookmark(),
                glyph::ARROWS_LEFT_RIGHT,
                ContextAction::ShowBookmarkDiff(request),
            ));
        }
        if let Some((selected_rev, selected_immutable, can_merge)) = selected_rev {
            // Rebase and squash rewrite the selected change; the menu target is only a destination.
            if !selected_immutable {
                items.push(ContextMenuItem::new(
                    jayjay_i18n::menu::rebase_selected_onto_this(),
                    glyph::ARROW_UP,
                    change_action(ChangeAction::Rebase {
                        rev: selected_rev.clone(),
                        dest: rev.clone(),
                    }),
                ));
                if !change.is_immutable {
                    items.push(ContextMenuItem::new(
                        jayjay_i18n::menu::squash_selected_into_this(),
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
                    jayjay_i18n::menu::merge_with_selected(),
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
            ChangeCommand::CreateBookmark.title(),
            glyph::BOOKMARK,
            ContextAction::CreateBookmark(rev.clone().into()),
        ));
        items.push(ContextMenuItem::new(
            ChangeCommand::CreateTag.title(),
            glyph::TAG,
            ContextAction::CreateTag(change.commit_id.id.clone().into()),
        ));
        if !change.is_immutable {
            items.push(ContextMenuItem::new(
                jayjay_i18n::menu::create_update_stacked_prs_ellipsis(),
                glyph::GIT_BRANCH,
                ContextAction::OpenStackedPr(rev.clone().into()),
            ));
            let vm = self.vm.read(cx);
            if let Some(title) = vm.upload_title.clone() {
                items.push(ContextMenuItem::new(
                    title,
                    glyph::CLOUD,
                    ContextAction::UploadChange(rev.clone().into()),
                ));
            }
            if let Some(title) = vm.gerrit_update_title.clone() {
                items.push(ContextMenuItem::new(
                    title,
                    glyph::ARROW_DOWN,
                    ContextAction::UpdateFromGerrit(rev.clone().into()),
                ));
            }
        }
        for name in &change.tags {
            items.push(ContextMenuItem::submenu(
                jayjay_i18n::repo::tag_named(name),
                glyph::TAG,
                self.build_tag_menu(name, cx),
            ));
        }
        items.push(ContextMenuItem::new(
            jayjay_i18n::menu::show_ancestors_ellipsis(),
            glyph::GIT_BRANCH,
            ContextAction::ShowAncestors(change.commit_id.id.clone().into()),
        ));
        items.push(ContextMenuItem::new(
            ChangeCommand::ShowEvolution.title(),
            glyph::HISTORY,
            ContextAction::OpenEvologFor(rev.clone().into()),
        ));

        items.extend([
            ContextMenuItem::separator(),
            ContextMenuItem::new(
                jayjay_i18n::menu::copy_change_id(),
                glyph::COPY,
                ContextAction::CopyText(change.change_id.id.clone().into()),
            )
            .with_shortcut('c'),
            ContextMenuItem::new(
                jayjay_i18n::menu::copy_commit_id(),
                glyph::COPY,
                ContextAction::CopyText(change.commit_id.id.clone().into()),
            ),
            ContextMenuItem::separator(),
        ]);

        let mut more_actions = vec![ContextMenuItem::new(
            ChangeCommand::Duplicate.title(),
            glyph::COPY,
            change_action(ChangeAction::Duplicate { rev: rev.clone() }),
        )];
        if available(ChangeCommand::Absorb) {
            more_actions.push(ContextMenuItem::new(
                ChangeCommand::Absorb.title(),
                glyph::ARROW_DOWN,
                change_action(ChangeAction::Absorb { rev: rev.clone() }),
            ));
        }
        more_actions.push(ContextMenuItem::new(
            ChangeCommand::Revert.title(),
            glyph::HISTORY,
            change_action(ChangeAction::Revert { rev: rev.clone() }),
        ));
        items.push(ContextMenuItem::submenu(
            jayjay_i18n::menu::more_actions(),
            glyph::DOT,
            more_actions,
        ));
        items.push(ContextMenuItem::separator());
        items.push(ContextMenuItem::new(
            jayjay_i18n::menu::bisect_from_here_ellipsis(),
            glyph::ARROWS_LEFT_RIGHT,
            change_action(ChangeAction::BisectFrom {
                commit_id: change.commit_id.id.clone(),
            }),
        ));

        if available(ChangeCommand::Fix) {
            items.push(
                ContextMenuItem::new(
                    ChangeCommand::Fix.title(),
                    glyph::BRACES,
                    change_action(ChangeAction::Fix {
                        revs: vec![rev.clone()],
                    }),
                )
                .with_disabled_reason(self.vm.read(cx).fix_unavailable_reason.clone()),
            );
        }
        if available(ChangeCommand::Abandon) {
            items.push(ContextMenuItem::separator());
            let label = if change.is_divergent {
                jayjay_i18n::menu::abandon_resolve_divergence()
            } else {
                ChangeCommand::Abandon.title()
            };
            items.push(ContextMenuItem::new(
                label,
                glyph::X_CIRCLE,
                ContextAction::AbandonChange(rev.into()),
            ));
        }
        items
    }

    pub(crate) fn parallelize_selection(&mut self, cx: &mut Context<Self>) {
        let (revs, eligible) = {
            let vm = self.vm.read(cx);
            (
                vm.selected_revisions(),
                vm.can_parallelize_selected_changes(),
            )
        };
        if !eligible {
            self.show_toast(jayjay_i18n::menu::select_two_or_more_mutable_changes(), cx);
            return;
        }
        self.run_change_action(Arc::new(ChangeAction::ParallelizeMany { revs }), cx);
    }

    pub(crate) fn fix_selection(&mut self, cx: &mut Context<Self>) {
        let (revs, eligible) = {
            let vm = self.vm.read(cx);
            (vm.selected_revisions(), vm.selection_state().can_fix)
        };
        if !eligible {
            self.show_toast(jayjay_i18n::menu::select_one_or_more_mutable_changes(), cx);
            return;
        }
        self.run_change_action(Arc::new(ChangeAction::Fix { revs }), cx);
    }

    pub(crate) fn run_change_action(&mut self, action: Arc<ChangeAction>, cx: &mut Context<Self>) {
        match action.as_ref() {
            ChangeAction::Edit { rev } => self
                .vm
                .update(cx, |vm, cx| vm.edit_change(rev.clone(), cx))
                .detach(),
            ChangeAction::Insert { rev, at } => self
                .vm
                .update(cx, |vm, cx| vm.insert_change(rev.clone(), *at, cx))
                .detach(),
            ChangeAction::Squash { rev, into } => {
                let task = self
                    .vm
                    .update(cx, |vm, cx| vm.squash_change(rev.clone(), into.clone(), cx));
                self.notify_rewrite(task, |_| jayjay_i18n::repo::squashed_changes(1), cx);
            }
            ChangeAction::Rebase { rev, dest } => {
                self.rebase_change(rev, dest, RebaseMode::Source, cx);
            }
            ChangeAction::RebaseOntoTrunk { rev } => {
                self.rebase_change(rev, "trunk()", RebaseMode::Branch, cx);
            }
            ChangeAction::RebaseMany { revs, dest } => {
                let subject = RebaseSubject::Changes { count: revs.len() };
                let destination = self.rebase_label(dest, cx);
                let task = self.vm.update(cx, |vm, cx| {
                    vm.rebase_changes(revs.clone(), dest.clone(), cx)
                });
                self.notify_rebase(task, subject, destination, cx);
            }
            ChangeAction::Merge { parents } => {
                let task = self
                    .vm
                    .update(cx, |vm, cx| vm.merge_changes(parents.clone(), cx));
                self.notify_rewrite(task, |_| jayjay_i18n::repo::created_merge_change(), cx);
            }
            ChangeAction::SquashMany { revs } => {
                let count = revs.len();
                self.request_confirmation(
                    Confirmation {
                        title: jayjay_i18n::repo::squash_changes_title(count).into(),
                        message: jayjay_i18n::menu::this_combines_the_selected_linear_range()
                            .into(),
                        confirm_label: jayjay_i18n::menu::squash().into(),
                        action: ConfirmedAction::SquashChanges { revs: revs.clone() },
                        dont_ask_again: None,
                    },
                    cx,
                );
            }
            ChangeAction::AbandonMany { revs } => {
                let count = revs.len();
                self.request_confirmation(
                    Confirmation {
                        title: jayjay_i18n::repo::abandon_changes_title(count).into(),
                        message: jayjay_i18n::menu::this_removes_the_selected_changes_and().into(),
                        confirm_label: jayjay_i18n::repo::abandon_count(count).into(),
                        action: ConfirmedAction::AbandonChanges { revs: revs.clone() },
                        dont_ask_again: None,
                    },
                    cx,
                );
            }
            ChangeAction::ParallelizeMany { revs } => {
                let count = revs.len();
                let task = self
                    .vm
                    .update(cx, |vm, cx| vm.parallelize_changes(revs.clone(), cx));
                self.notify_rewrite(
                    task,
                    move |rewrite| match rewrite.value {
                        MutationEffect::Changed => jayjay_i18n::repo::parallelized_changes(count),
                        MutationEffect::Unchanged => {
                            jayjay_i18n::menu::nothing_to_parallelize_the_selection_would()
                        }
                    },
                    cx,
                );
            }
            ChangeAction::Duplicate { rev } => {
                let task = self
                    .vm
                    .update(cx, |vm, cx| vm.duplicate_change(rev.clone(), cx));
                self.notify_rewrite(task, |_| jayjay_i18n::repo::duplicated_change(), cx);
            }
            ChangeAction::Absorb { rev } => {
                let task = self
                    .vm
                    .update(cx, |vm, cx| vm.absorb_change(rev.clone(), cx));
                self.notify_rewrite(
                    task,
                    |rewrite| match rewrite.value {
                        MutationEffect::Changed => jayjay_i18n::repo::absorbed_into_ancestors(),
                        MutationEffect::Unchanged => {
                            jayjay_i18n::menu::nothing_to_absorb_use_squash_into()
                        }
                    },
                    cx,
                );
            }
            ChangeAction::Revert { rev } => {
                let task = self
                    .vm
                    .update(cx, |vm, cx| vm.revert_change(rev.clone(), cx));
                self.notify_rewrite(task, |_| jayjay_i18n::repo::reverted_change(), cx);
            }
            ChangeAction::BisectFrom { commit_id } => {
                self.open_bisect_sheet(Some(commit_id.clone()), cx);
            }
            ChangeAction::BisectBetween { first, second } => {
                self.vm.update(cx, |vm, cx| {
                    vm.start_bisect_between(first.clone(), second.clone(), cx)
                });
            }
            ChangeAction::Fix { revs } => {
                let task = self
                    .vm
                    .update(cx, |vm, cx| vm.fix_changes(revs.clone(), cx));
                cx.spawn(async move |this, cx| {
                    if let Ok(summary) = task.await
                        && summary.failures.is_empty()
                    {
                        let _ = this.update(cx, move |view, cx| {
                            view.show_toast(summary.message(), cx);
                        });
                    }
                })
                .detach();
            }
        }
    }

    fn rebase_change(&mut self, rev: &str, dest: &str, mode: RebaseMode, cx: &mut Context<Self>) {
        let subject = RebaseSubject::Change {
            label: self.rebase_label(rev, cx),
        };
        let destination = self.rebase_label(dest, cx);
        let task = self.vm.update(cx, |vm, cx| {
            vm.rebase_change(rev.to_owned(), dest.to_owned(), mode, cx)
        });
        self.notify_rebase(task, subject, destination, cx);
    }

    fn rebase_label(&self, rev: &str, cx: &App) -> String {
        let graph = &self.vm.read(cx).graph;
        if rev == "trunk()" {
            return graph.trunk_name.clone();
        }
        graph
            .changes
            .iter()
            .find(|change| change.selection_revision() == rev || change.commit_id.id == rev)
            .map_or_else(|| rev.to_owned(), DagDrag::label_for_change)
    }
}

pub(super) fn change_action(action: ChangeAction) -> ContextAction {
    ContextAction::Change(Arc::new(action))
}

fn divergent_compares(
    changes: &[ChangeInfo],
    change: &ChangeInfo,
) -> Vec<(String, compare::CompareState)> {
    if !change.is_divergent {
        return Vec::new();
    }
    changes
        .iter()
        .filter(|sibling| {
            sibling.change_id.id == change.change_id.id
                && sibling.commit_id.id != change.commit_id.id
        })
        .map(|sibling| {
            (
                divergent_sibling_label(sibling),
                compare::CompareState::between(sibling, change),
            )
        })
        .collect()
}

/// Siblings share a change id, so the commit prefix and first description line tell them apart.
fn divergent_sibling_label(change: &ChangeInfo) -> String {
    let short_commit: String = change.commit_id.id.chars().take(8).collect();
    let first_line = change.description.split('\n').next().unwrap_or("").trim();
    if first_line.is_empty() {
        short_commit
    } else {
        format!("{short_commit} — {first_line}")
    }
}
