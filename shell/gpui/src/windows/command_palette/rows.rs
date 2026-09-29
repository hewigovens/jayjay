//! The single mapping from a palette row index to its source — `ACTIONS` first, then help topics, then workspace switches; search, dispatch, and rendering all consume this one ordering.

use std::borrow::Cow;
use std::sync::OnceLock;

use jayjay_core::WorkspaceInfo;

use super::actions::{ACTIONS, PaletteAction};
use super::help::{self, HelpTopic};

pub(super) struct WorkspaceTarget {
    pub name: String,
    pub path: String,
    pub label: String,
}

impl WorkspaceTarget {
    pub(super) fn new(workspace: &WorkspaceInfo) -> Self {
        Self {
            name: workspace.name.clone(),
            path: workspace.path.clone(),
            label: format!("Switch to {}", workspace.name),
        }
    }
}

pub(super) enum PaletteRow<'a> {
    Action(&'static PaletteAction),
    Help(&'static HelpTopic),
    Workspace(&'a WorkspaceTarget),
}

pub(super) fn row(ix: usize, workspaces: &[WorkspaceTarget]) -> Option<PaletteRow<'_>> {
    if let Some(action) = ACTIONS.get(ix) {
        return Some(PaletteRow::Action(action));
    }
    if let Some(topic) = help::topic_for_row(ix) {
        return Some(PaletteRow::Help(topic));
    }
    let workspace_ix = ix.checked_sub(ACTIONS.len() + help::topics().len())?;
    workspaces.get(workspace_ix).map(PaletteRow::Workspace)
}

/// Fuzzy-search haystacks, one per row in row order; the static sources are built once per process.
pub(super) fn search_candidates(workspaces: &[WorkspaceTarget]) -> Cow<'static, [String]> {
    static CANDIDATES: OnceLock<Vec<String>> = OnceLock::new();
    let base = CANDIDATES.get_or_init(|| {
        let mut candidates: Vec<String> = ACTIONS
            .iter()
            .map(|a| format!("{} {}", a.name, a.keywords.join(" ")))
            .collect();
        candidates.extend(help::topics().iter().map(HelpTopic::search_text));
        candidates
    });
    if workspaces.is_empty() {
        return Cow::Borrowed(base);
    }
    let mut candidates = base.clone();
    candidates.extend(
        workspaces
            .iter()
            .map(|workspace| format!("{} switch workspace", workspace.label)),
    );
    Cow::Owned(candidates)
}
