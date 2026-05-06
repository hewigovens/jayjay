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
        let state = match Self::store_path().and_then(|p| fs::read_to_string(p).ok()) {
            Some(text) => serde_json::from_str(&text).unwrap_or_default(),
            None => StoredReviews::default(),
        };
        Self {
            state,
            save_disabled: false,
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

    pub fn clear_all(&mut self) {
        self.state.reviewed.clear();
        self.save();
    }

    fn store_path() -> Option<PathBuf> {
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
        if let Some(parent) = path.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            eprintln!("[review_store] mkdir {}: {}", parent.display(), e);
            return;
        }
        let text = match serde_json::to_string(&self.state) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[review_store] serialize: {e}");
                return;
            }
        };
        if let Err(e) = fs::write(&path, text) {
            eprintln!("[review_store] write {}: {}", path.display(), e);
        }
    }
}

fn key(change_id: &str, path: &str) -> String {
    format!("{change_id}|{path}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> ReviewStore {
        ReviewStore {
            state: StoredReviews::default(),
            save_disabled: true,
        }
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
    fn rebase_preserving_identity_keeps_marks() {
        // Same identity (caller derives it from content) — review survives.
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
        assert!(parsed.reviewed.get("c|legacy").is_none());
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
}
