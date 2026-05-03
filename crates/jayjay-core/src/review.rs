use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
struct StoredReviews {
    /// Key: "change_id|path"  →  Value: file mtime (seconds since epoch) when reviewed.
    reviewed: HashMap<String, f64>,
}

pub struct ReviewStore {
    repo_path: PathBuf,
    state: StoredReviews,
}

impl ReviewStore {
    pub fn load(repo_path: PathBuf) -> Self {
        let state = match Self::store_path().and_then(|p| fs::read_to_string(p).ok()) {
            Some(text) => serde_json::from_str(&text).unwrap_or_default(),
            None => StoredReviews::default(),
        };
        Self { repo_path, state }
    }

    pub fn set_repo_path(&mut self, path: PathBuf) {
        self.repo_path = path;
    }

    /// Returns true only if the file was reviewed AND its mtime is unchanged
    /// since. If the file has been edited since, it counts as unreviewed —
    /// the user needs to look at it again.
    ///
    /// `current == 0.0` is treated as "couldn't read mtime" (file deleted or
    /// permission error) and downgrades to unreviewed so users aren't
    /// misled into thinking a missing file is still verified.
    pub fn is_reviewed(&self, change_id: &str, path: &str) -> bool {
        let key = key(change_id, path);
        let Some(reviewed_mtime) = self.state.reviewed.get(&key) else {
            return false;
        };
        let current = self.file_mtime(path);
        if current == 0.0 {
            return false;
        }
        current <= *reviewed_mtime
    }

    pub fn mark_reviewed(&mut self, change_id: &str, path: &str) {
        self.state
            .reviewed
            .insert(key(change_id, path), self.file_mtime(path));
        self.save();
    }

    pub fn mark_unreviewed(&mut self, change_id: &str, path: &str) {
        self.state.reviewed.remove(&key(change_id, path));
        self.save();
    }

    pub fn toggle(&mut self, change_id: &str, path: &str) {
        if self.is_reviewed(change_id, path) {
            self.mark_unreviewed(change_id, path);
        } else {
            self.mark_reviewed(change_id, path);
        }
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

    fn file_mtime(&self, relative: &str) -> f64 {
        let full = self.repo_path.join(relative);
        mtime_seconds(&full)
    }
}

fn key(change_id: &str, path: &str) -> String {
    format!("{change_id}|{path}")
}

fn mtime_seconds(path: &Path) -> f64 {
    let Ok(meta) = fs::metadata(path) else {
        return 0.0;
    };
    let Ok(modified) = meta.modified() else {
        return 0.0;
    };
    modified
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}
