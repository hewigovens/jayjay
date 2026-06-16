//! Per-window cache for the flattened, visibility-filtered file tree, keyed on
//! `(hunks identity, collapsed_dirs)`.
//!
//! Without a cache, per-mouse-move notifies during a drag would rebuild the whole
//! tree each event. Identity is the live `Arc<Vec<DiffHunk>>` address, so a new
//! file set (or a toggled directory) rekeys.

use std::collections::HashSet;
use std::sync::Arc;

use jayjay_core::file_tree::build_file_tree;
use jayjay_core::{DiffHunk, FileTreeEntry};

use super::tree::is_entry_visible;

#[derive(Default)]
pub(crate) struct FileTreeCache {
    entry: Option<Entry>,
}

struct Entry {
    identity: usize,
    collapsed: HashSet<String>,
    visible: Arc<Vec<FileTreeEntry>>,
}

impl FileTreeCache {
    /// Visible tree entries for `hunks` under `collapsed`, reusing the cached
    /// value when both the file set and the collapsed dirs are unchanged.
    pub(crate) fn visible(
        &mut self,
        hunks: &Arc<Vec<DiffHunk>>,
        collapsed: &HashSet<String>,
    ) -> Arc<Vec<FileTreeEntry>> {
        let identity = Arc::as_ptr(hunks) as usize;
        if let Some(entry) = &self.entry
            && entry.identity == identity
            && &entry.collapsed == collapsed
        {
            return entry.visible.clone();
        }
        let paths: Vec<String> = hunks.iter().map(|h| h.path.clone()).collect();
        let visible: Arc<Vec<FileTreeEntry>> = Arc::new(
            build_file_tree(&paths)
                .into_iter()
                .filter(|e| is_entry_visible(e, collapsed))
                .collect(),
        );
        self.entry = Some(Entry {
            identity,
            collapsed: collapsed.clone(),
            visible: visible.clone(),
        });
        visible
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jayjay_core::HunkType;

    fn hunks(paths: &[&str]) -> Arc<Vec<DiffHunk>> {
        Arc::new(
            paths
                .iter()
                .map(|p| DiffHunk {
                    path: (*p).to_owned(),
                    old_path: None,
                    old_content: None,
                    new_content: None,
                    old_preview: None,
                    new_preview: None,
                    hunk_type: HunkType::Modified,
                    review_identity: (*p).to_owned(),
                })
                .collect(),
        )
    }

    #[test]
    fn reuses_same_allocation_on_hit() {
        let h = hunks(&["src/a.rs", "src/b.rs"]);
        let collapsed = HashSet::new();
        let mut cache = FileTreeCache::default();
        let first = cache.visible(&h, &collapsed);
        let second = cache.visible(&h, &collapsed);
        assert!(
            Arc::ptr_eq(&first, &second),
            "same key should reuse the Arc"
        );
    }

    #[test]
    fn rebuilds_when_collapsed_changes() {
        let h = hunks(&["src/a.rs", "src/b.rs"]);
        let mut cache = FileTreeCache::default();
        let first = cache.visible(&h, &HashSet::new());
        let collapsed = HashSet::from(["src".to_owned()]);
        let second = cache.visible(&h, &collapsed);
        assert!(
            !Arc::ptr_eq(&first, &second),
            "collapse change should rebuild"
        );
        assert!(second.len() < first.len(), "collapsing hides children");
    }

    #[test]
    fn rebuilds_when_file_set_changes() {
        let collapsed = HashSet::new();
        let mut cache = FileTreeCache::default();
        // Hold both file sets live so the second Arc can't reuse the first's
        // freed address and forge a false cache hit, as `vm.files` does at runtime.
        let one = hunks(&["src/a.rs"]);
        let two = hunks(&["src/a.rs", "src/b.rs"]);
        let first = cache.visible(&one, &collapsed);
        let second = cache.visible(&two, &collapsed);
        assert!(!Arc::ptr_eq(&first, &second), "new file set should rebuild");
        assert!(second.len() > first.len(), "added file enlarges the tree");
    }
}
