use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use jayjay_primitives::AppDirs;

use super::super::models::StoredReviews;
use super::super::persist::ReviewStore;

/// (mtime, len) of the store file when its contents were loaded; a differing stamp means another process wrote since.
#[derive(Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    mtime: SystemTime,
    len: u64,
}

enum StoredFile {
    Missing,
    Parsed(StoredReviews),
    Corrupt(serde_json::Error),
}

pub(crate) struct Persistence {
    save_path: Option<PathBuf>,
    loaded_stamp: Option<FileStamp>,
}

impl Persistence {
    pub(crate) fn in_memory() -> Self {
        Self {
            save_path: None,
            loaded_stamp: None,
        }
    }

    pub(crate) fn is_persistent(&self) -> bool {
        self.save_path.is_some()
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
        match Self::try_load_from(path.clone()) {
            Ok(store) => store,
            Err(error) => {
                eprintln!("[review_store] load: {error}");
                let mut store = Self::in_memory();
                store.persistence = Persistence::at_path(path, None);
                store
            }
        }
    }

    pub fn try_load() -> io::Result<Self> {
        let path = Self::store_path()
            .ok_or_else(|| io::Error::other("Review store directory is unavailable"))?;
        Self::try_load_from(path)
    }

    pub fn try_load_from(path: PathBuf) -> io::Result<Self> {
        let mut store = Self::in_memory();
        store.persistence = Persistence::at_path(path.clone(), None);
        store.reload()?;
        Ok(store)
    }

    // Writers replace the JSON atomically, so reads never need the lock; only quarantining a corrupt file does.
    fn reload(&mut self) -> io::Result<()> {
        let Some(path) = self.persistence.save_path.clone() else {
            return Ok(());
        };
        let stamp = Self::stamp(&path);
        match Self::read_path(&path)? {
            StoredFile::Missing => {}
            StoredFile::Parsed(state) => {
                self.state = state;
                self.persistence.loaded_stamp = stamp;
            }
            StoredFile::Corrupt(_) => {
                self.begin_mutation()?;
            }
        }
        Ok(())
    }

    pub(crate) fn begin_mutation(&mut self) -> io::Result<Option<File>> {
        let Some(path) = self.persistence.save_path.clone() else {
            return Ok(None);
        };
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        // A separate inode stays locked across atomic replacement of the JSON file.
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.with_extension("json.lock"))?;
        lock.lock()?;
        match Self::read_path(&path)? {
            StoredFile::Missing => {}
            StoredFile::Parsed(state) => self.state = state,
            StoredFile::Corrupt(error) => {
                let bad = path.with_extension("json.corrupt");
                eprintln!(
                    "[review_store] {} failed to parse ({error}); preserving as {}",
                    path.display(),
                    bad.display()
                );
                fs::rename(&path, bad)?;
                self.state = StoredReviews::default();
            }
        }
        self.persistence.loaded_stamp = Self::stamp(&path);
        Ok(Some(lock))
    }

    fn stamp(path: &Path) -> Option<FileStamp> {
        let meta = fs::metadata(path).ok()?;
        Some(FileStamp {
            mtime: meta.modified().ok()?,
            len: meta.len(),
        })
    }

    fn read_path(path: &Path) -> io::Result<StoredFile> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(StoredFile::Missing);
            }
            Err(error) => return Err(error),
        };
        Ok(match serde_json::from_str::<StoredReviews>(&text) {
            Ok(mut state) => {
                state.extra.remove("review_baselines");
                StoredFile::Parsed(state)
            }
            Err(error) => StoredFile::Corrupt(error),
        })
    }

    pub fn refresh_from_disk(&mut self) {
        if let Err(error) = self.reload() {
            eprintln!("[review_store] refresh: {error}");
        }
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
        self.refresh_from_disk();
    }

    /// Canonical on-disk path shared by every shell.
    pub fn store_path() -> Option<PathBuf> {
        if let Ok(path) = std::env::var("JAYJAY_REVIEW_STORE_PATH")
            && !path.is_empty()
        {
            return Some(PathBuf::from(path));
        }
        AppDirs::new().map(|dirs| dirs.config.join("review_store.json"))
    }

    pub(crate) fn save_checked(&mut self) -> std::io::Result<()> {
        let Some(path) = self.persistence.save_path.clone() else {
            return Ok(());
        };
        self.write_to(&path)?;
        self.persistence.loaded_stamp = Self::stamp(&path);
        Ok(())
    }

    /// Persist atomically with a unique sibling temp file so concurrent writers cannot observe or rename a half-written store.
    pub(crate) fn write_to(&self, path: &Path) -> std::io::Result<()> {
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
