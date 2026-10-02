use std::path::{Path, PathBuf};

use gpui::{App, BorrowAppContext, Global};
use jayjay_core::{AppDirs, write_atomically};

use super::RatingPromptState;

pub struct RatingPromptStore {
    path: Option<PathBuf>,
    state: RatingPromptState,
}

impl Global for RatingPromptStore {}

impl RatingPromptStore {
    pub fn load() -> Self {
        let path = AppDirs::new().map(|dirs| dirs.data.join("rating_prompt.json"));
        let state = path.as_deref().map(read_state).unwrap_or_default();
        Self { path, state }
    }

    pub fn in_memory(state: RatingPromptState) -> Self {
        Self { path: None, state }
    }

    pub fn state(&self) -> RatingPromptState {
        self.state
    }

    pub(crate) fn record_action(cx: &mut App, can_show: bool) -> bool {
        cx.has_global::<Self>()
            && cx.update_global::<Self, _>(|store, _| {
                store.update(|state| state.record_action(can_show))
            })
    }

    pub(crate) fn turn_off(cx: &mut App) {
        if cx.has_global::<Self>() {
            cx.update_global::<Self, _>(|store, _| store.update(|state| state.dismissed = true));
        }
    }

    /// Rereads the file first so counts from other processes survive.
    fn update<R>(&mut self, change: impl FnOnce(&mut RatingPromptState) -> R) -> R {
        if let Some(path) = &self.path {
            self.state = read_state(path);
        }
        let before = self.state;
        let result = change(&mut self.state);
        if let Some(path) = &self.path
            && self.state != before
            && let Err(error) = serde_json::to_vec(&self.state)
                .map_err(std::io::Error::other)
                .and_then(|contents| write_atomically(path, &contents))
        {
            eprintln!("[jayjay-gpui] failed to save the rating prompt state: {error}");
        }
        result
    }
}

fn read_state(path: &Path) -> RatingPromptState {
    std::fs::read(path)
        .ok()
        .and_then(|contents| serde_json::from_slice(&contents).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_sharing_a_file_keep_each_others_counts_and_dismissal() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("rating_prompt.json");
        let open = || RatingPromptStore {
            path: Some(path.clone()),
            state: read_state(&path),
        };
        let (mut first, mut second) = (open(), open());

        for _ in 0..2 {
            first.update(|state| state.record_action(true));
            second.update(|state| state.record_action(true));
        }
        assert_eq!(read_state(&path).action_count, 4);

        first.update(|state| state.dismissed = true);
        assert!(!second.update(|state| state.record_action(true)));
        assert_eq!(read_state(&path).action_count, 4);
    }
}
