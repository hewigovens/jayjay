//! Per-window cache for the flattened, visibility-filtered file tree, keyed on
//! `(hunks identity, visible indices, collapsed_dirs)`.
//!
//! Without a cache, per-mouse-move notifies during a drag would rebuild the whole
//! tree each event. `hunks` identity is the live `Arc<Vec<DiffHunk>>` address of the
//! *unfiltered* file list (`vm.files`), which stays stable across renders; callers
//! must not pre-filter into a transient `Arc` before calling `visible`, since a
//! freshly allocated `Arc` has no stable address to key on. `visible_indices` and
//! `collapsed` are compared by content since callers rebuild those each render.

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
    visible_indices: Vec<usize>,
    collapsed: HashSet<String>,
    visible: Arc<Vec<FileTreeEntry>>,
}

impl FileTreeCache {
    /// Visible tree entries for `hunks[visible_indices]` under `collapsed`, reusing
    /// the cached value when the file set, the visible subset, and the collapsed
    /// dirs are all unchanged.
    pub(crate) fn visible(
        &mut self,
        hunks: &Arc<Vec<DiffHunk>>,
        visible_indices: &Arc<Vec<usize>>,
        collapsed: &HashSet<String>,
    ) -> Arc<Vec<FileTreeEntry>> {
        let identity = Arc::as_ptr(hunks) as usize;
        if let Some(entry) = &self.entry
            && entry.identity == identity
            && &entry.visible_indices == visible_indices.as_ref()
            && &entry.collapsed == collapsed
        {
            return entry.visible.clone();
        }
        let paths: Vec<String> = visible_indices
            .iter()
            .filter_map(|&ix| hunks.get(ix))
            .map(|h| h.path.clone())
            .collect();
        let visible: Arc<Vec<FileTreeEntry>> = Arc::new(
            build_file_tree(&paths)
                .into_iter()
                .filter(|e| is_entry_visible(e, collapsed))
                .collect(),
        );
        self.entry = Some(Entry {
            identity,
            visible_indices: visible_indices.as_ref().clone(),
            collapsed: collapsed.clone(),
            visible: visible.clone(),
        });
        visible
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jayjay_core::{DiffContent, HunkType};

    fn hunks(paths: &[&str]) -> Arc<Vec<DiffHunk>> {
        Arc::new(
            paths
                .iter()
                .map(|p| DiffHunk {
                    path: (*p).to_owned(),
                    old_path: None,
                    old: DiffContent::default(),
                    new: DiffContent::default(),
                    hunk_type: HunkType::Modified,
                    supports_conflict_editor: false,
                    supports_file_editor: false,
                    review_identity: (*p).to_owned(),
                    projection: None,
                })
                .collect(),
        )
    }

    fn all_indices(hunks: &Arc<Vec<DiffHunk>>) -> Arc<Vec<usize>> {
        Arc::new((0..hunks.len()).collect())
    }

    #[test]
    fn reuses_same_allocation_on_hit() {
        let h = hunks(&["src/a.rs", "src/b.rs"]);
        let indices = all_indices(&h);
        let collapsed = HashSet::new();
        let mut cache = FileTreeCache::default();
        let first = cache.visible(&h, &indices, &collapsed);
        let second = cache.visible(&h, &indices, &collapsed);
        assert!(
            Arc::ptr_eq(&first, &second),
            "same key should reuse the Arc"
        );
    }

    #[test]
    fn rebuilds_when_collapsed_changes() {
        let h = hunks(&["src/a.rs", "src/b.rs"]);
        let indices = all_indices(&h);
        let mut cache = FileTreeCache::default();
        let first = cache.visible(&h, &indices, &HashSet::new());
        let collapsed = HashSet::from(["src".to_owned()]);
        let second = cache.visible(&h, &indices, &collapsed);
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
        let first = cache.visible(&one, &all_indices(&one), &collapsed);
        let second = cache.visible(&two, &all_indices(&two), &collapsed);
        assert!(!Arc::ptr_eq(&first, &second), "new file set should rebuild");
        assert!(second.len() > first.len(), "added file enlarges the tree");
    }

    #[test]
    fn rebuilds_when_only_visible_indices_change_on_a_fresh_arc_each_call() {
        // Regression test: callers (e.g. the "hide reviewed" filter) rebuild the
        // `visible_indices` Arc every render even though the underlying `hunks`
        // Arc from `vm.files` stays the same address. The cache must key off the
        // *content* of visible_indices, not its (unstable) address, or a filtered
        // render can serve a stale tree built for a different visible set.
        let h = hunks(&["a.rs", "b.rs", "c.rs"]);
        let collapsed = HashSet::new();
        let mut cache = FileTreeCache::default();

        let all = Arc::new(vec![0, 1, 2]);
        let first = cache.visible(&h, &all, &collapsed);
        assert_eq!(first.len(), 3);

        // A fresh Arc with the same *content* as `all` should still hit the cache.
        let all_again = Arc::new(vec![0, 1, 2]);
        let hit = cache.visible(&h, &all_again, &collapsed);
        assert!(Arc::ptr_eq(&first, &hit), "same content should reuse cache");

        // A fresh Arc with different content (b.rs filtered out) must rebuild,
        // even though `h`'s address is unchanged.
        let filtered = Arc::new(vec![0, 2]);
        let second = cache.visible(&h, &filtered, &collapsed);
        assert!(
            !Arc::ptr_eq(&first, &second),
            "different visible indices must rebuild, not reuse the stale tree"
        );
        assert_eq!(second.len(), 2);
    }
}
