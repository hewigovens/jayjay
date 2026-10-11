use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::recent::{RECENT_REVSET_LIMIT, remember_revset};
use crate::json_store::JsonStore;
use crate::repositories::normalize_repository_path;

#[derive(Clone, Default, Deserialize, Serialize)]
struct RecentRevsets {
    repositories: BTreeMap<String, Vec<String>>,
}

impl RecentRevsets {
    fn normalize(&mut self) {
        self.repositories.retain(|path, recent| {
            let mut seen = HashSet::with_capacity(recent.len());
            recent.retain(|revset| !revset.is_empty() && seen.insert(revset.clone()));
            recent.truncate(RECENT_REVSET_LIMIT);
            !path.is_empty() && !recent.is_empty()
        });
    }
}

const LABEL: &str = "recent-revsets";

/// Typed revsets remembered per repository, newest first, and shared by every JayJay shell.
pub struct RecentRevsetStore(JsonStore<RecentRevsets>);

impl RecentRevsetStore {
    pub fn load() -> Self {
        Self(JsonStore::load(
            LABEL,
            "JAYJAY_RECENT_REVSETS_PATH",
            "recent_revsets.json",
            RecentRevsets::normalize,
        ))
    }

    pub fn load_from(path: PathBuf) -> Self {
        Self(JsonStore::new(LABEL, Some(path), RecentRevsets::normalize))
    }

    pub fn in_memory() -> Self {
        Self(JsonStore::new(LABEL, None, RecentRevsets::normalize))
    }

    pub fn recent(&mut self, repository: &Path) -> Vec<String> {
        let key = repository_key(repository);
        self.0
            .read()
            .repositories
            .get(&key)
            .cloned()
            .unwrap_or_default()
    }

    /// Records a typed revset; presets and the default filter are not remembered.
    pub fn remember(&mut self, repository: &Path, revset: &str) -> Vec<String> {
        self.edit(repository, |recent| remember_revset(recent, revset))
    }

    pub fn forget(&mut self, repository: &Path, revset: &str) -> Vec<String> {
        self.edit(repository, |recent| {
            recent
                .iter()
                .filter(|entry| *entry != revset)
                .cloned()
                .collect()
        })
    }

    fn edit(
        &mut self,
        repository: &Path,
        edit: impl FnOnce(&[String]) -> Vec<String>,
    ) -> Vec<String> {
        let key = repository_key(repository);
        self.0
            .update(|state| {
                let current = state.repositories.get(&key).map_or(&[][..], Vec::as_slice);
                let next = edit(current);
                if next == current {
                    return false;
                }
                if next.is_empty() {
                    state.repositories.remove(&key);
                } else {
                    state.repositories.insert(key.clone(), next);
                }
                true
            })
            .repositories
            .get(&key)
            .cloned()
            .unwrap_or_default()
    }
}

/// Recent revsets are history, not identity, so a lossy key for a non-UTF-8 path is acceptable.
fn repository_key(repository: &Path) -> String {
    normalize_repository_path(repository)
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_revsets_persist_per_repository() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("recent_revsets.json");
        let first = directory.path().join("first");
        let second = directory.path().join("second");
        std::fs::create_dir_all(&first).unwrap();
        std::fs::create_dir_all(&second).unwrap();

        let mut store = RecentRevsetStore::load_from(path.clone());
        store.remember(&first, "a()");
        store.remember(&first, "mine()");
        store.remember(&first, "b()");
        store.remember(&second, "c()");

        let mut reloaded = RecentRevsetStore::load_from(path.clone());
        assert_eq!(reloaded.recent(&first), ["b()", "a()"]);
        assert_eq!(reloaded.recent(&first.join(".")), ["b()", "a()"]);
        assert_eq!(reloaded.recent(&second), ["c()"]);

        assert_eq!(reloaded.forget(&first, "b()"), ["a()"]);
        assert!(reloaded.forget(&second, "c()").is_empty());
        let mut reloaded = RecentRevsetStore::load_from(path);
        assert_eq!(reloaded.recent(&first), ["a()"]);
        assert!(reloaded.recent(&second).is_empty());
    }

    #[test]
    fn loading_drops_duplicates_empty_entries_and_overflow() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("recent_revsets.json");
        let overflow: Vec<String> = (0..RECENT_REVSET_LIMIT + 3)
            .map(|n| format!("r{n}()"))
            .collect();
        let contents = serde_json::json!({
            "repositories": {
                "/repo": ["a()", "", "a()", "b()"],
                "/long": overflow,
                "/empty": [""],
            }
        });
        std::fs::write(&path, contents.to_string()).unwrap();

        let mut store = RecentRevsetStore::load_from(path);
        let state = store.0.read();
        assert_eq!(state.repositories["/repo"], ["a()", "b()"]);
        assert_eq!(state.repositories["/long"], overflow[..RECENT_REVSET_LIMIT]);
        assert!(!state.repositories.contains_key("/empty"));
    }
}
