use std::collections::HashSet;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq)]
struct ContentsFingerprint(u64);

impl ContentsFingerprint {
    fn from_contents(contents: &[u8]) -> Self {
        let mut hasher = DefaultHasher::new();
        contents.hash(&mut hasher);
        Self(hasher.finish())
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct RepositoryStore {
    repositories: Vec<String>,
}

/// Canonical file-backed pin store shared by every JayJay shell.
pub struct Store {
    state: RepositoryStore,
    save_path: Option<PathBuf>,
    loaded_fingerprint: Option<ContentsFingerprint>,
}

impl Store {
    pub fn load() -> Self {
        match Self::store_path() {
            Some(path) => Self::load_from(path),
            None => Self::in_memory(),
        }
    }

    pub fn load_from(path: PathBuf) -> Self {
        Self {
            state: RepositoryStore::default(),
            save_path: Some(path),
            loaded_fingerprint: None,
        }
    }

    pub fn in_memory() -> Self {
        Self {
            state: RepositoryStore::default(),
            save_path: None,
            loaded_fingerprint: None,
        }
    }

    pub fn repositories(&mut self) -> Vec<String> {
        self.refresh(false);
        self.state.repositories.clone()
    }

    pub fn set_pinned(&mut self, path: &Path, pinned: bool) -> Vec<String> {
        self.refresh(true);
        let Some(path) = stored_repository_path(path) else {
            return self.state.repositories.clone();
        };
        let mut next = self.state.clone();
        let was_pinned = next.repositories.contains(&path);
        if pinned && !was_pinned {
            next.repositories.insert(0, path);
        } else if !pinned && was_pinned {
            next.repositories.retain(|entry| entry != &path);
        } else {
            return self.state.repositories.clone();
        }
        if self.save(&next) {
            self.state = next;
        }
        self.state.repositories.clone()
    }

    pub fn store_path() -> Option<PathBuf> {
        if let Ok(path) = std::env::var("JAYJAY_REPOSITORIES_PATH")
            && !path.is_empty()
        {
            return Some(PathBuf::from(path));
        }
        ProjectDirs::from("dev", "hewig", "jayjay")
            .map(|dirs| dirs.config_dir().join("repositories.json"))
    }

    fn load_contents(path: &Path, contents: &[u8]) -> RepositoryStore {
        match serde_json::from_slice::<RepositoryStore>(contents) {
            Ok(mut state) => {
                let mut seen = HashSet::with_capacity(state.repositories.len());
                state
                    .repositories
                    .retain(|path| !path.is_empty() && seen.insert(path.clone()));
                state
            }
            Err(error) => {
                let corrupt = path.with_extension("json.corrupt");
                eprintln!(
                    "[repositories] {} failed to parse ({error}); preserving as {}",
                    path.display(),
                    corrupt.display()
                );
                let _ = fs::rename(path, corrupt);
                RepositoryStore::default()
            }
        }
    }

    fn refresh(&mut self, force: bool) {
        let Some(path) = self.save_path.as_ref() else {
            return;
        };
        let contents = match fs::read(path) {
            Ok(contents) => Some(contents),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => {
                eprintln!("[repositories] read {}: {error}", path.display());
                return;
            }
        };
        let fingerprint = contents.as_deref().map(ContentsFingerprint::from_contents);
        if !force && fingerprint == self.loaded_fingerprint {
            return;
        }
        self.state = contents
            .as_deref()
            .map(|contents| Self::load_contents(path, contents))
            .unwrap_or_default();
        self.loaded_fingerprint = fingerprint;
    }

    fn save(&mut self, state: &RepositoryStore) -> bool {
        let Some(path) = self.save_path.as_ref() else {
            return true;
        };
        match write_atomically(path, state) {
            Ok(fingerprint) => {
                self.loaded_fingerprint = Some(fingerprint);
                true
            }
            Err(error) => {
                eprintln!("[repositories] save {}: {error}", path.display());
                false
            }
        }
    }
}

pub fn normalize_repository_path(path: &Path) -> PathBuf {
    path.canonicalize()
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

fn write_atomically(path: &Path, state: &RepositoryStore) -> std::io::Result<ContentsFingerprint> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static WRITE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!(
        "json.tmp.{}.{}",
        std::process::id(),
        WRITE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let contents = serde_json::to_vec(state)?;
    let fingerprint = ContentsFingerprint::from_contents(&contents);
    if let Err(error) = fs::write(&temporary, contents) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    fs::rename(&temporary, path).inspect_err(|_| {
        let _ = fs::remove_file(&temporary);
    })?;
    Ok(fingerprint)
}

#[cfg(test)]
mod tests;
