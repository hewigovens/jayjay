use gpui::{App, Menu, MenuItem, PathPromptOptions};

use super::actions::{
    OpenAbout, OpenBookmarkManager, OpenCommandPalette, OpenFind, OpenJujutsuDocumentation,
    OpenOperationLog, OpenRemoteRepository, OpenRepoInEditor, OpenRepoInTerminal, OpenRepository,
    OpenSettings, OpenUserGuide, Quit, ReportIssue, ResetZoom, ShowRepoInFileManager,
    ToggleHideGitLfsFiles, ToggleIgnoreWhitespace, ToggleSideBySideDiff, ToggleTreeFileList,
    ZoomIn, ZoomOut,
};
use super::config::{self, current};
use super::tools;
use crate::app::links::GUIDE_URL;
use crate::windows::settings::{SettingsSection, SettingsView};

const JUJUTSU_DOCS_URL: &str = "https://jj-vcs.github.io/jj/latest/";
const REPORT_ISSUE_URL: &str = "https://github.com/hewigovens/jayjay/issues";

pub fn install(cx: &mut App) {
    register_global_actions(cx);
    refresh(cx);
}

pub fn refresh(cx: &mut App) {
    let menus = app_menus(cx);
    cx.set_menus(menus);
}

pub fn app_menus(cx: &mut App) -> Vec<Menu> {
    let cfg = current(cx);
    vec![
        app_menu(),
        Menu::new("File").items([MenuItem::action("Open Repository...", OpenRepository)]),
        Menu::new("Edit").items([MenuItem::action("Find...", OpenFind)]),
        Menu::new("View").items([
            MenuItem::action("Zoom In", ZoomIn).disabled(cfg.font_size >= 24.),
            MenuItem::action("Zoom Out", ZoomOut).disabled(cfg.font_size <= 9.),
            MenuItem::action("Reset Zoom", ResetZoom).checked((cfg.font_size - 12.).abs() < 0.01),
        ]),
        Menu::new("Repository").items([
            MenuItem::action("Command Palette", OpenCommandPalette),
            MenuItem::action("Undo Last Operation", OpenOperationLog),
            MenuItem::separator(),
            MenuItem::action("Bookmark Manager", OpenBookmarkManager),
            MenuItem::separator(),
            MenuItem::action("View Remote Repository", OpenRemoteRepository),
            MenuItem::action("Show in File Manager", ShowRepoInFileManager),
            MenuItem::action(tools::open_in_editor_label(cx), OpenRepoInEditor),
            MenuItem::action(tools::open_in_terminal_label(cx), OpenRepoInTerminal),
        ]),
        Menu::new("Help").items([
            MenuItem::action("JayJay User Guide", OpenUserGuide),
            MenuItem::action("Jujutsu Documentation", OpenJujutsuDocumentation),
            MenuItem::separator(),
            MenuItem::action("Report an Issue", ReportIssue),
        ]),
    ]
}

fn app_menu() -> Menu {
    Menu::new("JayJay").items([
        MenuItem::action("About JayJay", OpenAbout),
        MenuItem::separator(),
        MenuItem::action("Settings...", OpenSettings),
        MenuItem::separator(),
        MenuItem::action("Quit JayJay", Quit),
    ])
}

fn register_global_actions(cx: &mut App) {
    cx.on_action(|_: &OpenSettings, cx| SettingsView::open(cx));
    cx.on_action(|_: &OpenAbout, cx| {
        SettingsView::open_section(SettingsSection::About, cx);
    });
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_action(|_: &OpenRepository, cx| open_repository(cx));
    cx.on_action(|_: &OpenUserGuide, cx| cx.open_url(GUIDE_URL));
    cx.on_action(|_: &OpenJujutsuDocumentation, cx| cx.open_url(JUJUTSU_DOCS_URL));
    cx.on_action(|_: &ReportIssue, cx| cx.open_url(REPORT_ISSUE_URL));
    cx.on_action(|_: &ZoomIn, cx| {
        toggle(cx, |c| {
            c.font_size = (c.font_size + 1.).min(24.);
        })
    });
    cx.on_action(|_: &ZoomOut, cx| {
        toggle(cx, |c| {
            c.font_size = (c.font_size - 1.).max(9.);
        })
    });
    cx.on_action(|_: &ResetZoom, cx| {
        toggle(cx, |c| {
            c.font_size = 12.;
        })
    });
    cx.on_action(|_: &ToggleSideBySideDiff, cx| {
        toggle(cx, |c| {
            c.diff.side_by_side ^= true;
        })
    });
    cx.on_action(|_: &ToggleIgnoreWhitespace, cx| {
        toggle(cx, |c| {
            c.diff.ignore_whitespace ^= true;
        })
    });
    cx.on_action(|_: &ToggleHideGitLfsFiles, cx| {
        toggle(cx, |c| {
            c.diff.hide_git_lfs ^= true;
        })
    });
    cx.on_action(|_: &ToggleTreeFileList, cx| {
        toggle(cx, |c| {
            c.diff.tree_file_list ^= true;
        })
    });
}

fn toggle(cx: &mut App, mutate: impl FnOnce(&mut config::AppConfig)) {
    config::update(cx, mutate);
}

fn open_repository(cx: &mut App) {
    let paths = cx.prompt_for_paths(PathPromptOptions {
        files: false,
        directories: true,
        multiple: false,
        prompt: Some("Choose a Jujutsu repository".into()),
    });
    cx.spawn(async move |cx| {
        let Ok(Ok(Some(paths))) = paths.await else {
            return;
        };
        let Some(path) = paths.into_iter().next() else {
            return;
        };
        cx.update(|cx| {
            crate::repo::open_repo_window(path, cx);
        });
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use gpui::{MenuItem, OwnedMenuItem};

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

    fn assert_checked(items: &[OwnedMenuItem], label: &str) {
        assert!(
            items.iter().any(|item| matches!(
                item,
                OwnedMenuItem::Action { name, checked: true, .. } if name == label
            )),
            "{label} should be checked"
        );
    }
}
