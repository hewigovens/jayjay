use gpui::{Menu, MenuItem, OwnedMenuItem};

use super::*;
use crate::app::config::{AppConfig, AppConfigStore};

#[gpui::test]
fn app_menus_reflect_zoom_state(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let cfg = AppConfig {
            font_size: 12.,
            ..Default::default()
        };
        cx.set_global(AppConfigStore::new(cfg));
        let menus = app_menus(cx);
        let view_menu = menus
            .into_iter()
            .find(|menu| menu.name.as_ref() == "View")
            .expect("View menu")
            .owned();
        assert_checked(&view_menu.items, "Reset Zoom");
    });
}

#[gpui::test]
fn app_menus_reflect_configured_editor(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let mut cfg = AppConfig::default();
        cfg.tools.external_editor = "zed".to_owned();
        cx.set_global(AppConfigStore::new(cfg));
        let menus = app_menus(cx);
        assert_action(&menus[4], "Open in Zed");
    });
}

#[gpui::test]
fn app_menus_reflect_configured_terminal(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let mut cfg = AppConfig::default();
        cfg.tools.terminal = "ghostty".to_owned();
        cx.set_global(AppConfigStore::new(cfg));
        let menus = app_menus(cx);
        assert_action(&menus[4], "Open in Ghostty");
    });
}

#[gpui::test]
fn file_menu_lists_recent_repositories(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let cfg = AppConfig {
            recent_repos: vec!["/tmp/jayjay-alpha".to_owned(), "/workspace/beta".to_owned()],
            ..Default::default()
        };
        cx.set_global(AppConfigStore::new(cfg));
        let menus = app_menus(cx);
        let file = menus
            .iter()
            .find(|menu| menu.name.as_ref() == "File")
            .expect("File menu");
        let recent = open_recent_submenu(file);

        assert_action(recent, "jayjay-alpha");
        assert_action(recent, "beta");
        assert_action(recent, "Clear");
    });
}

#[gpui::test]
fn file_menu_disables_empty_recent_repositories_label(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        cx.set_global(AppConfigStore::new(AppConfig::default()));
        let menus = app_menus(cx);
        let file = menus
            .iter()
            .find(|menu| menu.name.as_ref() == "File")
            .expect("File menu");
        let recent = open_recent_submenu(file);

        assert_disabled_action(recent, "No Recent Repositories");
    });
}

#[gpui::test]
fn help_menu_user_guide_opens_canonical_guide_url(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        cx.set_global(AppConfigStore::new(AppConfig::default()));
        install(cx);
        let menus = app_menus(cx);
        let help = menus
            .iter()
            .find(|menu| menu.name.as_ref() == "Help")
            .expect("Help menu");
        assert_action(help, "JayJay User Guide");

        // No active window in this test, so this exercises the global OpenUserGuide handler the menu item dispatches.
        cx.dispatch_action(&OpenUserGuide);
    });

    assert_eq!(cx.opened_url().as_deref(), Some(GUIDE_URL));
}

#[gpui::test]
fn help_menu_includes_send_feedback(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        cx.set_global(AppConfigStore::new(AppConfig::default()));
        let menus = app_menus(cx);
        let help = menus
            .iter()
            .find(|menu| menu.name.as_ref() == "Help")
            .expect("Help menu");
        assert_action(help, "Send Feedback");
    });
}

fn assert_action(menu: &Menu, label: &str) {
    assert!(
        menu.items.iter().any(|item| matches!(
            item,
            MenuItem::Action { name, .. } if name.as_ref() == label
        )),
        "{label} action missing from {} menu",
        menu.name
    );
}

fn assert_disabled_action(menu: &Menu, label: &str) {
    assert!(
        menu.items.iter().any(|item| matches!(
            item,
            MenuItem::Action { name, disabled: true, .. } if name.as_ref() == label
        )),
        "{label} disabled action missing from {} menu",
        menu.name
    );
}

fn open_recent_submenu(menu: &Menu) -> &Menu {
    menu.items
        .iter()
        .find_map(|item| match item {
            MenuItem::Submenu(submenu) if submenu.name.as_ref() == "Open Recent" => Some(submenu),
            _ => None,
        })
        .expect("Open Recent submenu")
}

fn assert_checked(items: &[OwnedMenuItem], label: &str) {
    assert!(
        items.iter().any(|item| matches!(
            item,
            OwnedMenuItem::Action { name, checked: true, .. } if name == label
        )),
        "{label} should be checked"
    );
}
