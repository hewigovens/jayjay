use std::collections::HashMap;

use gpui::SharedString;

/// `Line` is an index into the cached wrapped lines; owning lines here would reclone every span per render.
#[derive(Clone, Debug, PartialEq)]
pub enum DiffRenderRow {
    Line(usize),
    NoteText {
        note_id: SharedString,
        text: SharedString,
        is_first: bool,
        is_last: bool,
    },
}

/// Stale/orphaned notes get no dot — their anchor no longer matches the diff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteDotKind {
    Active,
    Resolved,
}

#[derive(Default)]
pub struct DiffRenderRows {
    pub rows: Vec<DiffRenderRow>,
    /// Keyed by wrapped-fragment index.
    pub dots: HashMap<usize, NoteDotKind>,
    /// Anchor-line indent per `NoteText` row, keyed by the row's own index in `rows` (unlike `dots`).
    pub(crate) note_indents: HashMap<usize, u32>,
}
