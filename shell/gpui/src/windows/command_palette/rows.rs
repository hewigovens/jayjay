//! The single mapping from a palette row index to its source — `ACTIONS` first, then help topics; search, dispatch, and rendering all consume this one ordering.

use std::sync::OnceLock;

use super::actions::{ACTIONS, PaletteAction};
use super::help::{self, HelpTopic};

pub(super) enum PaletteRow {
    Action(&'static PaletteAction),
    Help(&'static HelpTopic),
}

pub(super) fn row(ix: usize) -> Option<PaletteRow> {
    if let Some(action) = ACTIONS.get(ix) {
        return Some(PaletteRow::Action(action));
    }
    help::topic_for_row(ix).map(PaletteRow::Help)
}

/// Fuzzy-search haystacks, one per row in row order; both sources are immutable statics, so the strings are built once per process instead of per keystroke/frame.
pub(super) fn search_candidates() -> &'static [String] {
    static CANDIDATES: OnceLock<Vec<String>> = OnceLock::new();
    CANDIDATES.get_or_init(|| {
        let mut candidates: Vec<String> = ACTIONS
            .iter()
            .map(|a| format!("{} {}", a.name, a.keywords.join(" ")))
            .collect();
        candidates.extend(help::topics().iter().map(HelpTopic::search_text));
        candidates
    })
}
