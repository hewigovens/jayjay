use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use jayjay_primitives::AppDirs;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[derive(Clone, Copy, PartialEq, Eq)]
struct ContentsFingerprint(u64);

impl ContentsFingerprint {
    fn from_contents(contents: &[u8]) -> Self {
        let mut hasher = DefaultHasher::new();
        contents.hash(&mut hasher);
        Self(hasher.finish())
    }
}

/// A shared JSON document that reparses only when its contents changed and rereads before every atomic update, so separate JayJay processes keep each other's changes.
pub(crate) struct JsonStore<T> {
    label: &'static str,
    state: T,
    save_path: Option<PathBuf>,
    loaded_fingerprint: Option<ContentsFingerprint>,
    normalize: fn(&mut T),
}

impl<T: Clone + Default + Serialize + DeserializeOwned> JsonStore<T> {
    /// `env_override` names the variable tests use to redirect `file_name` away from the user's config.
    pub(crate) fn load(
        label: &'static str,
        env_override: &str,
        file_name: &str,
        normalize: fn(&mut T),
    ) -> Self {
        let path = std::env::var(env_override)
            .ok()
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
            .or_else(|| AppDirs::new().map(|dirs| dirs.config.join(file_name)));
        Self::new(label, path, normalize)
    }

    pub(crate) fn new(label: &'static str, path: Option<PathBuf>, normalize: fn(&mut T)) -> Self {
        Self {
            label,
            state: T::default(),
            save_path: path,
            loaded_fingerprint: None,
            normalize,
        }
    }

    pub(crate) fn read(&mut self) -> &T {
        self.refresh(false);
        &self.state
    }

    /// Saves only when `mutate` reports a change; a failed save keeps the previous state.
    pub(crate) fn update(&mut self, mutate: impl FnOnce(&mut T) -> bool) -> &T {
        self.refresh(true);
        let mut next = self.state.clone();
        if mutate(&mut next) && self.save(&next) {
            self.state = next;
        }
        &self.state
    }

    fn parse(&self, path: &Path, contents: &[u8]) -> T {
        match serde_json::from_slice::<T>(contents) {
            Ok(mut state) => {
                (self.normalize)(&mut state);
                state
            }
            Err(error) => {
                let corrupt = path.with_extension("json.corrupt");
                eprintln!(
                    "[{}] {} failed to parse ({error}); preserving as {}",
                    self.label,
                    path.display(),
                    corrupt.display()
                );
                let _ = fs::rename(path, corrupt);
                T::default()
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
                eprintln!("[{}] read {}: {error}", self.label, path.display());
                return;
            }
        };
        let fingerprint = contents.as_deref().map(ContentsFingerprint::from_contents);
        if !force && fingerprint == self.loaded_fingerprint {
            return;
        }
        self.state = contents
            .as_deref()
            .map(|contents| self.parse(path, contents))
            .unwrap_or_default();
        self.loaded_fingerprint = fingerprint;
    }

    fn save(&mut self, state: &T) -> bool {
        let Some(path) = self.save_path.as_ref() else {
            return true;
        };
        let written = serde_json::to_vec(state)
            .map_err(std::io::Error::from)
            .and_then(|contents| {
                crate::write_atomically(path, &contents)?;
                Ok(ContentsFingerprint::from_contents(&contents))
            });
        match written {
            Ok(fingerprint) => {
                self.loaded_fingerprint = Some(fingerprint);
                true
            }
            Err(error) => {
                eprintln!("[{}] save {}: {error}", self.label, path.display());
                false
            }
        }
    }
}
