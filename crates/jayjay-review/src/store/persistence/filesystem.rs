use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use directories::ProjectDirs;

use super::super::models::StoredReviews;
use super::super::persist::ReviewStore;

/// (mtime, len) of the store file when its contents were loaded; a differing stamp means another process wrote since.
#[derive(Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    mtime: SystemTime,
    len: u64,
}

pub(in crate::store) struct Persistence {
    save_path: Option<PathBuf>,
    loaded_stamp: Option<FileStamp>,
}

impl Persistence {
    pub(in crate::store) fn in_memory() -> Self {
        Self {
            save_path: None,
            loaded_stamp: None,
        }
    }

    fn at_path(save_path: PathBuf, loaded_stamp: Option<FileStamp>) -> Self {
        Self {
            save_path: Some(save_path),
            loaded_stamp,
        }
    }
}

impl ReviewStore {
    pub fn load() -> Self {
        match Self::store_path() {
            Some(path) => Self::load_from(path),
            None => Self::from_state(StoredReviews::default()),
        }
    }

    pub fn load_from(path: PathBuf) -> Self {
        // Stamp before reading: if a writer lands in between, the stamp mismatch forces a safe reload on the next staleness check.
        let stamp = Self::stamp(&path);
        let state = Self::load_path(path.clone());
        let mut store = Self::from_state(state);
        store.persistence = Persistence::at_path(path, stamp);
        store
    }

    fn stamp(path: &Path) -> Option<FileStamp> {
        let meta = fs::metadata(path).ok()?;
        Some(FileStamp {
            mtime: meta.modified().ok()?,
            len: meta.len(),
        })
    }

    /// An unparseable file is renamed to `.json.corrupt` before defaulting to empty, so the next save cannot destroy recoverable marks.
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

    /// Long-lived stores must call this before mutating, otherwise save rewrites state another process persisted since loading.
    pub fn refresh_from_disk(&mut self) {
        let Some(path) = self.persistence.save_path.clone() else {
            return;
        };
        // A missing file means the store was deleted externally; keep the in-memory state so the next save restores it.
        if !path.exists() {
            return;
        }
        self.persistence.loaded_stamp = Self::stamp(&path);
        self.state = Self::load_path(path);
    }

    /// Cheap staleness check for read/render paths; mutations still refresh unconditionally.
    pub fn refresh_if_stale(&mut self) {
        let Some(path) = self.persistence.save_path.clone() else {
            return;
        };
        let current = Self::stamp(&path);
        if current.is_none() || current == self.persistence.loaded_stamp {
            return;
        }
        self.persistence.loaded_stamp = current;
        self.state = Self::load_path(path);
    }

    /// Canonical on-disk path shared by every shell.
    pub fn store_path() -> Option<PathBuf> {
        if let Ok(path) = std::env::var("JAYJAY_REVIEW_STORE_PATH")
            && !path.is_empty()
        {
            return Some(PathBuf::from(path));
        }
        ProjectDirs::from("dev", "hewig", "jayjay")
            .map(|dirs| dirs.config_dir().join("review_store.json"))
    }

    pub(crate) fn save(&mut self) {
        let Some(path) = self.persistence.save_path.clone() else {
            return;
        };
        if let Err(e) = self.write_to(&path) {
            eprintln!("[review_store] save {}: {}", path.display(), e);
        }
        self.persistence.loaded_stamp = Self::stamp(&path);
    }

    /// Persist atomically with a unique sibling temp file so concurrent writers cannot observe or rename a half-written store.
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
