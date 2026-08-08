use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use directories::ProjectDirs;

use super::models::StoredReviews;

pub trait IdSource {
    fn next_id(&mut self) -> String;
}

pub struct UuidIdSource;

impl IdSource for UuidIdSource {
    fn next_id(&mut self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

/// (mtime, len) of the store file at the moment its contents were loaded; a differing stamp means another process wrote since.
#[derive(Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    mtime: SystemTime,
    len: u64,
}

pub struct ReviewStore {
    pub(crate) state: StoredReviews,
    save_disabled: bool,
    save_path: Option<PathBuf>,
    loaded_stamp: Option<FileStamp>,
    pub(crate) id_source: Box<dyn IdSource>,
}

impl ReviewStore {
    pub fn load() -> Self {
        match Self::store_path() {
            Some(path) => Self::load_from(path),
            None => Self::from_state_with_path(StoredReviews::default(), false, None),
        }
    }

    pub fn load_from(path: PathBuf) -> Self {
        // Stamp before reading: if a writer lands in between, the stamp mismatch forces a (redundant but safe) reload on the next staleness check.
        let stamp = Self::stamp(&path);
        let state = Self::load_path(path.clone());
        let mut store = Self::from_state_with_path(state, false, Some(path));
        store.loaded_stamp = stamp;
        store
    }

    pub fn in_memory() -> Self {
        Self::from_state(StoredReviews::default(), true)
    }

    #[cfg(any(test, feature = "test-util"))]
    pub fn in_memory_with_ids(id_source: Box<dyn IdSource>) -> Self {
        Self {
            state: StoredReviews::default(),
            save_disabled: true,
            save_path: None,
            loaded_stamp: None,
            id_source,
        }
    }

    pub(crate) fn from_state(state: StoredReviews, save_disabled: bool) -> Self {
        Self::from_state_with_path(state, save_disabled, None)
    }

    fn from_state_with_path(
        state: StoredReviews,
        save_disabled: bool,
        save_path: Option<PathBuf>,
    ) -> Self {
        Self {
            state,
            save_disabled,
            save_path,
            loaded_stamp: None,
            id_source: Box::new(UuidIdSource),
        }
    }

    fn stamp(path: &Path) -> Option<FileStamp> {
        let meta = fs::metadata(path).ok()?;
        Some(FileStamp {
            mtime: meta.modified().ok()?,
            len: meta.len(),
        })
    }

    /// An unparseable file (e.g. truncated by an interrupted write) is renamed to `.json.corrupt` before defaulting to empty, so the next save cannot destroy recoverable marks.
    pub(crate) fn load_path(path: PathBuf) -> StoredReviews {
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

    /// Long-lived stores must call this before mutating, otherwise their save() rewrites marks and notes another process persisted since they loaded.
    pub fn refresh_from_disk(&mut self) {
        if self.save_disabled {
            return;
        }
        let Some(path) = self.save_path.clone().or_else(Self::store_path) else {
            return;
        };
        // A missing file means the store was deleted externally; keep the in-memory state so the next save restores it instead of persisting only the newest mutation.
        if !path.exists() {
            return;
        }
        self.loaded_stamp = Self::stamp(&path);
        self.state = Self::load_path(path);
    }

    /// Cheap staleness check for read/render paths: reload only when the file's stamp says another process wrote since we loaded. Mutations still refresh unconditionally via `refresh_from_disk`.
    pub fn refresh_if_stale(&mut self) {
        if self.save_disabled {
            return;
        }
        let Some(path) = self.save_path.clone().or_else(Self::store_path) else {
            return;
        };
        let current = Self::stamp(&path);
        if current.is_none() || current == self.loaded_stamp {
            return;
        }
        self.loaded_stamp = current;
        self.state = Self::load_path(path);
    }

    /// Canonical on-disk path shared by every shell; the Rust store and the SwiftUI shell both persist here so review marks transfer between shells.
    pub fn store_path() -> Option<PathBuf> {
        if let Ok(path) = std::env::var("JAYJAY_REVIEW_STORE_PATH")
            && !path.is_empty()
        {
            return Some(PathBuf::from(path));
        }
        ProjectDirs::from("dev", "hewig", "jayjay")
            .map(|d| d.config_dir().join("review_store.json"))
    }

    pub(crate) fn save(&mut self) {
        if self.save_disabled {
            return;
        }
        let Some(path) = self.save_path.clone().or_else(Self::store_path) else {
            return;
        };
        if let Err(e) = self.write_to(&path) {
            eprintln!("[review_store] save {}: {}", path.display(), e);
        }
        self.loaded_stamp = Self::stamp(&path);
    }

    /// Persist atomically (sibling temp file, then rename over the target) so a concurrent reader never sees a half-written file; the temp name is unique per process and write because the store is shared across the app, GPUI, and the CLI, and a fixed name would let concurrent writers rename or delete each other's half-written temp files.
    pub(crate) fn write_to(&self, path: &Path) -> std::io::Result<()> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static WRITE_SEQ: AtomicU64 = AtomicU64::new(0);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string(&self.state)?;
        let tmp = path.with_extension(format!(
            "json.tmp.{}.{}",
            std::process::id(),
            WRITE_SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        if let Err(e) = fs::write(&tmp, text) {
            let _ = fs::remove_file(&tmp);
            return Err(e);
        }
        fs::rename(&tmp, path).inspect_err(|_| {
            let _ = fs::remove_file(&tmp);
        })
    }
}
