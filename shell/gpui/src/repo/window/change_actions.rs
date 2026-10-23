use std::sync::Arc;

use gpui::{App, Context};
use jayjay_core::compare;
use jayjay_core::dag::MergeParentChoice;
use jayjay_core::{ChangeInfo, InsertPosition, MutationEffect, RebaseMode};

use super::RepoWindow;
use super::confirmation::{Confirmation, ConfirmedAction};
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
        if change.new_change.on_top {
            items.push(ContextMenuItem::new(
                jayjay_i18n::menu::new_change_on_top(),
                glyph::PLUS,
                ContextAction::NewChangeOnTop(rev.clone().into()),
            ));
        }
        if change.new_change.before {
            items.push(ContextMenuItem::new(
                jayjay_i18n::menu::new_change_before(),
                glyph::ARROW_DOWN,
                change_action(ChangeAction::Insert {
                    rev: rev.clone(),
                    at: InsertPosition::Before,
                }),
            ));
        }
        if change.new_change.after {
            items.push(ContextMenuItem::new(
                jayjay_i18n::menu::new_change_after(),
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
                jayjay_i18n::menu::edit_modify_this_change(),
                glyph::PENCIL,
                change_action(ChangeAction::Edit { rev: rev.clone() }),
            ));
            if !change.is_working_copy {
                items.push(ContextMenuItem::new(
                    jayjay_i18n::menu::describe_ellipsis(),
                    glyph::EDIT_CODE,
                    ContextAction::EditDescription {
                        rev: rev.clone().into(),
                        description: change.description.clone().into(),
                    },
                ));
            }
            if can_squash_into_parent {
                items.push(ContextMenuItem::new(
                    jayjay_i18n::menu::squash_into_parent(),
                    glyph::ARROW_DOWN,
                    change_action(ChangeAction::Squash {
                        rev: rev.clone(),
                        into: None,
                    }),
                ));
            }
            if !change.is_working_copy {
                items.push(
                    ContextMenuItem::new(
                        jayjay_i18n::menu::move_changes_to_working_copy(),
                        glyph::ARROW_DOWN,
                        change_action(ChangeAction::Squash {
                            rev: rev.clone(),
                            into: Some("@".to_owned()),
                        }),
                    )
                    .with_shortcut('m'),
                );
            }
            items.push(ContextMenuItem::new(
                jayjay_i18n::menu::rebase_onto_trunk(),
                glyph::ARROW_UP,
                change_action(ChangeAction::RebaseOntoTrunk { rev: rev.clone() }),
            ));
        }

        if !divergent_compares.is_empty() || bookmark_diff.is_some() || selected_rev.is_some() {
            items.push(ContextMenuItem::separator());
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
            jayjay_i18n::menu::create_bookmark_here_ellipsis(),
            glyph::BOOKMARK,
            ContextAction::CreateBookmark(rev.clone().into()),
        ));
        items.push(ContextMenuItem::new(
            jayjay_i18n::menu::create_tag_here_ellipsis(),
            glyph::TAG,
            ContextAction::CreateTag(change.commit_id.id.clone().into()),
        ));
        if !change.is_immutable {
            items.push(ContextMenuItem::new(
                jayjay_i18n::menu::create_update_stacked_prs_ellipsis(),
                glyph::GIT_BRANCH,
                ContextAction::OpenStackedPr(rev.clone().into()),
            ));
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
            jayjay_i18n::menu::show_evolution_ellipsis(),
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
            jayjay_i18n::menu::duplicate(),
            glyph::COPY,
            change_action(ChangeAction::Duplicate { rev: rev.clone() }),
        )];
        if !change.is_immutable {
            more_actions.push(ContextMenuItem::new(
                jayjay_i18n::menu::absorb_into_ancestors(),
                glyph::ARROW_DOWN,
                change_action(ChangeAction::Absorb { rev: rev.clone() }),
            ));
        }
        more_actions.push(ContextMenuItem::new(
            jayjay_i18n::menu::revert_change(),
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

        if !change.is_immutable {
            items.push(
                ContextMenuItem::new(
                    jayjay_i18n::menu::run_formatters_jj_fix(),
                    glyph::BRACES,
                    change_action(ChangeAction::Fix {
                        revs: vec![rev.clone()],
                    }),
                )
                .with_disabled_reason(self.vm.read(cx).fix_unavailable_reason.clone()),
            );
            items.push(ContextMenuItem::separator());
            let label = if change.is_divergent {
                jayjay_i18n::menu::abandon_resolve_divergence()
            } else {
                jayjay_i18n::menu::abandon()
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
            ChangeAction::Rebase { rev, dest } => self.vm.update(cx, |vm, cx| {
                vm.rebase_change(rev.clone(), dest.clone(), RebaseMode::Source, cx)
            }),
            ChangeAction::RebaseOntoTrunk { rev } => self.vm.update(cx, |vm, cx| {
                vm.rebase_change(rev.clone(), "trunk()".to_owned(), RebaseMode::Branch, cx)
            }),
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
                        title: jayjay_i18n::repo::squash_changes_title(count).into(),
                        message: jayjay_i18n::menu::this_combines_the_selected_linear_range()
                            .into(),
                        confirm_label: jayjay_i18n::menu::squash().into(),
                        action: ConfirmedAction::SquashChanges { revs: revs.clone() },
                        dont_ask_again: None,
                    },
                    cx,
                );
                return;
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
                return;
            }
            ChangeAction::ParallelizeMany { revs } => {
                let task = self
                    .vm
                    .update(cx, |vm, cx| vm.parallelize_changes(revs.clone(), cx));
                cx.spawn(async move |this, cx| {
                    if let Ok(MutationEffect::Unchanged) = task.await {
                        let _ = this.update(cx, move |view, cx| {
                            view.show_toast(
                                jayjay_i18n::menu::nothing_to_parallelize_the_selection_would(),
                                cx,
                            );
                        });
                    }
                })
                .detach();
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
                                jayjay_i18n::menu::nothing_to_absorb_use_squash_into(),
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
            ChangeAction::BisectFrom { commit_id } => {
                self.open_bisect_sheet(Some(commit_id.clone()), cx);
                return;
            }
            ChangeAction::BisectBetween { first, second } => {
                self.vm.update(cx, |vm, cx| {
                    vm.start_bisect_between(first.clone(), second.clone(), cx)
                });
                return;
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
                return;
            }
        };
        task.detach();
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
