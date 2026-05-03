use std::sync::Arc;

use gpui::{App, BorrowAppContext, Global};

use super::AppConfig;
use crate::app::theme::Theme;

/// Wrapper makes `AppConfig` registrable as a GPUI global. Fields are
/// accessed via `cx.global::<AppConfigStore>().config` and mutated via the
/// `update` helper below (which also persists to disk).
pub struct AppConfigStore {
    pub config: Arc<AppConfig>,
}

impl Global for AppConfigStore {}

impl AppConfigStore {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

/// Read the current config from a `cx`. Pair with
/// `cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())` in any view
/// that needs to re-render on changes.
pub fn current(cx: &App) -> Arc<AppConfig> {
    cx.global::<AppConfigStore>().config.clone()
}

/// Apply a mutation to the config; persist to disk and notify global
/// observers.
pub fn update<F>(cx: &mut App, mutate: F)
where
    F: FnOnce(&mut AppConfig),
{
    cx.update_global::<AppConfigStore, _>(|store, _| {
        let mut next = (*store.config).clone();
        mutate(&mut next);
        let _ = next.save();
        store.config = Arc::new(next);
    });
    let appearance = cx.global::<AppConfigStore>().config.appearance;
    cx.set_global(Theme::for_appearance(appearance));
    cx.refresh_windows();
}
