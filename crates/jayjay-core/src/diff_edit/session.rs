use std::collections::{BTreeSet, HashMap, HashSet};

use jayjay_primitives::{
    DiffEditDestination, DiffEditFileSelection, FileDiffStats, diff_edit_auto_collapsed_paths,
    diff_edit_collapses_while_stats_pending, diff_edit_ranges, diff_edit_starts_collapsed,
};

use super::file::DiffEditFile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffEditCheckbox {
    None,
    Some,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DiffEditSelectionSummary {
    pub files: u32,
    pub lines: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DiffEditFileCounts {
    pub selected: u32,
    pub changed: u32,
}

struct LoadedFile {
    file: DiffEditFile,
    changed: BTreeSet<u32>,
    selected: BTreeSet<u32>,
}

/// Selection, collapse and card-focus policy for one Diff Edit session; every line number is a row of the uncollapsed diff.
#[derive(Default)]
pub struct DiffEditSession {
    files: HashMap<String, LoadedFile>,
    select_all_pending: HashSet<String>,
    collapsed: HashSet<String>,
    collapse_touched: bool,
    focused: Option<String>,
}

impl DiffEditSession {
    /// Registers a loaded card. An existing selection keeps only the rows that still changed, and a pending Select All claims the whole file.
    pub fn load(&mut self, file: DiffEditFile) {
        let changed: BTreeSet<u32> = file.changed_lines.iter().copied().collect();
        let path = file.path.clone();
        let selected = if self.select_all_pending.remove(&path) {
            changed.clone()
        } else {
            self.files
                .get(&path)
                .map(|loaded| loaded.selected.intersection(&changed).copied().collect())
                .unwrap_or_default()
        };
        self.files.insert(
            path,
            LoadedFile {
                file,
                changed,
                selected,
            },
        );
    }

    /// Drops a card that cannot be edited, so it never holds Select All open.
    pub fn skip(&mut self, path: &str) {
        self.select_all_pending.remove(path);
        self.files.remove(path);
    }

    /// Forgets every loaded card: row numbers silently remap when the whitespace mode or the commit changes.
    pub fn unload(&mut self) {
        self.files.clear();
        self.select_all_pending.clear();
    }

    pub fn is_loaded(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }

    pub fn selected_lines(&self, path: &str) -> Vec<u32> {
        self.files
            .get(path)
            .map(|loaded| loaded.selected.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn is_selected(&self, path: &str, line: u32) -> bool {
        self.files
            .get(path)
            .is_some_and(|loaded| loaded.selected.contains(&line))
    }

    pub fn toggle_line(&mut self, path: &str, line: u32) {
        let Some(loaded) = self.files.get_mut(path) else {
            return;
        };
        if !loaded.changed.contains(&line) {
            return;
        }
        if !loaded.selected.remove(&line) {
            loaded.selected.insert(line);
        }
    }

    pub fn toggle_file(&mut self, path: &str) {
        let Some(loaded) = self.files.get_mut(path) else {
            return;
        };
        if loaded.changed.is_empty() {
            return;
        }
        if loaded.changed.len() == loaded.selected.len() {
            loaded.selected.clear();
        } else {
            loaded.selected = loaded.changed.clone();
        }
    }

    pub fn select_file(&mut self, path: &str) {
        if let Some(loaded) = self.files.get_mut(path) {
            loaded.selected = loaded.changed.clone();
        }
    }

    /// Adds a hunk's rows to the selection, ignoring the unchanged ones a display range sweeps up.
    pub fn select_lines(&mut self, path: &str, lines: &[u32]) {
        let Some(loaded) = self.files.get_mut(path) else {
            return;
        };
        loaded
            .selected
            .extend(lines.iter().filter(|line| loaded.changed.contains(line)));
    }

    pub fn toggle_all(&mut self, paths: &[String]) -> Vec<String> {
        if self.should_deselect() {
            self.select_all_pending.clear();
            for loaded in self.files.values_mut() {
                loaded.selected.clear();
            }
            return Vec::new();
        }
        let mut pending = Vec::new();
        for path in paths {
            match self.files.get_mut(path) {
                Some(loaded) => loaded.selected = loaded.changed.clone(),
                None => {
                    self.select_all_pending.insert(path.clone());
                    pending.push(path.clone());
                }
            }
        }
        pending
    }

    pub fn is_selecting_all(&self) -> bool {
        !self.select_all_pending.is_empty()
    }

    pub fn has_selection(&self) -> bool {
        self.files
            .values()
            .any(|loaded| !loaded.selected.is_empty())
    }

    /// True when the bulk control should offer Deselect All: a pending Select All counts, so the label does not flip back mid-load.
    pub fn should_deselect(&self) -> bool {
        self.is_selecting_all() || self.has_selection()
    }

    pub fn checkbox(&self, path: &str) -> DiffEditCheckbox {
        let Some(loaded) = self.files.get(path) else {
            return DiffEditCheckbox::None;
        };
        let count = loaded.selected.len();
        if count == 0 {
            DiffEditCheckbox::None
        } else if count == loaded.changed.len() {
            DiffEditCheckbox::All
        } else {
            DiffEditCheckbox::Some
        }
    }

    pub fn file_counts(&self, path: &str) -> DiffEditFileCounts {
        self.files
            .get(path)
            .map(|loaded| DiffEditFileCounts {
                selected: loaded.selected.len() as u32,
                changed: loaded.changed.len() as u32,
            })
            .unwrap_or_default()
    }

    pub fn summary(&self) -> DiffEditSelectionSummary {
        let mut summary = DiffEditSelectionSummary::default();
        for loaded in self.files.values() {
            let lines = loaded.selected.len() as u32;
            if lines > 0 {
                summary.files += 1;
                summary.lines += lines;
            }
        }
        summary
    }

    pub fn summary_text(&self) -> String {
        let DiffEditSelectionSummary { files, lines } = self.summary();
        if lines == 0 {
            return "Select files, hunks, or line ranges to edit".to_owned();
        }
        format!(
            "{files} {}, {lines} {} selected",
            if files == 1 { "file" } else { "files" },
            if lines == 1 { "line" } else { "lines" }
        )
    }

    /// The apply request in card order; invert for `RemoveFromSource` so the rewrite removes the unselected rows.
    pub fn selections(
        &self,
        paths: &[String],
        destination: DiffEditDestination,
    ) -> Vec<DiffEditFileSelection> {
        let inverse = destination == DiffEditDestination::RemoveFromSource;
        paths
            .iter()
            .filter_map(|path| {
                let loaded = self.files.get(path)?;
                let lines: Vec<u32> = if inverse {
                    loaded
                        .changed
                        .difference(&loaded.selected)
                        .copied()
                        .collect()
                } else {
                    loaded.selected.iter().copied().collect()
                };
                (!lines.is_empty()).then(|| DiffEditFileSelection {
                    path: loaded.file.path.clone(),
                    old_path: loaded.file.old_path.clone(),
                    old_content: loaded.file.old_content.clone(),
                    new_content: loaded.file.new_content.clone(),
                    hunk_type: loaded.file.hunk_type,
                    line_ranges: diff_edit_ranges(lines),
                })
            })
            .collect()
    }

    /// Seeds collapse from the whole-change stats so a large diff never renders expanded while the per-file pass runs.
    pub fn seed_collapse(&mut self, paths: &[String], total_changed_lines: Option<u64>) {
        let collapse_all = match total_changed_lines {
            Some(total) => diff_edit_starts_collapsed(paths.len(), total),
            None => diff_edit_collapses_while_stats_pending(paths.len()),
        };
        if collapse_all {
            self.collapsed = paths.iter().cloned().collect();
        }
    }

    /// Replaces the seed with the per-file policy, unless the user already folded something by hand.
    pub fn apply_stats(&mut self, paths: &[String], stats: &[FileDiffStats]) {
        if self.collapse_touched {
            return;
        }
        let cards: Vec<String> = if paths.is_empty() {
            stats.iter().map(|file| file.path.clone()).collect()
        } else {
            paths.to_vec()
        };
        // Synthetic cards (dirty-only submodules) are absent from the jj tree diff, so count them or the file threshold disagrees with what is on screen.
        let known: HashSet<&str> = stats.iter().map(|file| file.path.as_str()).collect();
        let mut policy = stats.to_vec();
        policy.extend(
            cards
                .iter()
                .filter(|path| !known.contains(path.as_str()))
                .map(|path| FileDiffStats {
                    path: path.clone(),
                    insertions: 0,
                    deletions: 0,
                }),
        );
        let total: u64 = policy
            .iter()
            .map(|file| u64::from(file.insertions) + u64::from(file.deletions))
            .sum();
        self.collapsed = if diff_edit_starts_collapsed(policy.len(), total) {
            cards.into_iter().collect()
        } else {
            diff_edit_auto_collapsed_paths(&policy)
                .into_iter()
                .collect()
        };
    }

    pub fn toggle_collapse(&mut self, path: &str) {
        self.collapse_touched = true;
        if !self.collapsed.remove(path) {
            self.collapsed.insert(path.to_owned());
        }
    }

    /// Collapses or expands the given card; false when it already was, so a shell can leave a key press unhandled.
    pub fn set_collapsed(&mut self, path: &str, collapsed: bool) -> bool {
        if self.collapsed.contains(path) == collapsed {
            return false;
        }
        self.toggle_collapse(path);
        true
    }

    pub fn collapse_all(&mut self, paths: &[String]) {
        self.collapse_touched = true;
        self.collapsed = paths.iter().cloned().collect();
    }

    pub fn expand_all(&mut self) {
        self.collapse_touched = true;
        self.collapsed.clear();
    }

    pub fn is_collapsed(&self, path: &str) -> bool {
        self.collapsed.contains(path)
    }

    pub fn focused(&self) -> Option<&str> {
        self.focused.as_deref()
    }

    pub fn set_focused(&mut self, path: &str) {
        self.focused = Some(path.to_owned());
    }

    /// Drops focus once its card is gone, so the next arrow key starts from the top instead of nowhere.
    pub fn prune_focus(&mut self, paths: &[String]) {
        if self
            .focused
            .as_ref()
            .is_some_and(|focused| !paths.contains(focused))
        {
            self.focused = None;
        }
    }

    pub fn move_focus(&mut self, paths: &[String], forward: bool) -> Option<&str> {
        let index = self
            .focused
            .as_ref()
            .and_then(|focused| paths.iter().position(|path| path == focused));
        let next = match index {
            Some(index) => {
                let target = if forward {
                    Some(index + 1)
                } else {
                    index.checked_sub(1)
                };
                target
                    .and_then(|target| paths.get(target))
                    .or(paths.get(index))
            }
            None if forward => paths.first(),
            None => paths.last(),
        };
        self.focused = next.cloned();
        self.focused()
    }
}
