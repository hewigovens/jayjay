use std::sync::Arc;

use gpui::{App, BorrowAppContext, Global};

use super::AppConfig;
use crate::app::fonts;
use crate::app::theme;

pub struct AppConfigStore {
    config: Arc<AppConfig>,
    persist: bool,
}

impl Global for AppConfigStore {}

impl AppConfigStore {
    pub fn new(config: AppConfig) -> Self {
        fonts::sync_from_config(&config);
        Self {
            config: Arc::new(config),
            persist: true,
        }
    }

    /// Test-only config state that never writes the user's real config file.
    pub fn new_ephemeral(config: AppConfig) -> Self {
        fonts::sync_from_config(&config);
        Self {
            config: Arc::new(config),
            persist: false,
        }
    }
}

pub fn current(cx: &App) -> Arc<AppConfig> {
    cx.global::<AppConfigStore>().config.clone()
}

pub fn update<F>(cx: &mut App, mutate: F)
where
    F: FnOnce(&mut AppConfig),
{
    let mod_key_changed = cx.update_global::<AppConfigStore, _>(|store, _| {
        let mut next = (*store.config).clone();
        mutate(&mut next);
        if store.persist
            && let Err(err) = next.save()
        {
            eprintln!("[jayjay-gpui] failed to save config: {err}");
        }
        fonts::sync_from_config(&next);
        let mod_key_changed = next.mod_key() != store.config.mod_key();
        store.config = Arc::new(next);
        mod_key_changed
    });
    if mod_key_changed {
        cx.clear_key_bindings();
        cx.bind_keys(crate::app::actions::app_key_bindings(current(cx).mod_key()));
    }
    theme::refresh_for_current_appearance(cx);
    crate::app::menus::refresh(cx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::theme::Theme;

    #[gpui::test]
    fn ephemeral_updates_still_change_current_config(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            cx.set_global(AppConfigStore::new_ephemeral(AppConfig::default()));
            cx.set_global(Theme::light());

            update(cx, |c| c.diff.tree_file_list = true);

            assert!(current(cx).diff.tree_file_list);
        });
    }
}
