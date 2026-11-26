use std::collections::HashMap;
use std::sync::Arc;

use gpui::ScrollHandle;
use jayjay_core::FileCollapse;
use jayjay_core::diff::FileDiff;

use super::plan::StackPlan;
use crate::diff::DiffShape;
use crate::repo::window::DiffWrapCacheSlot;

#[derive(Default)]
pub(crate) struct DiffStackState {
    pub(crate) scroll: ScrollHandle,
    /// A file picked from the list shows alone until every file is asked for again.
    pub(crate) solo: bool,
    pub(crate) collapsed: FileCollapse,
    pub(super) wraps: HashMap<String, DiffWrapCacheSlot>,
    /// Kept with the diff it was read from, so a file is scanned once and not on every render.
    pub(super) shapes: HashMap<String, (Arc<FileDiff>, DiffShape)>,
    /// Body heights by path, exact for files that have been on screen.
    pub(super) heights: HashMap<String, f32>,
    pub(super) largest_line_number: u32,
    pub(super) revision: Option<String>,
    pub(crate) plan: Option<StackPlan>,
}
