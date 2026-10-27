use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::json_store::JsonStore;
use crate::ssh::RemoteLocation;

mod grouping;
pub use grouping::{RepoGroup, RepoListGroups, group_repositories};

#[derive(Clone, Default, Deserialize, Serialize)]
struct RepositoryStore {
    repositories: Vec<String>,
}

impl RepositoryStore {
    fn normalize(&mut self) {
        let mut seen = HashSet::with_capacity(self.repositories.len());
        self.repositories
            .retain(|path| !path.is_empty() && seen.insert(path.clone()));
    }
}

const LABEL: &str = "repositories";

/// Canonical file-backed pin store shared by every JayJay shell.
pub struct Store(JsonStore<RepositoryStore>);

impl Store {
    pub fn load() -> Self {
        Self(JsonStore::load(
            LABEL,
            "JAYJAY_REPOSITORIES_PATH",
            "repositories.json",
            RepositoryStore::normalize,
        ))
    }

    pub fn load_from(path: PathBuf) -> Self {
        Self(JsonStore::new(
            LABEL,
            Some(path),
            RepositoryStore::normalize,
        ))
    }

    pub fn in_memory() -> Self {
        Self(JsonStore::new(LABEL, None, RepositoryStore::normalize))
    }

    pub fn repositories(&mut self) -> Vec<String> {
        self.0.read().repositories.clone()
    }

    pub fn set_pinned(&mut self, path: &Path, pinned: bool) -> Vec<String> {
        let Some(path) = stored_repository_path(path) else {
            return self.repositories();
        };
        self.0
            .update(|next| {
                // Stored entries may predate normalization, so match by canonical form too.
                let is_target = |entry: &String| {
                    entry == &path
                        || stored_repository_path(Path::new(entry)).as_deref()
                            == Some(path.as_str())
                };
                if pinned && next.repositories.contains(&path)
                    || !pinned && !next.repositories.iter().any(is_target)
                {
                    return false;
                }
                next.repositories.retain(|entry| !is_target(entry));
                if pinned {
                    next.repositories.insert(0, path);
                }
                true
            })
            .repositories
            .clone()
    }
}

pub fn normalize_repository_path(path: &Path) -> PathBuf {
    if let Some(location) = RemoteLocation::from_path(path) {
        return PathBuf::from(location.encode());
    }
    dunce::canonicalize(path)
        .or_else(|_| std::path::absolute(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

fn stored_repository_path(path: &Path) -> Option<String> {
    let normalized = normalize_repository_path(path);
    match normalized.into_os_string().into_string() {
        Ok(path) => Some(path),
        Err(path) => {
            eprintln!(
                "[repositories] cannot pin non-UTF-8 path {}",
                PathBuf::from(path).display()
            );
            None
        }
    }
}

#[cfg(test)]
mod tests;
