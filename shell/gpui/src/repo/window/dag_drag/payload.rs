use std::sync::Arc;

use jayjay_core::{ChangeInfo, GraphEntry};

use super::selection::DagDragSelection;

#[derive(Clone)]
pub(crate) enum DagDrag {
    WorkingCopy,
    Bookmark {
        name: String,
        conflicted: bool,
    },
    Change {
        source_ix: usize,
        entries: Arc<Vec<GraphEntry>>,
        selection: Option<Arc<DagDragSelection>>,
    },
}

impl DagDrag {
    pub(in crate::repo::window) fn for_change(
        ix: usize,
        entries: &Arc<Vec<GraphEntry>>,
        selection: Option<Arc<DagDragSelection>>,
    ) -> Option<Self> {
        (!entries.get(ix)?.change.is_immutable).then(|| Self::Change {
            source_ix: ix,
            entries: entries.clone(),
            selection,
        })
    }

    pub(super) fn selection(&self) -> Option<&DagDragSelection> {
        match self {
            Self::Change { selection, .. } => selection.as_deref(),
            _ => None,
        }
    }

    pub(super) fn source_label(&self) -> String {
        match (self.selection(), self.source_change()) {
            (Some(selection), _) => format!("{} changes", selection.commit_ids.len()),
            (None, Some(source)) => Self::label_for_change(source),
            (None, None) => String::new(),
        }
    }

    pub(super) fn source_change(&self) -> Option<&ChangeInfo> {
        match self {
            Self::Change {
                source_ix, entries, ..
            } => entries.get(*source_ix).map(|entry| &entry.change),
            _ => None,
        }
    }

    pub(super) fn label_for_change(change: &ChangeInfo) -> String {
        change
            .bookmarks
            .first()
            .filter(|bookmark| !bookmark.is_empty())
            .cloned()
            .or_else(|| {
                change
                    .description
                    .lines()
                    .next()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(str::to_owned)
            })
            .or_else(|| change.is_working_copy.then(|| "@".to_owned()))
            .unwrap_or_else(|| change.change_id.prefix(8))
    }

    pub(in crate::repo::window) fn can_drop_on(&self, change: &ChangeInfo) -> bool {
        match self {
            Self::WorkingCopy => !change.is_working_copy,
            Self::Bookmark { name, conflicted } => {
                *conflicted || !change.bookmarks.iter().any(|bookmark| bookmark == name)
            }
            Self::Change {
                selection: Some(selection),
                ..
            } => selection.targets.contains(change.commit_id.as_str()),
            Self::Change { entries, .. } => {
                let Some(source) = self.source_change() else {
                    return false;
                };
                jayjay_core::dag::can_rebase_onto(
                    entries,
                    source.commit_id.as_str(),
                    change.commit_id.as_str(),
                )
            }
        }
    }
}
