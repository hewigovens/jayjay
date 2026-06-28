use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReviewEntry {
    /// Caller-supplied content identity at mark time. Treated as opaque.
    identity: String,
    #[serde(default)]
    file_marked: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    hunks: Vec<u32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct StoredReviews {
    #[serde(deserialize_with = "deserialize_reviewed")]
    reviewed: HashMap<String, ReviewEntry>,
}

// Drop unrecognized entry shapes (legacy mtime numbers, old hash-keyed entries) on load.
fn deserialize_reviewed<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<HashMap<String, ReviewEntry>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Loaded {
        Entry(ReviewEntry),
        #[allow(dead_code)]
        Unknown(serde_json::Value),
    }
    let raw: HashMap<String, Loaded> = HashMap::deserialize(d)?;
    Ok(raw
        .into_iter()
        .filter_map(|(k, v)| match v {
            Loaded::Entry(e) => Some((k, e)),
            Loaded::Unknown(_) => None,
        })
        .collect())
}

pub struct ReviewStore {
    state: StoredReviews,
    save_disabled: bool,
}

impl ReviewStore {
    pub fn load() -> Self {
        let state = Self::store_path().map(Self::load_path).unwrap_or_default();
        Self {
            state,
            save_disabled: false,
        }
    }

    /// Load the store at `path`; missing means empty. An unparseable file (e.g.
    /// truncated by an interrupted write) is renamed to `.json.corrupt` before
    /// defaulting, so the next save cannot destroy recoverable marks.
    fn load_path(path: PathBuf) -> StoredReviews {
        let Ok(text) = fs::read_to_string(&path) else {
            return StoredReviews::default();
        };
        match serde_json::from_str(&text) {
            Ok(state) => state,
            Err(e) => {
                let bad = path.with_extension("json.corrupt");
                eprintln!(
                    "[review_store] {} failed to parse ({e}); preserving as {}",
                    path.display(),
                    bad.display()
                );
                let _ = fs::rename(&path, &bad);
                StoredReviews::default()
            }
        }
    }

    /// Empty store that never touches disk. For tests and ephemeral contexts.
    pub fn in_memory() -> Self {
        Self {
            state: StoredReviews::default(),
            save_disabled: true,
        }
    }

    pub fn is_reviewed(&self, change_id: &str, path: &str, identity: &str) -> bool {
        let Some(entry) = self.state.reviewed.get(&key(change_id, path)) else {
            return false;
        };
        entry.file_marked && entry.identity == identity
    }

    // file_marked implies all hunks reviewed; otherwise check the explicit set.
    pub fn is_hunk_reviewed(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        hunk_idx: u32,
    ) -> bool {
        let Some(entry) = self.state.reviewed.get(&key(change_id, path)) else {
            return false;
        };
        if entry.identity != identity {
            return false;
        }
        entry.file_marked || entry.hunks.contains(&hunk_idx)
    }

    pub fn mark_reviewed(&mut self, change_id: &str, path: &str, identity: &str) {
        if identity.is_empty() {
            return;
        }
        self.state.reviewed.insert(
            key(change_id, path),
            ReviewEntry {
                identity: identity.to_string(),
                file_marked: true,
                hunks: vec![],
            },
        );
        self.save();
    }

    pub fn mark_unreviewed(&mut self, change_id: &str, path: &str) {
        self.state.reviewed.remove(&key(change_id, path));
        self.save();
    }

    pub fn toggle(&mut self, change_id: &str, path: &str, identity: &str) {
        if self.is_reviewed(change_id, path, identity) {
            self.mark_unreviewed(change_id, path);
        } else {
            self.mark_reviewed(change_id, path, identity);
        }
    }

    pub fn mark_hunk_reviewed(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        hunk_idx: u32,
    ) {
        if identity.is_empty() {
            return;
        }
        let k = key(change_id, path);
        match self.state.reviewed.get_mut(&k) {
            Some(entry) if entry.identity == identity => {
                if !entry.hunks.contains(&hunk_idx) {
                    entry.hunks.push(hunk_idx);
                    entry.hunks.sort_unstable();
                }
            }
            _ => {
                self.state.reviewed.insert(
                    k,
                    ReviewEntry {
                        identity: identity.to_string(),
                        file_marked: false,
                        hunks: vec![hunk_idx],
                    },
                );
            }
        }
        self.save();
    }

    pub fn mark_hunk_unreviewed(&mut self, change_id: &str, path: &str, hunk_idx: u32) {
        let k = key(change_id, path);
        let Some(entry) = self.state.reviewed.get_mut(&k) else {
            return;
        };
        entry.hunks.retain(|i| *i != hunk_idx);
        // Caller calls set_reviewed_hunks if they want the surviving hunks kept.
        entry.file_marked = false;
        if entry.hunks.is_empty() {
            self.state.reviewed.remove(&k);
        }
        self.save();
    }

    pub fn toggle_hunk(&mut self, change_id: &str, path: &str, identity: &str, hunk_idx: u32) {
        if self.is_hunk_reviewed(change_id, path, identity, hunk_idx) {
            self.mark_hunk_unreviewed(change_id, path, hunk_idx);
        } else {
            self.mark_hunk_reviewed(change_id, path, identity, hunk_idx);
        }
    }

    pub fn set_reviewed_hunks(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        hunk_indices: Vec<u32>,
    ) {
        if identity.is_empty() {
            return;
        }
        let k = key(change_id, path);
        if hunk_indices.is_empty() {
            self.state.reviewed.remove(&k);
        } else {
            let mut hunks = hunk_indices;
            hunks.sort_unstable();
            hunks.dedup();
            self.state.reviewed.insert(
                k,
                ReviewEntry {
                    identity: identity.to_string(),
                    file_marked: false,
                    hunks,
                },
            );
        }
        self.save();
    }

    /// Clear the committed working-copy marks without touching other changes or windows.
    pub fn clear_change(&mut self, change_id: &str) {
        let prefix = format!("{change_id}|");
        self.state.reviewed.retain(|k, _| !k.starts_with(&prefix));
        self.save();
    }

    /// Canonical on-disk path shared by every shell. Both the Rust store and
    /// the SwiftUI shell persist here so review marks transfer between shells.
    pub fn store_path() -> Option<PathBuf> {
        ProjectDirs::from("dev", "hewig", "jayjay")
            .map(|d| d.config_dir().join("review_store.json"))
    }

    fn save(&self) {
        if self.save_disabled {
            return;
        }
        let Some(path) = Self::store_path() else {
            return;
        };
        if let Err(e) = self.write_to(&path) {
            eprintln!("[review_store] save {}: {}", path.display(), e);
        }
    }

    /// Persist atomically: write a sibling temp file then rename over the
    /// target, so a concurrent reader never sees a half-written file.
    fn write_to(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string(&self.state)?;
        let tmp = path.with_extension("json.tmp");
        if let Err(e) = fs::write(&tmp, text) {
            let _ = fs::remove_file(&tmp);
            return Err(e);
        }
        fs::rename(&tmp, path).inspect_err(|_| {
            let _ = fs::remove_file(&tmp);
        })
    }
}

fn key(change_id: &str, path: &str) -> String {
    format!("{change_id}|{path}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> ReviewStore {
        ReviewStore::in_memory()
    }

    #[test]
    fn file_mark_roundtrip() {
        let mut s = make_store();
        s.mark_reviewed("c1", "a.txt", "id-v1");
        assert!(s.is_reviewed("c1", "a.txt", "id-v1"));
    }

    #[test]
    fn identity_change_invalidates_marks() {
        // Different identity for the same (change_id, path) — content changed.
        let mut s = make_store();
        s.mark_reviewed("c1", "a.txt", "id-v1");
        s.mark_hunk_reviewed("c1", "a.txt", "id-v1", 0);
        assert!(!s.is_reviewed("c1", "a.txt", "id-v2"));
        assert!(!s.is_hunk_reviewed("c1", "a.txt", "id-v2", 0));
    }

    #[test]
    fn matching_identity_keeps_marks() {
        // Store treats identity as opaque: a re-read with the same identity keeps
        // the mark. Rebase-invariance of the identity itself is proven against a
        // real repo in tests/review_identity.rs.
        let mut s = make_store();
        s.mark_reviewed("c1", "a.txt", "id-v1");
        assert!(s.is_reviewed("c1", "a.txt", "id-v1"));
    }

    #[test]
    fn empty_identity_is_a_no_op() {
        // Empty identity (e.g., file has no diff context) refuses to record.
        let mut s = make_store();
        s.mark_reviewed("c1", "a.txt", "");
        assert!(s.state.reviewed.is_empty());
    }

    #[test]
    fn hunk_mark_independent_of_file_flag() {
        let mut s = make_store();
        s.mark_hunk_reviewed("c1", "a.txt", "id", 2);
        assert!(s.is_hunk_reviewed("c1", "a.txt", "id", 2));
        assert!(!s.is_hunk_reviewed("c1", "a.txt", "id", 0));
        assert!(!s.is_reviewed("c1", "a.txt", "id"));
    }

    #[test]
    fn clear_change_only_drops_marks_for_that_change() {
        let mut s = make_store();
        s.mark_reviewed("c1", "a.txt", "id-v1");
        s.mark_reviewed("c2", "a.txt", "id-v1");
        s.clear_change("c1");
        assert!(!s.is_reviewed("c1", "a.txt", "id-v1"));
        assert!(s.is_reviewed("c2", "a.txt", "id-v1"));
    }

    #[test]
    fn file_marked_rollup_and_demotion() {
        let mut s = make_store();
        s.mark_reviewed("c1", "a.txt", "id");
        assert!(s.is_hunk_reviewed("c1", "a.txt", "id", 999));
        s.mark_hunk_unreviewed("c1", "a.txt", 1);
        assert!(!s.is_reviewed("c1", "a.txt", "id"));
    }

    #[test]
    fn json_load_drops_legacy_and_save_round_trips() {
        let json = r#"{"reviewed":{"c|legacy":12.34,"c|new":{"identity":"id1","file_marked":true,"hunks":[1,3]}}}"#;
        let parsed: StoredReviews = serde_json::from_str(json).unwrap();
        assert!(!parsed.reviewed.contains_key("c|legacy"));
        let e = &parsed.reviewed["c|new"];
        assert_eq!(e.identity, "id1");
        assert!(e.file_marked);
        assert_eq!(e.hunks, vec![1, 3]);
        let text = serde_json::to_string(&parsed).unwrap();
        assert_eq!(
            text,
            r#"{"reviewed":{"c|new":{"identity":"id1","file_marked":true,"hunks":[1,3]}}}"#
        );
    }

    #[test]
    fn write_to_persists_atomically_and_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("review_store.json");

        let mut s = make_store();
        s.mark_reviewed("c1", "a.txt", "id-v1");
        s.write_to(&path).unwrap();

        // No stray temp file survives a successful write.
        assert!(!path.with_extension("json.tmp").exists());

        // Reload through the same parser the app uses on startup.
        let text = std::fs::read_to_string(&path).unwrap();
        let state: StoredReviews = serde_json::from_str(&text).unwrap();
        let reloaded = ReviewStore {
            state,
            save_disabled: true,
        };
        assert!(reloaded.is_reviewed("c1", "a.txt", "id-v1"));
    }

    #[test]
    fn write_to_replaces_existing_file_without_clobbering_other_keys() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("review_store.json");

        let mut first = make_store();
        first.mark_reviewed("c1", "a.txt", "id-v1");
        first.write_to(&path).unwrap();

        // A second snapshot that also kept the first mark replaces the file.
        let mut second = make_store();
        second.mark_reviewed("c1", "a.txt", "id-v1");
        second.mark_reviewed("c1", "b.txt", "id-v1");
        second.write_to(&path).unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        let state: StoredReviews = serde_json::from_str(&text).unwrap();
        assert_eq!(state.reviewed.len(), 2);
    }

    #[test]
    fn corrupt_file_is_preserved_not_silently_wiped_on_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("review_store.json");
        // Truncated JSON, the shape an interrupted write would leave behind.
        std::fs::write(&path, r#"{"reviewed":{"c|a.txt":{"identi"#).unwrap();

        let state = ReviewStore::load_path(path.clone());

        // Load defaults to empty so the app stays usable...
        assert!(state.reviewed.is_empty());
        // ...but the bad file is moved aside, not left for the next save to clobber.
        assert!(!path.exists());
        assert!(path.with_extension("json.corrupt").exists());
    }

    #[test]
    fn missing_file_loads_empty_without_creating_corrupt_sibling() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("review_store.json");

        let state = ReviewStore::load_path(path.clone());

        assert!(state.reviewed.is_empty());
        assert!(!path.with_extension("json.corrupt").exists());
    }
}
