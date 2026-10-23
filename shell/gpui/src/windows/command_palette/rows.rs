//! The single mapping from a palette row index to its source — `ACTIONS` first, then help topics, then dynamic rows; search, dispatch, and rendering all consume this one ordering.

use std::borrow::Cow;
use std::sync::{Arc, OnceLock};

use jayjay_core::{ChangeInfo, ShortId, WorkspaceInfo, default_revset_preset, revset_presets};

use super::actions::{ACTIONS, PaletteAction};
use super::help::{self, HelpTopic};
use crate::repo::window::{ChangeAction, RepoWindow, SyncActivity};
use crate::ui::context_menu::ContextAction;
use crate::ui::icons::glyph;

pub(super) struct DynamicRow {
    pub label: String,
    pub search: String,
    pub glyph: &'static str,
    pub selector: String,
    pub kind: DynamicKind,
}

pub(super) enum DynamicKind {
    SwitchWorkspace { path: String },
    Context(ContextAction),
    Window(fn(&mut RepoWindow, &mut gpui::Context<RepoWindow>)),
}

pub(super) enum PaletteRow<'a> {
    Action(&'static PaletteAction),
    Help(&'static HelpTopic),
    Dynamic(&'a DynamicRow),
}

pub(super) fn dynamic_rows(
    workspaces: &[WorkspaceInfo],
    selected: Option<&ChangeInfo>,
    sync: &SyncActivity,
) -> Vec<DynamicRow> {
    let mut rows = cancel_sync_rows(sync);
    for workspace in workspaces.iter().filter(|workspace| !workspace.is_current) {
        if workspace.is_switch_target() {
            rows.push(switch_row(workspace));
        }
        rows.push(forget_row(workspace));
        if workspace.is_path_resolved {
            rows.push(delete_row(workspace));
        }
    }
    rows.extend(revset_rows());
    if let Some(change) = selected {
        rows.extend(change_rows(change));
    }
    rows
}

pub(super) fn row(ix: usize, dynamic: &[DynamicRow]) -> Option<PaletteRow<'_>> {
    if let Some(action) = ACTIONS.get(ix) {
        return Some(PaletteRow::Action(action));
    }
    if let Some(topic) = help::topic_for_row(ix) {
        return Some(PaletteRow::Help(topic));
    }
    let dynamic_ix = ix.checked_sub(ACTIONS.len() + help::topics().len())?;
    dynamic.get(dynamic_ix).map(PaletteRow::Dynamic)
}

/// Fuzzy-search haystacks, one per row in row order; the static sources are built once per process.
pub(super) fn search_candidates(dynamic: &[DynamicRow]) -> Cow<'static, [String]> {
    static CANDIDATES: OnceLock<Vec<String>> = OnceLock::new();
    let base = CANDIDATES.get_or_init(|| {
        let mut candidates: Vec<String> = ACTIONS
            .iter()
            .map(|action| {
                format!(
                    "{} {} {}",
                    (action.label)(),
                    action.id,
                    action.keywords.join(" ")
                )
            })
            .collect();
        candidates.extend(help::topics().iter().map(HelpTopic::search_text));
        candidates
    });
    if dynamic.is_empty() {
        return Cow::Borrowed(base);
    }
    let mut candidates = base.clone();
    candidates.extend(dynamic.iter().map(|row| row.search.clone()));
    Cow::Owned(candidates)
}

fn cancel_sync_rows(sync: &SyncActivity) -> Vec<DynamicRow> {
    let row = |label: &str, selector: &str, cancel| DynamicRow {
        label: label.to_owned(),
        search: format!("{label} git sync stop"),
        glyph: glyph::X_CIRCLE,
        selector: selector.to_owned(),
        kind: DynamicKind::Window(cancel),
    };
    let mut rows = Vec::new();
    if sync.is_fetching() {
        rows.push(row(
            &jayjay_i18n::palette::cancel_pull(),
            "command-palette-cancel-pull",
            RepoWindow::cancel_pull,
        ));
    }
    if sync.is_pushing() {
        rows.push(row(
            &jayjay_i18n::palette::cancel_push(),
            "command-palette-cancel-push",
            RepoWindow::cancel_push,
        ));
    }
    rows
}

fn switch_row(workspace: &WorkspaceInfo) -> DynamicRow {
    let label = jayjay_i18n::palette::switch_to(&workspace.name);
    DynamicRow {
        search: format!("{label} switch workspace"),
        selector: format!("command-palette-workspace-{}", workspace.name),
        glyph: glyph::ARROW_CIRCLE_RIGHT,
        kind: DynamicKind::SwitchWorkspace {
            path: workspace.path.clone(),
        },
        label,
    }
}

fn forget_row(workspace: &WorkspaceInfo) -> DynamicRow {
    let label = jayjay_i18n::palette::forget_workspace_named(&workspace.name);
    DynamicRow {
        search: label.clone(),
        selector: format!("command-palette-forget-{}", workspace.name),
        glyph: glyph::FOLDER_MINUS,
        kind: DynamicKind::Context(ContextAction::ForgetWorkspace {
            name: workspace.name.clone().into(),
            path: workspace
                .is_path_resolved
                .then(|| workspace.path.clone().into()),
        }),
        label,
    }
}

fn delete_row(workspace: &WorkspaceInfo) -> DynamicRow {
    let label = jayjay_i18n::palette::forget_delete_workspace_named(&workspace.name);
    DynamicRow {
        search: label.clone(),
        selector: format!("command-palette-delete-{}", workspace.name),
        glyph: glyph::TRASH,
        kind: DynamicKind::Context(ContextAction::DeleteWorkspace {
            name: workspace.name.clone().into(),
            path: workspace.path.clone().into(),
        }),
        label,
    }
}

fn revset_rows() -> Vec<DynamicRow> {
    let mut rows = revset_presets()
        .iter()
        .map(|preset| {
            revset_row(
                &preset.id,
                jayjay_i18n::palette::show_filter(&preset.label),
                preset.revset.clone(),
            )
        })
        .collect::<Vec<_>>();
    rows.push(revset_row(
        "mutable",
        jayjay_i18n::palette::show_mutable(),
        "mutable()".to_owned(),
    ));
    rows.push(revset_row(
        "reset",
        jayjay_i18n::palette::reset_filter(),
        default_revset_preset().revset,
    ));
    rows
}

fn revset_row(id: &str, label: String, revset: String) -> DynamicRow {
    DynamicRow {
        search: format!("{label} revset filter preset"),
        selector: format!("command-palette-revset-{id}"),
        glyph: glyph::FILTER,
        kind: DynamicKind::Context(ContextAction::FilterRevset(revset.into())),
        label,
    }
}

fn change_rows(change: &ChangeInfo) -> Vec<DynamicRow> {
    let rev = change.selection_revision().to_owned();
    let short = change.change_id.prefix(ShortId::LABEL_CHARS);
    let mut rows = vec![change_row(
        jayjay_i18n::palette::new_child_change(&short),
        "command-palette-new-child",
        glyph::PLUS,
        ContextAction::NewChangeOnTop(rev.clone().into()),
    )];
    if !change.is_immutable {
        rows.push(change_row(
            jayjay_i18n::palette::edit_switch_to(&short),
            "command-palette-edit-change",
            glyph::PENCIL,
            ContextAction::Change(Arc::new(ChangeAction::Edit { rev: rev.clone() })),
        ));
    }
    rows.push(change_row(
        jayjay_i18n::palette::duplicate_change(&short),
        "command-palette-duplicate-change",
        glyph::COPY,
        ContextAction::Change(Arc::new(ChangeAction::Duplicate { rev: rev.clone() })),
    ));
    rows.push(change_row(
        jayjay_i18n::palette::revert_change(&short),
        "command-palette-revert-change",
        glyph::ARROW_UTURN_BACK,
        ContextAction::Change(Arc::new(ChangeAction::Revert { rev: rev.clone() })),
    ));
    rows.push(change_row(
        jayjay_i18n::palette::create_bookmark_on(&short),
        "command-palette-create-bookmark",
        glyph::BOOKMARK,
        ContextAction::CreateBookmark(rev.into()),
    ));
    rows
}

fn change_row(
    label: String,
    selector: &str,
    glyph_str: &'static str,
    action: ContextAction,
) -> DynamicRow {
    DynamicRow {
        search: format!("{label} change"),
        selector: selector.to_owned(),
        glyph: glyph_str,
        kind: DynamicKind::Context(action),
        label,
    }
}
